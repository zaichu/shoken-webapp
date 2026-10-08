use axum::{Router, http::StatusCode, http::header, routing::get};
use tower_service::Service;
use worker::*;

// ドメインハンドラーの wasm 移植が済むまでは、チェックイン済みの契約ファイルをそのまま返す
const OPENAPI_JSON: &str = include_str!("../../docs/openapi.json");

fn router() -> Router {
    Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/ready", get(|| async { StatusCode::OK }))
        .route(
            "/api-docs/openapi.json",
            get(|| async { ([(header::CONTENT_TYPE, "application/json")], OPENAPI_JSON) }),
        )
}

#[event(fetch)]
async fn fetch(
    req: HttpRequest,
    _env: Env,
    _ctx: Context,
) -> Result<axum::http::Response<axum::body::Body>> {
    Ok(router().call(req).await?)
}
