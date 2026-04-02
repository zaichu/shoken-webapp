use crate::{
    errors::{ApiError, ErrorResponse},
    extractors::auth::AuthenticatedUser,
    models::common::MessageResponse,
    models::csv_import::{CsvPreviewResponse, CsvUploadForm, CsvUploadResponse},
    models::mutualfund::Mutualfund,
    services::csv_domain::MutualfundDomain,
    services::mutualfund as mutualfund_service,
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

pub fn mutualfund_routes() -> Router<AppState> {
    Router::new()
        .route("/mutualfunds", get(list))
        .route("/mutualfunds/csv", post(upload_csv))
        .route("/mutualfunds/csv/preview", post(preview_csv))
        .route("/mutualfunds/all", delete(delete_all))
}

/// 認証ユーザーの投資信託一覧を取得
#[utoipa::path(
    get,
    path = "/mutualfunds",
    operation_id = "mutualfund_list",
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

/// CSV ファイルをパースして保存前プレビューを返す（DB 書き込みなし）
#[utoipa::path(
    post,
    path = "/mutualfunds/csv/preview",
    operation_id = "mutualfund_preview_csv",
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
    crate::handlers::csv_import::handle_preview_csv::<MutualfundDomain>(multipart).await
}

/// CSV ファイルをアップロードして投資信託を一括登録
#[utoipa::path(
    post,
    path = "/mutualfunds/csv",
    operation_id = "mutualfund_upload_csv",
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
    crate::handlers::csv_import::handle_upload_csv::<MutualfundDomain>(
        &state.pool,
        auth_user.id(),
        multipart,
    )
    .await
}

/// 認証ユーザーの投資信託を全削除
#[utoipa::path(
    delete,
    path = "/mutualfunds/all",
    operation_id = "mutualfund_delete_all",
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
    mutualfund_service::delete_all(&state.pool, auth_user.id()).await?;
    Ok((
        StatusCode::OK,
        Json(MessageResponse {
            message: "全ての投資信託データを削除しました".to_string(),
        }),
    ))
}
