use crate::{
    errors::ApiError,
    extractors::auth::AuthenticatedUser,
    extractors::validated_json::ValidatedJson,
    models::dividend_cache::{DividendPerShareBatchRequest, DividendPerShareBatchResponse},
    services::dividend_cache as dividend_cache_service,
    AppState,
};
use axum::{extract::State, response::IntoResponse, Json};

pub async fn batch(
    State(state): State<AppState>,
    _auth_user: AuthenticatedUser,
    ValidatedJson(data): ValidatedJson<DividendPerShareBatchRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let api_key = state.secrets.jquants_api_key.as_deref();

    let items = dividend_cache_service::get_batch(
        &state.pool,
        &state.client,
        api_key,
        &data.security_codes,
        &state.dividend_cache.running,
    )
    .await?;

    Ok(Json(DividendPerShareBatchResponse { items }))
}
