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
}
