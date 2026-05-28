use crate::{
    errors::{ApiError, ErrorResponse},
    extractors::{auth::AuthenticatedUser, validated_json::ValidatedJson},
    models::{
        common::MessageResponse,
        csv_import::{CsvPreviewResponse, CsvUploadForm, CsvUploadResponse},
        dividend::Dividend,
        dividend_cache::{DividendPerShareBatchRequest, DividendPerShareBatchResponse},
    },
    state::AppState,
};
use axum::{
    extract::{Multipart, State},
    response::IntoResponse,
};

// Dividend handlers
// ---------------------------------------------------------------------------

/// 配当金一覧を取得（v1）
#[utoipa::path(
    get,
    path = "/api/v1/dividends",
    operation_id = "v1_dividend_list",
    responses(
        (status = 200, body = Vec<Dividend>),
        (status = 401, body = ErrorResponse),
    ),
    security(("cookieAuth" = []))
)]
pub async fn list(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<impl IntoResponse, ApiError> {
    crate::handlers::dividend::list(State(state), auth_user).await
}

/// 配当金を全削除（v1）
#[utoipa::path(
    delete,
    path = "/api/v1/dividends",
    operation_id = "v1_dividend_delete_all",
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
    crate::handlers::dividend::delete_all(State(state), auth_user).await
}

/// 配当金 CSV をバリデーション（DB 書き込みなし）（v1）
#[utoipa::path(
    post,
    path = "/api/v1/dividend-import-validations",
    operation_id = "v1_dividend_validate_import",
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
    crate::handlers::dividend::preview_csv(auth_user, multipart).await
}

/// 配当金 CSV をインポート（v1）
#[utoipa::path(
    post,
    path = "/api/v1/dividend-imports",
    operation_id = "v1_dividend_import",
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
    crate::handlers::dividend::upload_csv(State(state), auth_user, multipart).await
}

/// 配当利回りを一括取得（v1）
#[utoipa::path(
    post,
    path = "/api/v1/dividend-per-share-estimates",
    operation_id = "v1_dividend_per_share_estimate",
    request_body = DividendPerShareBatchRequest,
    responses(
        (status = 200, body = DividendPerShareBatchResponse),
        (status = 400, body = ErrorResponse),
        (status = 401, body = ErrorResponse),
    ),
    security(("cookieAuth" = []))
)]
pub async fn estimate_per_share(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    ValidatedJson(data): ValidatedJson<DividendPerShareBatchRequest>,
) -> Result<impl IntoResponse, ApiError> {
    crate::handlers::dividend_per_share::batch(State(state), auth_user, ValidatedJson(data)).await
}

// ---------------------------------------------------------------------------
