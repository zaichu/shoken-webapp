use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use governor::{DefaultKeyedRateLimiter, Quota, RateLimiter};

use crate::errors::simple_error_response;

fn rate_limit_error() -> Response {
    simple_error_response(
        StatusCode::TOO_MANY_REQUESTS,
        "RATE_LIMIT_EXCEEDED",
        "リクエストが多すぎます。しばらくしてから再試行してください。".to_string(),
    )
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
    use {
        super::*,
        axum::{middleware, routing::post, Router},
        tower::ServiceExt,
    };
    fn test_route() -> Router {
        Router::new().route("/test", post(|| async { "ok" }))
    }
    fn keyed_app(limiter: Arc<DefaultKeyedRateLimiter<std::net::IpAddr>>) -> Router {
        test_route().layer(middleware::from_fn(move |req, next| {
            let limiter = limiter.clone();
            async move { keyed_rate_limit(limiter, req, next).await }
        }))
    }
    fn test_request(ip: Option<&str>) -> Request<Body> {
        let req = Request::builder()
            .method(axum::http::Method::POST)
            .uri("/test");
        match ip {
            Some(ip) => req.header("fly-client-ip", ip),
            None => req,
        }
        .body(Body::empty())
        .unwrap()
    }
    async fn assert_status(router: Router, ip: Option<&str>, expected: StatusCode) {
        assert_eq!(
            router.oneshot(test_request(ip)).await.unwrap().status(),
            expected
        );
    }
    #[tokio::test]
    async fn test_rate_limiters() {
        assert_eq!(
            (
                build_keyed_rate_limiter(0).is_none(),
                build_keyed_rate_limiter(10).is_some()
            ),
            (true, true)
        );
        let router = keyed_app(build_keyed_rate_limiter(1).unwrap());
        assert_status(router.clone(), Some("1.2.3.4"), StatusCode::OK).await;
        assert_status(router.clone(), Some("5.6.7.8"), StatusCode::OK).await;
        assert_status(router, Some("1.2.3.4"), StatusCode::TOO_MANY_REQUESTS).await;
    }
}
