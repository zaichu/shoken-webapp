use std::sync::Arc;

use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use governor::{DefaultDirectRateLimiter, DefaultKeyedRateLimiter, Quota, RateLimiter};

use crate::errors::{ErrorDetails, ErrorResponse};

fn rate_limit_error() -> Response {
    let error_response = ErrorResponse {
        error: ErrorDetails {
            code: "RATE_LIMIT_EXCEEDED".to_string(),
            message: "リクエストが多すぎます。しばらくしてから再試行してください。".to_string(),
            details: None,
        },
    };
    (StatusCode::TOO_MANY_REQUESTS, Json(error_response)).into_response()
}

/// 毎秒 rps リクエストを許可するグローバルレート制限インスタンスを生成する
/// （jquants など外部 API クォータ保護に使用）
/// rps = 0 の場合は制限なし（None）を返す
pub fn build_rate_limiter(rps: u32) -> Option<Arc<DefaultDirectRateLimiter>> {
    let rps = std::num::NonZeroU32::new(rps)?;
    Some(Arc::new(RateLimiter::direct(Quota::per_second(rps))))
}

/// IP 単位のレート制限インスタンスを生成する（auth など DoS 対策に使用）
/// rps = 0 の場合は制限なし（None）を返す
pub fn build_keyed_rate_limiter(
    rps: u32,
) -> Option<Arc<DefaultKeyedRateLimiter<std::net::IpAddr>>> {
    let rps = std::num::NonZeroU32::new(rps)?;
    Some(Arc::new(RateLimiter::keyed(Quota::per_second(rps))))
}

/// クライアント IP を取得する
///
/// Fly.io は `fly-client-ip` を必ずセットし、クライアントによる偽装を防ぐ。
/// `X-Forwarded-For` はクライアントが任意の値を送れるため信頼しない。
/// `fly-client-ip` が存在しない場合（ローカル開発など）は 0.0.0.0 を返す。
/// これにより「プロキシ未経由の不明リクエスト」は共有バケットに入るため、
/// バイパス攻撃には使えない。
fn extract_client_ip(req: &Request<Body>) -> std::net::IpAddr {
    req.headers()
        .get("fly-client-ip")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse().ok())
        .unwrap_or(std::net::IpAddr::from([0, 0, 0, 0]))
}

/// グローバルレート制限ミドルウェア。制限超過時は 429 を返す
pub async fn rate_limit(
    limiter: Arc<DefaultDirectRateLimiter>,
    req: Request<Body>,
    next: Next,
) -> Response {
    if limiter.check().is_err() {
        return rate_limit_error();
    }
    next.run(req).await
}

/// IP 単位レート制限ミドルウェア。制限超過時は 429 を返す
pub async fn keyed_rate_limit(
    limiter: Arc<DefaultKeyedRateLimiter<std::net::IpAddr>>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let ip = extract_client_ip(&req);
    if limiter.check_key(&ip).is_err() {
        return rate_limit_error();
    }
    next.run(req).await
}

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
    use axum::{body::Body, http::Request, middleware, routing::post, Router};
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
        let app = Router::new()
            .route("/test", post(|| async { "ok" }))
            .layer(middleware::from_fn(add_security_headers));
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
    }

    #[test]
    fn test_build_rate_limiter_zero_returns_none() {
        assert!(build_rate_limiter(0).is_none());
    }

    #[test]
    fn test_build_rate_limiter_nonzero_returns_some() {
        assert!(build_rate_limiter(10).is_some());
    }

    #[tokio::test]
    async fn test_rate_limit_allows_within_quota() {
        let limiter = build_rate_limiter(100).unwrap();
        let app = Router::new()
            .route("/test", post(|| async { "ok" }))
            .layer(middleware::from_fn(move |req, next| {
                let l = limiter.clone();
                async move { rate_limit(l, req, next).await }
            }));
        let req = Request::builder()
            .method(Method::POST)
            .uri("/test")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
    }

    #[tokio::test]
    async fn test_rate_limit_blocks_excess_requests() {
        // rps=1 で複数回リクエストを送ると 429 が返る
        let limiter = build_rate_limiter(1).unwrap();
        let make_app = || {
            let l = limiter.clone();
            Router::new()
                .route("/test", post(|| async { "ok" }))
                .layer(middleware::from_fn(move |req, next| {
                    let l = l.clone();
                    async move { rate_limit(l, req, next).await }
                }))
        };
        // 1回目は通過
        let req = Request::builder()
            .method(Method::POST)
            .uri("/test")
            .body(Body::empty())
            .unwrap();
        let resp = make_app().oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
        // 2回目は超過
        let req = Request::builder()
            .method(Method::POST)
            .uri("/test")
            .body(Body::empty())
            .unwrap();
        let resp = make_app().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
    }

    #[test]
    fn test_build_keyed_rate_limiter_zero_returns_none() {
        assert!(build_keyed_rate_limiter(0).is_none());
    }

    #[test]
    fn test_build_keyed_rate_limiter_nonzero_returns_some() {
        assert!(build_keyed_rate_limiter(10).is_some());
    }

    #[tokio::test]
    async fn test_keyed_rate_limit_allows_within_quota() {
        let limiter = build_keyed_rate_limiter(100).unwrap();
        let app = Router::new()
            .route("/test", post(|| async { "ok" }))
            .layer(middleware::from_fn(move |req, next| {
                let l = limiter.clone();
                async move { keyed_rate_limit(l, req, next).await }
            }));
        let req = Request::builder()
            .method(Method::POST)
            .uri("/test")
            .header("fly-client-ip", "1.2.3.4")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
    }

    #[tokio::test]
    async fn test_keyed_rate_limit_blocks_same_ip() {
        // rps=1: 同一 IP からの 2 回目は 429
        let limiter = build_keyed_rate_limiter(1).unwrap();
        let make_app = || {
            let l = limiter.clone();
            Router::new()
                .route("/test", post(|| async { "ok" }))
                .layer(middleware::from_fn(move |req, next| {
                    let l = l.clone();
                    async move { keyed_rate_limit(l, req, next).await }
                }))
        };
        let req = Request::builder()
            .method(Method::POST)
            .uri("/test")
            .header("fly-client-ip", "1.2.3.4")
            .body(Body::empty())
            .unwrap();
        let resp = make_app().oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
        // 2回目（同一 IP）は超過
        let req = Request::builder()
            .method(Method::POST)
            .uri("/test")
            .header("fly-client-ip", "1.2.3.4")
            .body(Body::empty())
            .unwrap();
        let resp = make_app().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
    }

    #[tokio::test]
    async fn test_keyed_rate_limit_different_ips_independent() {
        // rps=1: 異なる IP は独立したバケット
        let limiter = build_keyed_rate_limiter(1).unwrap();
        let make_app = || {
            let l = limiter.clone();
            Router::new()
                .route("/test", post(|| async { "ok" }))
                .layer(middleware::from_fn(move |req, next| {
                    let l = l.clone();
                    async move { keyed_rate_limit(l, req, next).await }
                }))
        };
        // IP1 が 1 回消費
        let req = Request::builder()
            .method(Method::POST)
            .uri("/test")
            .header("fly-client-ip", "1.2.3.4")
            .body(Body::empty())
            .unwrap();
        let resp = make_app().oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
        // IP2 はまだ許可される（独立したバケット）
        let req = Request::builder()
            .method(Method::POST)
            .uri("/test")
            .header("fly-client-ip", "5.6.7.8")
            .body(Body::empty())
            .unwrap();
        let resp = make_app().oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
    }
}
