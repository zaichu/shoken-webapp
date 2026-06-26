use crate::{
    errors::{ApiError, ErrorResponse},
    extractors::{auth::AuthenticatedUser, validated_json::ValidatedJson},
    models::stock::Stock,
    services::stock as stock_service,
    state::AppState,
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::Deserialize;
use utoipa::ToSchema;

// 銘柄ハンドラー
// ---------------------------------------------------------------------------

/// 銘柄検索クエリパラメータ（v1）
#[derive(Debug, Deserialize, ToSchema)]
pub struct StockSearchQuery {
    /// 銘柄コードまたは銘柄名
    pub query: String,
}

/// 銘柄情報を検索（v1: クエリパラメータ）
#[utoipa::path(
    get,
    path = "/api/v1/stocks",
    operation_id = "v1_stock_search",
    params(
        ("query" = String, Query, description = "銘柄コードまたは銘柄名")
    ),
    responses(
        (status = 200, body = Stock),
        (status = 404, body = ErrorResponse),
    ),
)]
pub async fn search(
    State(state): State<AppState>,
    Query(params): Query<StockSearchQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let stock = stock_service::search(&state.pool, &params.query).await?;
    Ok((StatusCode::OK, Json(stock)))
}

/// 銘柄情報を追加（v1）
#[utoipa::path(
    post,
    path = "/api/v1/stocks",
    operation_id = "v1_stock_create",
    request_body = Stock,
    responses(
        (status = 201, body = Stock),
        (status = 400, body = ErrorResponse),
        (status = 401, body = ErrorResponse),
    ),
    security(("cookieAuth" = []))
)]
pub async fn create(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    ValidatedJson(data): ValidatedJson<Stock>,
) -> Result<impl IntoResponse, ApiError> {
    crate::handlers::stock::create_stock(State(state), auth_user, ValidatedJson(data)).await
}

// ---------------------------------------------------------------------------
