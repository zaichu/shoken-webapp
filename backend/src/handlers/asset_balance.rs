use crate::{
    errors::{ApiError, ErrorResponse},
    extractors::auth::AuthenticatedUser,
    extractors::validated_json::ValidatedJson,
    models::asset_balance::{AssetBalance, BulkCreateAssetBalanceRequest},
    models::common::{BulkCreateResponse, MessageResponse},
    models::csv_import::{CsvPreviewResponse, CsvUploadForm, CsvUploadResponse},
    services::asset_balance as asset_balance_service,
    state::AppState,
};
use axum::{extract::Multipart, extract::State, http::StatusCode, response::IntoResponse, Json};

/// 認証ユーザーの保有銘柄一覧を取得
#[utoipa::path(
    get,
    path = "/asset-balances",
    operation_id = "asset_balance_list",
    responses(
        (status = 200, body = Vec<AssetBalance>),
        (status = 401, body = ErrorResponse),
    ),
    security(("cookieAuth" = []))
)]
pub async fn list(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<impl IntoResponse, ApiError> {
    let balances = asset_balance_service::list(&state.pool, auth_user.id()).await?;
    Ok((StatusCode::OK, Json(balances)))
}

/// 保有銘柄を一括追加（既存は更新）
#[utoipa::path(
    post,
    path = "/asset-balances/bulk",
    operation_id = "asset_balance_bulk_create",
    request_body = BulkCreateAssetBalanceRequest,
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
    ValidatedJson(data): ValidatedJson<BulkCreateAssetBalanceRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let response =
        asset_balance_service::bulk_create(&state.pool, auth_user.id(), &data.items).await?;
    Ok((StatusCode::CREATED, Json(response)))
}

/// CSV ファイルをパースして保存前プレビューを返す（DB 書き込みなし）
#[utoipa::path(
    post,
    path = "/asset-balances/csv/preview",
    operation_id = "asset_balance_preview_csv",
    request_body(content = CsvUploadForm, content_type = "multipart/form-data"),
    responses(
        (status = 200, body = CsvPreviewResponse),
        (status = 400, body = ErrorResponse),
        (status = 401, body = ErrorResponse),
    ),
    security(("cookieAuth" = []))
)]
pub async fn preview_csv(
    _auth_user: AuthenticatedUser,
    multipart: Multipart,
) -> Result<impl IntoResponse, ApiError> {
    let bytes = crate::handlers::csv_import::read_csv_file_bytes(multipart).await?;
    let response = asset_balance_service::preview_csv(&bytes)?;
    Ok((StatusCode::OK, Json(response)))
}

/// CSV ファイルをアップロードして保有銘柄を一括登録（既存データ全置換）
#[utoipa::path(
    post,
    path = "/asset-balances/csv",
    operation_id = "asset_balance_upload_csv",
    request_body(content = CsvUploadForm, content_type = "multipart/form-data"),
    responses(
        (status = 201, body = CsvUploadResponse),
        (status = 400, body = ErrorResponse),
        (status = 401, body = ErrorResponse),
    ),
    security(("cookieAuth" = []))
)]
pub async fn upload_csv(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    multipart: Multipart,
) -> Result<impl IntoResponse, ApiError> {
    let bytes = crate::handlers::csv_import::read_csv_file_bytes(multipart).await?;
    let response = asset_balance_service::upload_csv(&state.pool, auth_user.id(), &bytes).await?;
    Ok((StatusCode::CREATED, Json(response)))
}

/// 認証ユーザーの保有銘柄を全削除
#[utoipa::path(
    delete,
    path = "/asset-balances/all",
    operation_id = "asset_balance_delete_all",
    responses(
        (status = 200, body = MessageResponse),
        (status = 401, body = ErrorResponse),
    ),
    security(("cookieAuth" = []))
)]
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
        // モジュールが正常にコンパイルされることを確認
    }
}
