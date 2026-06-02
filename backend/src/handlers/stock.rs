use crate::{
    errors::{ApiError, ErrorResponse},
    extractors::{auth::AuthenticatedUser, validated_json::ValidatedJson},
    models::stock::Stock,
    services::stock as stock_service,
    AppState,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

/// 銘柄情報を追加（認証必須）
#[utoipa::path(
    post,
    path = "/stocks",
    operation_id = "stock_create",
    request_body = Stock,
    responses(
        (status = 201, body = Stock),
        (status = 400, body = ErrorResponse),
        (status = 401, body = ErrorResponse),
    ),
    security(("cookieAuth" = []))
)]
pub async fn create_stock(
    State(state): State<AppState>,
    _auth_user: AuthenticatedUser,
    ValidatedJson(data): ValidatedJson<Stock>,
) -> Result<impl IntoResponse, ApiError> {
    let stock = stock_service::create(&state.pool, &data).await?;
    Ok((StatusCode::CREATED, Json(stock)))
}
#[cfg(test)]
mod tests;
