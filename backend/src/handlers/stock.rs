use crate::{
    errors::{ApiError, ErrorResponse},
    extractors::{auth::AuthenticatedUser, validated_json::ValidatedJson},
    models::stock::Stock,
    services::stock as stock_service,
    AppState,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};

#[allow(dead_code)]
pub fn stock_routes() -> Router<AppState> {
    Router::new()
        .route("/stocks", post(create_stock))
        .route("/stocks/{query}", get(search_stock))
}

/// 銘柄情報を検索（コードまたは名前）
#[utoipa::path(
    get,
    path = "/stocks/{query}",
    operation_id = "stock_search",
    params(
        ("query" = String, Path, description = "銘柄コードまたは銘柄名")
    ),
    responses(
        (status = 200, body = Stock),
        (status = 404, body = ErrorResponse),
    ),
)]
#[allow(dead_code)]
pub async fn search_stock(
    Path(search_query): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let stock = stock_service::search(&state.pool, &search_query).await?;
    Ok((StatusCode::OK, Json(stock)))
}

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
