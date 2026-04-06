use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{middleware, routing::post, Router};
    use tower::ServiceExt;

    #[test]
    fn test_build_rate_limiters() {
        assert!(build_rate_limiter(0).is_none());
        assert!(build_rate_limiter(10).is_some());
        assert!(build_keyed_rate_limiter(0).is_none());
        assert!(build_keyed_rate_limiter(10).is_some());
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
            .method(axum::http::Method::POST)
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
            .method(axum::http::Method::POST)
            .uri("/test")
            .body(Body::empty())
            .unwrap();
        let resp = make_app().oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
        // 2回目は超過
        let req = Request::builder()
            .method(axum::http::Method::POST)
            .uri("/test")
            .body(Body::empty())
            .unwrap();
        let resp = make_app().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
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
            .method(axum::http::Method::POST)
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
            .method(axum::http::Method::POST)
            .uri("/test")
            .header("fly-client-ip", "1.2.3.4")
            .body(Body::empty())
            .unwrap();
        let resp = make_app().oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
        // 2回目（同一 IP）は超過
        let req = Request::builder()
            .method(axum::http::Method::POST)
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
            .method(axum::http::Method::POST)
            .uri("/test")
            .header("fly-client-ip", "1.2.3.4")
            .body(Body::empty())
            .unwrap();
        let resp = make_app().oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
        // IP2 はまだ許可される（独立したバケット）
        let req = Request::builder()
            .method(axum::http::Method::POST)
            .uri("/test")
            .header("fly-client-ip", "5.6.7.8")
            .body(Body::empty())
            .unwrap();
        let resp = make_app().oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
    }
}
