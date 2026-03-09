use crate::{
    errors::{ApiError, ErrorResponse},
    extractors::auth::AuthenticatedUser,
    models::common::MessageResponse,
    models::csv_import::{CsvPreviewResponse, CsvUploadForm, CsvUploadResponse},
    models::domestic_stock::DomesticStock,
    services::domestic_stock as domestic_stock_service,
    state::AppState,
};
use axum::{extract::Multipart, extract::State, http::StatusCode, response::IntoResponse, Json};

/// 認証ユーザーの国内株式取引一覧を取得
#[utoipa::path(
    get,
    path = "/domestic-stocks",
    operation_id = "domestic_stock_list",
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

/// CSV ファイルをパースして保存前プレビューを返す（DB 書き込みなし）
#[utoipa::path(
    post,
    path = "/domestic-stocks/csv/preview",
    operation_id = "domestic_stock_preview_csv",
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
    let response = domestic_stock_service::preview_csv(&bytes)?;
    Ok((StatusCode::OK, Json(response)))
}

/// CSV ファイルをアップロードして国内株式取引を一括登録
#[utoipa::path(
    post,
    path = "/domestic-stocks/csv",
    operation_id = "domestic_stock_upload_csv",
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
    let response = domestic_stock_service::upload_csv(&state.pool, auth_user.id(), &bytes).await?;
    Ok((StatusCode::CREATED, Json(response)))
}

/// 認証ユーザーの国内株式取引を全削除
#[utoipa::path(
    delete,
    path = "/domestic-stocks/all",
    operation_id = "domestic_stock_delete_all",
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
    domestic_stock_service::delete_all(&state.pool, auth_user.id()).await?;
    Ok((
        StatusCode::OK,
        Json(serde_json::json!({"message": "全ての国内株式取引データを削除しました"})),
    ))
}
