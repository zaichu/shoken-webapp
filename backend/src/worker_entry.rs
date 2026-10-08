use axum::{Router, http::StatusCode, http::header, routing::get};
use tower_service::Service;
use worker::*;

// ドメインハンドラーの wasm 移植が済むまでは、チェックイン済みの契約ファイルをそのまま返す
const OPENAPI_JSON: &str = include_str!("../../docs/openapi.json");

fn router(db: Option<crate::db::Db>) -> Router {
    Router::new()
        .route("/health", get(|| async { "OK" }))
        .route(
            "/ready",
            get(move || {
                let db = db.clone();
                async move {
                    let Some(db) = db else {
                        return StatusCode::SERVICE_UNAVAILABLE;
                    };
                    match crate::db::query_scalar::<i32>("SELECT 1", vec![])
                        .fetch_one(&db)
                        .await
                    {
                        Ok(_) => StatusCode::OK,
                        Err(e) => {
                            console_error!("ready check の DB クエリに失敗: {e}");
                            StatusCode::SERVICE_UNAVAILABLE
                        }
                    }
                }
            }),
        )
        .route(
            "/api-docs/openapi.json",
            get(|| async { ([(header::CONTENT_TYPE, "application/json")], OPENAPI_JSON) }),
        )
}

#[event(fetch)]
async fn fetch(
    req: HttpRequest,
    env: Env,
    _ctx: Context,
) -> Result<axum::http::Response<axum::body::Body>> {
    let db = env
        .hyperdrive("HYPERDRIVE")
        .map(|h| crate::db::Db::from_hyperdrive(&h))
        .ok();
    Ok(router(db).call(req).await?)
}
