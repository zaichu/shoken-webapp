use crate::{
    AppState,
    errors::ApiError,
    extractors::auth::AuthenticatedUser,
    extractors::validated_json::ValidatedJson,
    models::dividend_cache::{DividendPerShareBatchRequest, DividendPerShareBatchResponse},
    services::dividend_cache as dividend_cache_service,
};
use axum::{Json, extract::State, response::IntoResponse};

pub async fn batch(
    State(state): State<AppState>,
    _auth_user: AuthenticatedUser,
    ValidatedJson(data): ValidatedJson<DividendPerShareBatchRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // native は reqwest クライアントとスレッドプール spawn によるバックグラウンド更新。
    // Workers は isolate 単位で spawn できないため pending 行を積み、cron が消化する
    #[cfg(not(target_arch = "wasm32"))]
    let items = dividend_cache_service::get_batch(
        &state.pool,
        &state.client,
        state.secrets.jquants_api_key.as_deref(),
        &data.security_codes,
        &state.dividend_cache.running,
    )
    .await?;
    #[cfg(target_arch = "wasm32")]
    let items = dividend_cache_service::get_batch(
        &state.pool,
        state.jquants_client.as_ref(),
        &data.security_codes,
    )
    .await?;

    Ok(Json(DividendPerShareBatchResponse { items }))
}
