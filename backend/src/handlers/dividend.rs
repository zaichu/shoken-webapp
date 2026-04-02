use crate::{
    errors::{ApiError, ErrorResponse},
    extractors::auth::AuthenticatedUser,
    models::common::MessageResponse,
    models::csv_import::{CsvPreviewResponse, CsvUploadForm, CsvUploadResponse},
    models::dividend::Dividend,
    services::csv_domain::DividendDomain,
    services::dividend as dividend_service,
    state::AppState,
};
use axum::{
    extract::Multipart,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};

pub fn dividend_routes() -> Router<AppState> {
    Router::new()
        .route("/dividends", get(list))
        .route("/dividends/csv", post(upload_csv))
        .route("/dividends/csv/preview", post(preview_csv))
        .route("/dividends/all", delete(delete_all))
}

/// 認証ユーザーの配当金一覧を取得
#[utoipa::path(
    get,
    path = "/dividends",
    operation_id = "dividend_list",
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
    let dividends = dividend_service::list(&state.pool, auth_user.id()).await?;
    Ok((StatusCode::OK, Json(dividends)))
}

/// CSV ファイルをパースして保存前プレビューを返す（DB 書き込みなし）
#[utoipa::path(
    post,
    path = "/dividends/csv/preview",
    operation_id = "dividend_preview_csv",
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
    crate::handlers::csv_import::handle_preview_csv::<DividendDomain>(multipart).await
}

/// CSV ファイルをアップロードして配当金を一括登録
#[utoipa::path(
    post,
    path = "/dividends/csv",
    operation_id = "dividend_upload_csv",
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
    crate::handlers::csv_import::handle_upload_csv::<DividendDomain>(
        &state.pool,
        auth_user.id(),
        multipart,
    )
    .await
}

/// 認証ユーザーの配当金を全削除
#[utoipa::path(
    delete,
    path = "/dividends/all",
    operation_id = "dividend_delete_all",
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
    dividend_service::delete_all(&state.pool, auth_user.id()).await?;
    Ok((
        StatusCode::OK,
        Json(MessageResponse {
            message: "全ての配当金データを削除しました".to_string(),
        }),
    ))
}
