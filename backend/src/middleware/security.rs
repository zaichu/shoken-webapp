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

    #[rustfmt::skip]
    fn test_app() -> Router { let allowed_origins = Arc::new(vec!["https://shoken-webapp.vercel.app".to_string(), "http://localhost:8080".to_string(), "http://127.0.0.1:8080".to_string(), "http://[::1]:8080".to_string(), "http://localhost.:8080".to_string()]); Router::new().route("/test", post(|| async { "ok" })).layer(middleware::from_fn(move |req, next| { let origins = allowed_origins.clone(); async move { validate_origin(origins, req, next).await } })) }

    #[rustfmt::skip]
    fn security_headers_app() -> Router { Router::new().route("/test", post(|| async { "ok" })).layer(middleware::from_fn(add_security_headers)) }

    #[rustfmt::skip]
    async fn oneshot_status(app: Router, method: Method, headers: &[(&str, &str)]) -> StatusCode { let builder = headers.iter().fold(Request::builder().method(method).uri("/test"), |builder, (name, value)| builder.header(*name, *value)); app.oneshot(builder.body(Body::empty()).unwrap()).await.unwrap().status() }

    #[tokio::test]
    async fn test_validate_origin() {
        #[rustfmt::skip]
        let origin_cases = [
            ("http://localhost:8080/some/page", Some("http://localhost:8080")),
            ("https://shoken-webapp.vercel.app", Some("https://shoken-webapp.vercel.app")),
            ("https://shoken-webapp.vercel.app.evil.com/steal", Some("https://shoken-webapp.vercel.app.evil.com")),
        ];
        for (input, expected) in origin_cases {
            assert_eq!(extract_origin(input), expected);
        }
        #[rustfmt::skip]
        assert_ne!(oneshot_status(test_app(), Method::GET, &[]).await, StatusCode::FORBIDDEN);

        #[rustfmt::skip]
        let cases: &[(&[(&str, &str)], StatusCode)] = &[
            (&[("origin", "http://localhost:8080")], StatusCode::OK),
            (&[("origin", "https://evil.example.com")], StatusCode::FORBIDDEN),
            (&[], StatusCode::OK),
            (&[("referer", "http://localhost:8080/some/page")], StatusCode::OK),
            (&[("referer", "https://evil.example.com/attack")], StatusCode::FORBIDDEN),
            (&[("referer", "https://shoken-webapp.vercel.app.evil.com/steal")], StatusCode::FORBIDDEN),
            (&[("origin", "https://shoken-webapp.vercel.app")], StatusCode::OK),
        ];
        for &(headers, expected) in cases {
            #[rustfmt::skip]
            assert_eq!(oneshot_status(test_app(), Method::POST, headers).await, expected);
        }
        let allowed_origins = Arc::new(vec!["http://localhost:8080".to_string()]);
        let app = Router::new()
            .route("/test", axum::routing::delete(|| async { "ok" }))
            .layer(middleware::from_fn(move |req, next| {
                let origins = allowed_origins.clone();
                async move { validate_origin(origins, req, next).await }
            }));
        #[rustfmt::skip]
        assert_eq!(oneshot_status(app, Method::DELETE, &[("origin", "https://evil.example.com")]).await, StatusCode::FORBIDDEN);
    }

    #[rustfmt::skip]
    async fn security_headers_response() -> axum::response::Response { security_headers_app().oneshot(Request::builder().method(Method::POST).uri("/test").body(Body::empty()).unwrap()).await.unwrap() }

    #[tokio::test]
    async fn test_security_headers() {
        {
            let _lock = ENV_MUTEX.lock().await;
            let _secure_cookie = EnvGuard::set("SECURE_COOKIE", None);
            let _backend_url = EnvGuard::set("BACKEND_URL", None);
            let resp = security_headers_response().await;
            #[rustfmt::skip]
            let header_cases = [
                ("X-Content-Type-Options", "nosniff"),
                ("X-Frame-Options", "DENY"),
                ("Referrer-Policy", "strict-origin-when-cross-origin"),
                ("Content-Security-Policy", "default-src 'none'"),
            ];
            for (name, expected) in header_cases {
                assert_eq!(resp.headers().get(name).unwrap(), expected);
            }
            assert!(
                resp.headers().get("Strict-Transport-Security").is_none(),
                "secure cookie 無効時は HSTS を付与しない"
            );
        }
        {
            let _lock = ENV_MUTEX.lock().await;
            let _secure_cookie = EnvGuard::set("SECURE_COOKIE", Some("true"));
            let resp = security_headers_response().await;
            #[rustfmt::skip]
            assert_eq!(resp.headers().get("Strict-Transport-Security").unwrap(), "max-age=31536000; includeSubDomains");
        }
    }
}
