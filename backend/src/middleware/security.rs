use std::sync::Arc;

use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};

use crate::errors::{ErrorDetails, ErrorResponse};

/// セキュリティヘッダー付与ミドルウェア
pub async fn add_security_headers(req: Request<Body>, next: Next) -> Response {
    let mut response = next.run(req).await;
    let headers = response.headers_mut();
    headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
    headers.insert("X-Frame-Options", "DENY".parse().unwrap());
    headers.insert(
        "Referrer-Policy",
        "strict-origin-when-cross-origin".parse().unwrap(),
    );
    headers.insert(
        "Content-Security-Policy",
        "default-src 'none'".parse().unwrap(),
    );
    if crate::config::is_secure_cookie() {
        headers.insert(
            "Strict-Transport-Security",
            "max-age=31536000; includeSubDomains".parse().unwrap(),
        );
    }
    response
}

/// URL 文字列からオリジン部分（scheme://host[:port]）を抽出する
///
/// `starts_with` での前方一致では `https://example.com.evil/` のような
/// 類似ドメインに対してバイパスされるため、ホスト境界まで厳密に切り出す。
fn extract_origin(url: &str) -> Option<&str> {
    let after_scheme = url.find("://")?;
    let authority_start = after_scheme + 3;
    let end = url[authority_start..]
        .find('/')
        .map(|i| authority_start + i)
        .unwrap_or(url.len());
    Some(&url[..end])
}

/// CSRF 検証失敗レスポンスを生成する
fn csrf_error() -> Response {
    let error_response = ErrorResponse {
        error: ErrorDetails {
            code: "CSRF_ERROR".to_string(),
            message: "不正なリクエスト元です".to_string(),
            details: None,
        },
    };
    (StatusCode::FORBIDDEN, Json(error_response)).into_response()
}

/// Origin検証ミドルウェア（CSRF対策）
/// POST/PUT/DELETE リクエストに対して Origin ヘッダーを検証し、
/// 許可されたオリジンからのリクエストのみ通過させる。
/// allowed_origins は Config::cors_origins と一致させる。
pub async fn validate_origin(
    allowed_origins: Arc<Vec<String>>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let method = request.method().clone();

    // GET/HEAD/OPTIONS はスキップ
    if method == Method::GET || method == Method::HEAD || method == Method::OPTIONS {
        return next.run(request).await;
    }

    // Origin ヘッダーを取得
    let origin = request
        .headers()
        .get("origin")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    match origin {
        Some(ref o) if allowed_origins.iter().any(|a| a == o) => {
            // 許可されたオリジン → 通過
            next.run(request).await
        }
        None => {
            // Origin なし: Referer フォールバック検証
            // ブラウザはクロスオリジンリクエストで Origin を付与するが、
            // 一部の環境（リダイレクト後など）では省略されることがある。
            // Referer が存在する場合は許可済みオリジンとの前方一致で検証する。
            // Referer も存在しない場合は同一オリジンまたは非ブラウザクライアントとみなし通過する。
            let referer = request
                .headers()
                .get("referer")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());

            match referer {
                Some(ref r)
                    if extract_origin(r)
                        .is_some_and(|o| allowed_origins.iter().any(|a| a == o)) =>
                {
                    // 許可済みオリジンの Referer → 通過
                    next.run(request).await
                }
                Some(_) => {
                    // 不正な Referer → 拒否
                    csrf_error()
                }
                None => {
                    // Origin も Referer もなし → 同一オリジンまたは非ブラウザクライアントとみなし通過
                    next.run(request).await
                }
            }
        }
        Some(_) => {
            // 不正なオリジン → 拒否
            csrf_error()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_env::{EnvGuard, ENV_MUTEX};
    use axum::{middleware, routing::post, Router};
    use tower::ServiceExt;

    #[test]
    fn test_extract_origin_with_path() {
        assert_eq!(
            extract_origin("http://localhost:8080/some/page"),
            Some("http://localhost:8080")
        );
    }

    #[test]
    fn test_extract_origin_without_path() {
        assert_eq!(
            extract_origin("https://shoken-webapp.vercel.app"),
            Some("https://shoken-webapp.vercel.app")
        );
    }

    #[test]
    fn test_extract_origin_spoofed_domain() {
        // 許可ドメインを接頭辞に持つ偽装ドメインは別オリジンとして抽出される
        assert_eq!(
            extract_origin("https://shoken-webapp.vercel.app.evil.com/steal"),
            Some("https://shoken-webapp.vercel.app.evil.com")
        );
    }

    fn test_app() -> Router {
        let allowed_origins = Arc::new(vec![
            "https://shoken-webapp.vercel.app".to_string(),
            "http://localhost:8080".to_string(),
            "http://127.0.0.1:8080".to_string(),
            "http://[::1]:8080".to_string(),
            "http://localhost.:8080".to_string(),
        ]);
        Router::new()
            .route("/test", post(|| async { "ok" }))
            .layer(middleware::from_fn(move |req, next| {
                let origins = allowed_origins.clone();
                async move { validate_origin(origins, req, next).await }
            }))
    }

    fn security_headers_app() -> Router {
        Router::new()
            .route("/test", post(|| async { "ok" }))
            .layer(middleware::from_fn(add_security_headers))
    }

    #[tokio::test]
    async fn test_get_request_passes() {
        let app = test_app();
        let req = Request::builder()
            .method(Method::GET)
            .uri("/test")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        // GET はルート定義がないので 405 だが、ミドルウェアは通過
        assert_ne!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_post_with_allowed_origin() {
        let app = test_app();
        let req = Request::builder()
            .method(Method::POST)
            .uri("/test")
            .header("origin", "http://localhost:8080")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_post_with_disallowed_origin() {
        let app = test_app();
        let req = Request::builder()
            .method(Method::POST)
            .uri("/test")
            .header("origin", "https://evil.example.com")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_post_without_origin() {
        let app = test_app();
        let req = Request::builder()
            .method(Method::POST)
            .uri("/test")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        // Origin なしは同一オリジンとみなし通過
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_delete_with_disallowed_origin() {
        let allowed_origins = Arc::new(vec!["http://localhost:8080".to_string()]);
        let app = Router::new()
            .route("/test", axum::routing::delete(|| async { "ok" }))
            .layer(middleware::from_fn(move |req, next| {
                let origins = allowed_origins.clone();
                async move { validate_origin(origins, req, next).await }
            }));

        let req = Request::builder()
            .method(Method::DELETE)
            .uri("/test")
            .header("origin", "https://evil.example.com")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_post_with_allowed_referer_no_origin() {
        // Origin なし・許可済み Referer あり → 通過
        let app = test_app();
        let req = Request::builder()
            .method(Method::POST)
            .uri("/test")
            .header("referer", "http://localhost:8080/some/page")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_post_with_disallowed_referer_no_origin() {
        // Origin なし・不正な Referer → 403
        let app = test_app();
        let req = Request::builder()
            .method(Method::POST)
            .uri("/test")
            .header("referer", "https://evil.example.com/attack")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_post_with_spoofed_referer_prefix_is_rejected() {
        // 許可オリジンを接頭辞に持つ偽装ドメイン → starts_with バイパスを防ぐ
        let app = test_app();
        let req = Request::builder()
            .method(Method::POST)
            .uri("/test")
            .header("referer", "https://shoken-webapp.vercel.app.evil.com/steal")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_post_with_production_origin() {
        let app = test_app();
        let req = Request::builder()
            .method(Method::POST)
            .uri("/test")
            .header("origin", "https://shoken-webapp.vercel.app")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_security_headers_present() {
        let _lock = ENV_MUTEX.lock().await;
        let _secure_cookie = EnvGuard::set("SECURE_COOKIE", None);
        let _backend_url = EnvGuard::set("BACKEND_URL", None);

        let app = security_headers_app();
        let req = Request::builder()
            .method(Method::POST)
            .uri("/test")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(
            resp.headers().get("X-Content-Type-Options").unwrap(),
            "nosniff"
        );
        assert_eq!(resp.headers().get("X-Frame-Options").unwrap(), "DENY");
        assert_eq!(
            resp.headers().get("Referrer-Policy").unwrap(),
            "strict-origin-when-cross-origin"
        );
        assert_eq!(
            resp.headers().get("Content-Security-Policy").unwrap(),
            "default-src 'none'"
        );
        assert!(
            resp.headers().get("Strict-Transport-Security").is_none(),
            "secure cookie 無効時は HSTS を付与しない"
        );
    }

    #[tokio::test]
    async fn test_security_headers_include_hsts_when_secure_cookie_enabled() {
        let _lock = ENV_MUTEX.lock().await;
        let _secure_cookie = EnvGuard::set("SECURE_COOKIE", Some("true"));

        let app = security_headers_app();
        let req = Request::builder()
            .method(Method::POST)
            .uri("/test")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();

        assert_eq!(
            resp.headers().get("Strict-Transport-Security").unwrap(),
            "max-age=31536000; includeSubDomains"
        );
    }
}
