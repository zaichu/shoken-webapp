use crate::{
    errors::{ApiError, ErrorResponse},
    extractors::auth::AuthenticatedUser,
    extractors::validated_json::ValidatedJson,
    models::asset_balance::{AssetBalance, BulkCreateAssetBalanceRequest},
    models::common::{BulkCreateResponse, MessageResponse},
    models::csv_import::{CsvPreviewResponse, CsvUploadForm, CsvUploadResponse},
    services::asset_balance as asset_balance_service,
    services::csv_domain::AssetBalanceDomain,
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

pub fn asset_balance_routes() -> Router<AppState> {
    Router::new()
        .route("/asset-balances", get(list).delete(delete_all))
        .route("/asset-balances/bulk", post(bulk_create))
        .route("/asset-balances/csv/preview", post(preview_csv))
}

pub fn asset_balance_csv_upload_routes() -> Router<AppState> {
    Router::new().route("/asset-balances/csv", post(upload_csv))
}

/// 認証ユーザーの保有銘柄一覧を取得
#[utoipa::path(
    get,
    path = "/asset-balances",
    operation_id = "asset_balance_list",
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

/// 保有銘柄を一括追加（既存は更新）
#[utoipa::path(
    post,
    path = "/asset-balances/bulk",
    operation_id = "asset_balance_bulk_create",
    request_body = BulkCreateAssetBalanceRequest,
    responses(
        (status = 201, body = BulkCreateResponse),
        (status = 400, body = ErrorResponse),
        (status = 401, body = ErrorResponse),
    ),
    security(("cookieAuth" = []))
)]
pub async fn bulk_create(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    ValidatedJson(data): ValidatedJson<BulkCreateAssetBalanceRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let response =
        asset_balance_service::bulk_create(&state.pool, auth_user.id(), &data.items).await?;
    Ok((StatusCode::CREATED, Json(response)))
}

/// CSV ファイルをパースして保存前プレビューを返す（DB 書き込みなし）
#[utoipa::path(
    post,
    path = "/asset-balances/csv/preview",
    operation_id = "asset_balance_preview_csv",
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
    crate::handlers::csv_import::handle_preview_csv::<AssetBalanceDomain>(multipart).await
}

/// CSV ファイルをアップロードして保有銘柄を一括登録（既存データ全置換）
#[utoipa::path(
    post,
    path = "/asset-balances/csv",
    operation_id = "asset_balance_upload_csv",
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
    crate::handlers::csv_import::handle_upload_csv::<AssetBalanceDomain>(
        &state.pool,
        auth_user.id(),
        multipart,
    )
    .await
}

/// 認証ユーザーの保有銘柄を全削除
#[utoipa::path(
    delete,
    path = "/asset-balances",
    operation_id = "asset_balance_delete_all",
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
        asset_balance_routes().with_state(crate::AppState {
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
                    .uri("/asset-balances")
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
                    .uri("/asset-balances")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
