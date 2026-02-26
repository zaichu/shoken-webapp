use crate::{
    errors::{ApiError, ErrorResponse},
    extractors::auth::AuthenticatedUser,
    extractors::validated_json::ValidatedJson,
    models::common::BulkCreateResponse,
    models::domestic_stock::{BulkCreateDomesticStockRequest, DomesticStock},
    services::domestic_stock as domestic_stock_service,
    state::AppState,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

/// 認証ユーザーの国内株式取引一覧を取得
#[utoipa::path(
    get,
    path = "/domestic-stocks",
    responses(
        (status = 200, body = Vec<DomesticStock>),
        (status = 401, body = ErrorResponse),
    ),
    security(("cookieAuth" = []))
)]
pub async fn list(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<impl IntoResponse, ApiError> {
    let stocks = domestic_stock_service::list(&state.pool, auth_user.id()).await?;
    Ok((StatusCode::OK, Json(stocks)))
}

/// 国内株式取引を一括追加（全件挿入）
#[utoipa::path(
    post,
    path = "/domestic-stocks/bulk",
    request_body = BulkCreateDomesticStockRequest,
    responses(
        (status = 201, body = BulkCreateResponse),
        (status = 400, body = ErrorResponse),
        (status = 401, body = ErrorResponse),
    ),
    security(("cookieAuth" = []))
)]
pub async fn bulk_create(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    ValidatedJson(data): ValidatedJson<BulkCreateDomesticStockRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let response =
        domestic_stock_service::bulk_create(&state.pool, auth_user.id(), &data.items).await?;
    Ok((StatusCode::CREATED, Json(response)))
}

/// 認証ユーザーの国内株式取引を全削除
#[utoipa::path(
    delete,
    path = "/domestic-stocks/all",
    responses(
        (status = 200, description = "全削除成功"),
        (status = 401, body = ErrorResponse),
    ),
    security(("cookieAuth" = []))
)]
pub async fn delete_all(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<impl IntoResponse, ApiError> {
    domestic_stock_service::delete_all(&state.pool, auth_user.id()).await?;
    Ok((
        StatusCode::OK,
        Json(serde_json::json!({"message": "全ての国内株式取引データを削除しました"})),
    ))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_compilation() {
        assert!(true);
    }
}
