use crate::{
    errors::{ApiError, ErrorResponse},
    extractors::auth::AuthenticatedUser,
    extractors::validated_json::ValidatedJson,
    models::dividend_cache::{DividendPerShareBatchRequest, DividendPerShareBatchResponse},
    services::dividend_cache as dividend_cache_service,
    AppState,
};
use axum::{extract::State, response::IntoResponse, routing::post, Json, Router};

pub fn dividend_per_share_routes() -> Router<AppState> {
    Router::new().route("/dividends/per-share/batch", post(batch))
}

/// 配当利回り一括取得
#[utoipa::path(
    post,
    path = "/dividends/per-share/batch",
    operation_id = "dividend_per_share_batch",
    request_body = DividendPerShareBatchRequest,
    responses(
        (status = 200, body = DividendPerShareBatchResponse),
        (status = 400, body = ErrorResponse),
        (status = 401, body = ErrorResponse),
    ),
    security(("cookieAuth" = []))
)]
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
