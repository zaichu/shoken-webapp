#[cfg(target_arch = "wasm32")]
use crate::errors::simple_error_response;
#[cfg(target_arch = "wasm32")]
use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
#[cfg(target_arch = "wasm32")]
use std::sync::Arc;

#[cfg(target_arch = "wasm32")]
fn rate_limit_error() -> Response {
    simple_error_response(
        StatusCode::TOO_MANY_REQUESTS,
        "RATE_LIMIT_EXCEEDED",
        "リクエストが多すぎます。しばらくしてから再試行してください。".to_string(),
    )
}

#[cfg(target_arch = "wasm32")]
fn binding_unavailable_error() -> Response {
    simple_error_response(
        StatusCode::SERVICE_UNAVAILABLE,
        "SERVICE_UNAVAILABLE",
        "一時的に利用できません。しばらくしてから再試行してください。".to_string(),
    )
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
        .map_or_else(|| fallback.to_string(), str::to_owned)
}

/// バインディング失敗時に拒否すべきか。本番だけ拒否し、開発・テストは通過させる。
#[cfg(any(target_arch = "wasm32", test))]
pub(crate) const fn binding_failure_should_reject(is_production: bool) -> bool {
    is_production
}

/// Cloudflare の `[[ratelimits]]` バインディングによるレート制限ミドルウェア。
/// キーは `cf-connecting-ip`（Cloudflare がクライアント IP を正規化して付与するため
/// クライアントによる偽装はできない）。
#[cfg(target_arch = "wasm32")]
pub async fn binding_rate_limit(
    limiter: Arc<worker::RateLimiter>,
    is_production: bool,
    req: Request<Body>,
    next: Next,
) -> Response {
    let key = client_ip_key_from_header(req.headers(), "cf-connecting-ip", "unknown");
    match limiter.limit(key).await {
        Ok(outcome) if outcome.success => next.run(req).await,
        Ok(_) => rate_limit_error(),
        Err(e) => {
            if binding_failure_should_reject(is_production) {
                worker::console_warn!("ratelimits binding error: {e}");
                binding_unavailable_error()
            } else {
                worker::console_warn!("ratelimits binding error (fail-open): {e}");
                next.run(req).await
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{HeaderMap, HeaderValue};

    #[test]
    fn test_client_ip_key_from_header() {
        let mut headers = HeaderMap::new();
        // 値があればそのままキーになる（前後空白は除去）
        headers.insert(
            "cf-connecting-ip",
            HeaderValue::from_static(" 203.0.113.10 "),
        );
        assert_eq!(
            client_ip_key_from_header(&headers, "cf-connecting-ip", "unknown"),
            "203.0.113.10"
        );
        // 欠落・空値はフォールバックの共有バケットに入る
        assert_eq!(
            client_ip_key_from_header(&HeaderMap::new(), "cf-connecting-ip", "unknown"),
            "unknown"
        );
        let mut empty = HeaderMap::new();
        empty.insert("cf-connecting-ip", HeaderValue::from_static(""));
        assert_eq!(
            client_ip_key_from_header(&empty, "cf-connecting-ip", "unknown"),
            "unknown"
        );
        // x-forwarded-for は信頼しない（指定ヘッダ以外は読まない）
        let mut spoof = HeaderMap::new();
        spoof.insert("x-forwarded-for", HeaderValue::from_static("198.51.100.77"));
        assert_eq!(
            client_ip_key_from_header(&spoof, "cf-connecting-ip", "unknown"),
            "unknown"
        );
    }

    #[test]
    fn test_binding_failure_should_reject_only_in_production() {
        assert!(binding_failure_should_reject(true));
        assert!(!binding_failure_should_reject(false));
    }
}
