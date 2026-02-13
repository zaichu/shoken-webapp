use crate::{
    errors::ApiError, extractors::auth::AuthenticatedUser,
    extractors::validated_json::ValidatedJson,
    models::asset_balance::BulkCreateAssetBalanceRequest,
    services::asset_balance as asset_balance_service, state::AppState,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

/// 認証ユーザーの保有銘柄一覧を取得
pub async fn list(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<impl IntoResponse, ApiError> {
    let balances = asset_balance_service::list(&state.pool, auth_user.id()).await?;
    Ok((StatusCode::OK, Json(balances)))
}

/// 保有銘柄を一括追加（既存は更新）
pub async fn bulk_create(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    ValidatedJson(data): ValidatedJson<BulkCreateAssetBalanceRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let response =
        asset_balance_service::bulk_create(&state.pool, auth_user.id(), &data.items).await?;
    Ok((StatusCode::CREATED, Json(response)))
}

/// 認証ユーザーの保有銘柄を全削除
pub async fn delete_all(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<impl IntoResponse, ApiError> {
    asset_balance_service::delete_all(&state.pool, auth_user.id()).await?;
    Ok((
        StatusCode::OK,
        Json(serde_json::json!({"message": "全ての保有銘柄データを削除しました"})),
    ))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_compilation() {
        assert!(true);
    }
}
