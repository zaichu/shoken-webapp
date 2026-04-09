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

#[cfg(test)] #[rustfmt::skip] mod tests {
    use {super::*, axum::{middleware, routing::post, Router}, tower::ServiceExt};
    fn test_route() -> Router { Router::new().route("/test", post(|| async { "ok" })) }
    fn direct_app(limiter: Arc<DefaultDirectRateLimiter>) -> Router { test_route().layer(middleware::from_fn(move |req, next| { let limiter = limiter.clone(); async move { rate_limit(limiter, req, next).await } })) }
    fn keyed_app(limiter: Arc<DefaultKeyedRateLimiter<std::net::IpAddr>>) -> Router { test_route().layer(middleware::from_fn(move |req, next| { let limiter = limiter.clone(); async move { keyed_rate_limit(limiter, req, next).await } })) }
    fn test_request(ip: Option<&str>) -> Request<Body> { let req = Request::builder().method(axum::http::Method::POST).uri("/test"); match ip { Some(ip) => req.header("fly-client-ip", ip), None => req }.body(Body::empty()).unwrap() }
    async fn assert_status(router: Router, ip: Option<&str>, expected: StatusCode) { assert_eq!(router.oneshot(test_request(ip)).await.unwrap().status(), expected); }
    #[tokio::test]
    async fn test_rate_limiters() {
        assert_eq!((build_rate_limiter(0).is_none(), build_rate_limiter(10).is_some(), build_keyed_rate_limiter(0).is_none(), build_keyed_rate_limiter(10).is_some()), (true, true, true, true));
        let router = direct_app(build_rate_limiter(1).unwrap()); assert_status(router.clone(), None, StatusCode::OK).await; assert_status(router, None, StatusCode::TOO_MANY_REQUESTS).await;
        let router = keyed_app(build_keyed_rate_limiter(1).unwrap()); assert_status(router.clone(), Some("1.2.3.4"), StatusCode::OK).await; assert_status(router.clone(), Some("5.6.7.8"), StatusCode::OK).await; assert_status(router, Some("1.2.3.4"), StatusCode::TOO_MANY_REQUESTS).await;
    }
}
