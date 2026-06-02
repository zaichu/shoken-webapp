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
use axum::{extract::Multipart, extract::State, http::StatusCode, response::IntoResponse, Json};

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
    path = "/dividends",
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

#[cfg(test)]
mod tests {
    use {
        super::*,
        axum::{
            body::Body,
            http::{Request, StatusCode},
            routing::get,
            Router,
        },
        reqwest::Client,
        std::sync::Arc,
        tower::ServiceExt,
    };

    fn setup_test_app() -> Router<()> {
        let pool =
            crate::db::connect_pool_lazy("postgresql://postgres:postgres@localhost/postgres", 1)
                .unwrap();
        Router::new()
            .route("/dividends", get(list).delete(delete_all))
            .with_state(crate::AppState {
                pool,
                secrets: Arc::new(crate::state::Secrets {
                    database_url: "postgresql://postgres:postgres@localhost/postgres".to_string(),
                    jquants_api_key: None,
                    google_client_id: None,
                    google_client_secret: None,
                    frontend_url: "http://localhost:8080".to_string(),
                }),
                client: Client::new(),
                dividend_cache: crate::state::DividendCacheState::default(),
            })
    }

    #[tokio::test]
    async fn test_list_unauthorized() {
        let app = setup_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/dividends")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_delete_all_unauthorized() {
        let app = setup_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri("/dividends")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
