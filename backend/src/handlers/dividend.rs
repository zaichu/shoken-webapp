use crate::{
    errors::ApiError, extractors::auth::AuthenticatedUser,
    extractors::validated_json::ValidatedJson, models::dividend::BulkCreateDividendRequest,
    services::dividend as dividend_service, state::AppState,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

/// 認証ユーザーの配当金一覧を取得
pub async fn list(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<impl IntoResponse, ApiError> {
    let dividends = dividend_service::list(&state.pool, auth_user.id()).await?;
    Ok((StatusCode::OK, Json(dividends)))
}

/// 配当金を一括追加（重複はスキップ）
pub async fn bulk_create(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    ValidatedJson(data): ValidatedJson<BulkCreateDividendRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let response = dividend_service::bulk_create(&state.pool, auth_user.id(), &data.items).await?;
    Ok((StatusCode::CREATED, Json(response)))
}

/// 認証ユーザーの配当金を全削除
pub async fn delete_all(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<impl IntoResponse, ApiError> {
    dividend_service::delete_all(&state.pool, auth_user.id()).await?;
    Ok((
        StatusCode::OK,
        Json(serde_json::json!({"message": "全ての配当金データを削除しました"})),
    ))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_compilation() {
        assert!(true);
    }
}
