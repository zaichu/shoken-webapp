use crate::{
    errors::{ApiError, ErrorResponse},
    extractors::auth::AuthenticatedUser,
    handlers::common::ok_message,
    models::{
        common::{MessageResponse, PaginatedResponse, PaginationParams},
        csv_import::{CsvPreviewResponse, CsvUploadForm, CsvUploadResponse},
        mutualfund::Mutualfund,
    },
    services::{csv_domain::MutualfundDomain, mutualfund as mutualfund_service},
    state::AppState,
};
use axum::{
    extract::{Multipart, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};

// 投資信託取引ハンドラー
// ---------------------------------------------------------------------------

/// 投資信託一覧を取得（v1）
#[utoipa::path(
    get,
    path = "/api/v1/mutual-fund-transactions",
    operation_id = "v1_mutual_fund_transaction_list",
    params(
        ("page" = Option<i64>, Query, description = "ページ番号（デフォルト: 1）"),
        ("per_page" = Option<i64>, Query, description = "1ページあたりの件数（デフォルト: 200、最大: 1000）"),
    ),
    responses(
        (status = 200, body = PaginatedResponse<Mutualfund>),
        (status = 401, body = ErrorResponse),
    ),
    security(("cookieAuth" = []))
)]
pub async fn list_transactions(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, ApiError> {
    let result = mutualfund_service::list(&state.pool, auth_user.id(), &params).await?;
    Ok((StatusCode::OK, Json(result)))
}

/// 投資信託を全削除（v1）
#[utoipa::path(
    delete,
    path = "/api/v1/mutual-fund-transactions",
    operation_id = "v1_mutual_fund_transaction_delete_all",
    responses(
        (status = 200, body = MessageResponse),
        (status = 401, body = ErrorResponse),
    ),
    security(("cookieAuth" = []))
)]
pub async fn delete_transactions(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<impl IntoResponse, ApiError> {
    mutualfund_service::delete_all(&state.pool, auth_user.id()).await?;
    Ok(ok_message("全ての投資信託データを削除しました"))
}

/// 投資信託 CSV をバリデーション（v1）
#[utoipa::path(
    post,
    path = "/api/v1/mutual-fund-import-validations",
    operation_id = "v1_mutual_fund_validate_import",
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
    crate::handlers::csv_import::handle_preview_csv::<MutualfundDomain>(multipart).await
}

/// 投資信託 CSV をインポート（v1）
#[utoipa::path(
    post,
    path = "/api/v1/mutual-fund-imports",
    operation_id = "v1_mutual_fund_import",
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
    let json = crate::handlers::csv_import::handle_upload_csv::<MutualfundDomain>(
        &state.pool,
        auth_user.id(),
        multipart,
    )
    .await?;
    Ok((StatusCode::CREATED, json))
}

// ---------------------------------------------------------------------------
