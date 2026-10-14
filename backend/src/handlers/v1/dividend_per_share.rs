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
    // isolate 内で spawn できないため、未取得/TTL切れ銘柄は pending 行として積み、
    // scheduled イベント（cron）が消化する
    let items = dividend_cache_service::get_batch(
        &state.pool,
        state.jquants_client.as_ref(),
        &data.security_codes,
    )
    .await?;

    Ok(Json(DividendPerShareBatchResponse { items }))
}
