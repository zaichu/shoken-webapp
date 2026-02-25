use crate::{
    errors::ApiError, extractors::auth::AuthenticatedUser,
    extractors::validated_json::ValidatedJson,
    models::domestic_stock::BulkCreateDomesticStockRequest,
    services::domestic_stock as domestic_stock_service, state::AppState,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

/// 認証ユーザーの国内株式取引一覧を取得
pub async fn list(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<impl IntoResponse, ApiError> {
    let stocks = domestic_stock_service::list(&state.pool, auth_user.id()).await?;
    Ok((StatusCode::OK, Json(stocks)))
}

/// 国内株式取引を一括追加（全件挿入）
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
