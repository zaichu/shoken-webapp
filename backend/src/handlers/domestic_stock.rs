use crate::{
    errors::{ApiError, ErrorResponse},
    extractors::auth::AuthenticatedUser,
    models::common::MessageResponse,
    models::csv_import::{CsvPreviewResponse, CsvUploadForm, CsvUploadResponse},
    models::domestic_stock::DomesticStock,
    services::csv_domain::DomesticStockDomain,
    services::domestic_stock as domestic_stock_service,
    state::AppState,
};
use axum::{
    extract::Multipart,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};

pub fn domestic_stock_routes() -> Router<AppState> {
    Router::new()
        .route("/domestic-stocks", get(list).delete(delete_all))
        .route("/domestic-stocks/csv/preview", post(preview_csv))
}

pub fn domestic_stock_csv_upload_routes() -> Router<AppState> {
    Router::new().route("/domestic-stocks/csv", post(upload_csv))
}

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
    crate::handlers::csv_import::handle_preview_csv::<DomesticStockDomain>(multipart).await
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
    crate::handlers::csv_import::handle_upload_csv::<DomesticStockDomain>(
        &state.pool,
        auth_user.id(),
        multipart,
    )
    .await
}

/// 認証ユーザーの国内株式取引を全削除
#[utoipa::path(
    delete,
    path = "/domestic-stocks",
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
        Json(MessageResponse {
            message: "全ての国内株式取引データを削除しました".to_string(),
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
        },
        reqwest::Client,
        std::sync::Arc,
        tower::ServiceExt,
    };

    fn setup_test_app() -> Router<()> {
        let pool =
            crate::db::connect_pool_lazy("postgresql://postgres:postgres@localhost/postgres", 1)
                .unwrap();
        domestic_stock_routes().with_state(crate::AppState {
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
                    .uri("/domestic-stocks")
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
                    .uri("/domestic-stocks")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
