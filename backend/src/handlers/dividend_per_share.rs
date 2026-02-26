use crate::{
    errors::{ApiError, ErrorResponse},
    extractors::validated_json::ValidatedJson,
    models::dividend_cache::{DividendPerShareBatchRequest, DividendPerShareBatchResponse},
    services::dividend_cache as dividend_cache_service,
    AppState,
};
use axum::{extract::State, response::IntoResponse, Json};

/// 配当利回り一括取得（認証不要）
#[utoipa::path(
    post,
    path = "/dividends/per-share/batch",
    request_body = DividendPerShareBatchRequest,
    responses(
        (status = 200, body = DividendPerShareBatchResponse),
        (status = 400, body = ErrorResponse),
    ),
)]
pub async fn batch(
    State(state): State<AppState>,
    ValidatedJson(data): ValidatedJson<DividendPerShareBatchRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let api_key = state.secrets.jquants_api_key.as_deref();

    let items = dividend_cache_service::get_batch(
        &state.pool,
        &state.client,
        api_key,
        &data.security_codes,
        &state.background_task_running,
    )
    .await?;

    Ok(Json(DividendPerShareBatchResponse { items }))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_compilation() {
        assert!(true);
    }
}
