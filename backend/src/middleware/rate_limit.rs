use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
#[cfg(not(target_arch = "wasm32"))]
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
#[cfg(not(target_arch = "wasm32"))]
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
#[cfg(not(target_arch = "wasm32"))]
fn extract_client_ip(req: &Request<Body>) -> std::net::IpAddr {
    req.headers()
        .get("fly-client-ip")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse().ok())
        .unwrap_or(std::net::IpAddr::from([0, 0, 0, 0]))
}

/// IP 単位レート制限ミドルウェア。制限超過時は 429 を返す
#[cfg(not(target_arch = "wasm32"))]
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

/// 指定ヘッダの値をレート制限キーとして取り出す。
/// 欠落・読み取り不可・空値は `fallback` にまとめる（0.0.0.0 の共有バケットと同じ意味）。
/// `X-Forwarded-For` 系はクライアントが偽装できるため対象にしない
#[cfg(any(target_arch = "wasm32", test))]
fn client_ip_key_from_header(
    headers: &axum::http::HeaderMap,
    header_name: &str,
    fallback: &str,
) -> String {
    headers
        .get(header_name)
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| fallback.to_string())
}

/// Cloudflare の `[[ratelimits]]` バインディングによるレート制限ミドルウェア。
/// キーは `cf-connecting-ip`（Cloudflare がクライアント IP を正規化して付与するため
/// クライアントによる偽装はできない）。`limit()` の呼び出し自体が失敗した場合は
/// 通過させる（バインディング障害で全リクエストを止めない fail-open）
#[cfg(target_arch = "wasm32")]
pub async fn binding_rate_limit(
    limiter: Arc<worker::RateLimiter>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let key = client_ip_key_from_header(req.headers(), "cf-connecting-ip", "unknown");
    match limiter.limit(key).await {
        Ok(outcome) if outcome.success => next.run(req).await,
        Ok(_) => rate_limit_error(),
        Err(e) => {
            worker::console_warn!("ratelimits binding error (fail-open): {e}");
            next.run(req).await
        }
    }
}
#[cfg(test)]
mod tests {
    use {
        super::*,
        axum::{Router, middleware, routing::post},
        tower::ServiceExt,
    };
    fn test_route() -> Router {
        Router::new().route("/test", post(|| async { "ok" }))
    }
    #[cfg(not(target_arch = "wasm32"))]
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
    #[cfg(not(target_arch = "wasm32"))]
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

    #[test]
    fn test_client_ip_key_from_header() {
        let mut headers = axum::http::HeaderMap::new();
        // 値があればそのままキーになる（前後空白は除去）
        headers.insert("cf-connecting-ip", " 203.0.113.10 ".parse().unwrap());
        assert_eq!(
            client_ip_key_from_header(&headers, "cf-connecting-ip", "unknown"),
            "203.0.113.10"
        );
        // 欠落・空値はフォールバックの共有バケットに入る
        assert_eq!(
            client_ip_key_from_header(&axum::http::HeaderMap::new(), "cf-connecting-ip", "unknown"),
            "unknown"
        );
        let mut empty = axum::http::HeaderMap::new();
        empty.insert("cf-connecting-ip", "".parse().unwrap());
        assert_eq!(
            client_ip_key_from_header(&empty, "cf-connecting-ip", "unknown"),
            "unknown"
        );
        // x-forwarded-for は信頼しない（指定ヘッダ以外は読まない）
        let mut spoof = axum::http::HeaderMap::new();
        spoof.insert("x-forwarded-for", "198.51.100.77".parse().unwrap());
        assert_eq!(
            client_ip_key_from_header(&spoof, "cf-connecting-ip", "unknown"),
            "unknown"
        );
    }
}
