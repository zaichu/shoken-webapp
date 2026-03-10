use axum::http::Request as AxumRequest;
use tower_http::trace::MakeSpan;

/// TraceLayer 用スパンメーカー（クエリパラメータを除外）
///
/// `TraceLayer::new_for_http()` の既定実装はクエリ文字列込みの URI を記録するため、
/// OAuth コールバックの `code` / `state` など機密パラメータがログに残る。
/// このスパンメーカーはパスのみを記録し機密情報の漏洩を防ぐ。
#[derive(Clone, Copy, Debug)]
pub struct PathOnlyMakeSpan;

impl<B> MakeSpan<B> for PathOnlyMakeSpan {
    fn make_span(&mut self, request: &AxumRequest<B>) -> tracing::Span {
        // x-request-id は SetRequestIdLayer より内側で span を作るため、
        // ここで読んだ値がそのままログ相関 ID として機能する
        let request_id = request
            .headers()
            .get("x-request-id")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("-");
        tracing::debug_span!(
            "http_request",
            method = %request.method(),
            path = request.uri().path(),
            request_id = request_id,
        )
    }
}
