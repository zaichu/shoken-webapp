use crate::{
    errors::{ApiError, ErrorResponse},
    extractors::auth::AuthenticatedUser,
    extractors::validated_json::ValidatedJson,
    models::common::BulkCreateResponse,
    models::mutualfund::{BulkCreateMutualfundRequest, Mutualfund},
    services::mutualfund as mutualfund_service,
    state::AppState,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

/// 認証ユーザーの投資信託一覧を取得
#[utoipa::path(
    get,
    path = "/mutualfunds",
    responses(
        (status = 200, body = Vec<Mutualfund>),
        (status = 401, body = ErrorResponse),
    ),
    security(("cookieAuth" = []))
)]
pub async fn list(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<impl IntoResponse, ApiError> {
    let funds = mutualfund_service::list(&state.pool, auth_user.id()).await?;
    Ok((StatusCode::OK, Json(funds)))
}

/// 投資信託を一括追加（重複はスキップ）
#[utoipa::path(
    post,
    path = "/mutualfunds/bulk",
    request_body = BulkCreateMutualfundRequest,
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
    ValidatedJson(data): ValidatedJson<BulkCreateMutualfundRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let response =
        mutualfund_service::bulk_create(&state.pool, auth_user.id(), &data.items).await?;
    Ok((StatusCode::CREATED, Json(response)))
}

/// 認証ユーザーの投資信託を全削除
#[utoipa::path(
    delete,
    path = "/mutualfunds/all",
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
    mutualfund_service::delete_all(&state.pool, auth_user.id()).await?;
    Ok((
        StatusCode::OK,
        Json(serde_json::json!({"message": "全ての投資信託データを削除しました"})),
    ))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_compilation() {
        assert!(true);
    }
}
