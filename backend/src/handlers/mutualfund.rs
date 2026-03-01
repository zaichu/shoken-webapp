use crate::{
    errors::{ApiError, ErrorResponse},
    extractors::auth::AuthenticatedUser,
    models::common::MessageResponse,
    models::csv_import::{CsvUploadForm, CsvUploadResponse},
    models::mutualfund::Mutualfund,
    services::mutualfund as mutualfund_service,
    state::AppState,
};
use axum::{extract::Multipart, extract::State, http::StatusCode, response::IntoResponse, Json};

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
    mut multipart: Multipart,
) -> Result<impl IntoResponse, ApiError> {
    let mut file_bytes: Option<Vec<u8>> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        ApiError::ValidationError(format!("マルチパートの読み込みに失敗しました: {}", e))
    })? {
        if field.name() == Some("file") {
            let bytes = field.bytes().await.map_err(|e| {
                ApiError::ValidationError(format!("ファイルの読み込みに失敗しました: {}", e))
            })?;
            file_bytes = Some(bytes.to_vec());
            break;
        }
    }

    let bytes = file_bytes
        .ok_or_else(|| ApiError::ValidationError("fileフィールドが見つかりません".to_string()))?;

    let response = mutualfund_service::upload_csv(&state.pool, auth_user.id(), &bytes).await?;
    Ok((StatusCode::CREATED, Json(response)))
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
