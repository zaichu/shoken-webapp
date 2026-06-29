use crate::{
    errors::{ApiError, ErrorResponse},
    extractors::{auth::AuthenticatedUser, validated_json::ValidatedJson},
    models::{
        asset_balance::{AssetBalance, BulkCreateAssetBalanceRequest},
        common::{BulkCreateResponse, MessageResponse},
        csv_import::{CsvPreviewResponse, CsvUploadForm, CsvUploadResponse},
    },
    services::{asset_balance as asset_balance_service, csv_domain::AssetBalanceDomain},
    state::AppState,
};
use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};

// 保有銘柄ハンドラー
// ---------------------------------------------------------------------------

/// 保有銘柄一覧を取得（v1）
#[utoipa::path(
    get,
    path = "/api/v1/asset-balances",
    operation_id = "v1_asset_balance_list",
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

/// 保有銘柄を全置換（v1）
#[utoipa::path(
    put,
    path = "/api/v1/asset-balances",
    operation_id = "v1_asset_balance_replace",
    request_body = BulkCreateAssetBalanceRequest,
    responses(
        (status = 200, body = BulkCreateResponse),
        (status = 400, body = ErrorResponse),
        (status = 401, body = ErrorResponse),
    ),
    security(("cookieAuth" = []))
)]
pub async fn replace(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    ValidatedJson(data): ValidatedJson<BulkCreateAssetBalanceRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let response =
        asset_balance_service::bulk_create(&state.pool, auth_user.id(), &data.items).await?;
    Ok((StatusCode::OK, Json(response)))
}

/// 保有銘柄を全削除（v1）
#[utoipa::path(
    delete,
    path = "/api/v1/asset-balances",
    operation_id = "v1_asset_balance_delete_all",
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
        Json(MessageResponse {
            message: "全ての保有銘柄データを削除しました".to_string(),
        }),
    ))
}

/// 保有銘柄 CSV をバリデーション（v1）
#[utoipa::path(
    post,
    path = "/api/v1/asset-balance-import-validations",
    operation_id = "v1_asset_balance_validate_import",
    request_body(content = CsvUploadForm, content_type = "multipart/form-data"),
    responses(
        (status = 200, body = CsvPreviewResponse),
        (status = 400, body = ErrorResponse),
        (status = 401, body = ErrorResponse),
    ),
    security(("cookieAuth" = []))
)]
pub async fn validate_import(
    _auth_user: AuthenticatedUser,
    multipart: Multipart,
) -> Result<impl IntoResponse, ApiError> {
    crate::handlers::csv_import::handle_preview_csv::<AssetBalanceDomain>(multipart).await
}

/// 保有銘柄 CSV をインポート（v1）
#[utoipa::path(
    post,
    path = "/api/v1/asset-balance-imports",
    operation_id = "v1_asset_balance_import",
    request_body(content = CsvUploadForm, content_type = "multipart/form-data"),
    responses(
        (status = 201, body = CsvUploadResponse),
        (status = 400, body = ErrorResponse),
        (status = 401, body = ErrorResponse),
    ),
    security(("cookieAuth" = []))
)]
pub async fn import(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    multipart: Multipart,
) -> Result<impl IntoResponse, ApiError> {
    let json = crate::handlers::csv_import::handle_upload_csv::<AssetBalanceDomain>(
        &state.pool,
        auth_user.id(),
        multipart,
    )
    .await?;
    Ok((StatusCode::CREATED, json))
}

// ---------------------------------------------------------------------------
