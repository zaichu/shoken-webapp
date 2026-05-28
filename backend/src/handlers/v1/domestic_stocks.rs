use crate::{
    errors::{ApiError, ErrorResponse},
    extractors::auth::AuthenticatedUser,
    models::{
        common::MessageResponse,
        csv_import::{CsvPreviewResponse, CsvUploadForm, CsvUploadResponse},
        domestic_stock::DomesticStock,
    },
    state::AppState,
};
use axum::{
    extract::{Multipart, State},
    response::IntoResponse,
};

// Domestic stock transaction handlers
// ---------------------------------------------------------------------------

/// 国内株式取引一覧を取得（v1）
#[utoipa::path(
    get,
    path = "/api/v1/domestic-stock-transactions",
    operation_id = "v1_domestic_stock_transaction_list",
    responses(
        (status = 200, body = Vec<DomesticStock>),
        (status = 401, body = ErrorResponse),
    ),
    security(("cookieAuth" = []))
)]
pub async fn list_transactions(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<impl IntoResponse, ApiError> {
    crate::handlers::domestic_stock::list(State(state), auth_user).await
}

/// 国内株式取引を全削除（v1）
#[utoipa::path(
    delete,
    path = "/api/v1/domestic-stock-transactions",
    operation_id = "v1_domestic_stock_transaction_delete_all",
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
    crate::handlers::domestic_stock::delete_all(State(state), auth_user).await
}

/// 国内株式取引 CSV をバリデーション（v1）
#[utoipa::path(
    post,
    path = "/api/v1/domestic-stock-import-validations",
    operation_id = "v1_domestic_stock_validate_import",
    request_body(content = CsvUploadForm, content_type = "multipart/form-data"),
    responses(
        (status = 200, body = CsvPreviewResponse),
        (status = 400, body = ErrorResponse),
        (status = 401, body = ErrorResponse),
    ),
    security(("cookieAuth" = []))
)]
pub async fn validate_import(
    auth_user: AuthenticatedUser,
    multipart: Multipart,
) -> Result<impl IntoResponse, ApiError> {
    crate::handlers::domestic_stock::preview_csv(auth_user, multipart).await
}

/// 国内株式取引 CSV をインポート（v1）
#[utoipa::path(
    post,
    path = "/api/v1/domestic-stock-imports",
    operation_id = "v1_domestic_stock_import",
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
    crate::handlers::domestic_stock::upload_csv(State(state), auth_user, multipart).await
}

// ---------------------------------------------------------------------------
