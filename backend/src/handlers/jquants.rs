use crate::errors::{ApiError, ErrorResponse};
use crate::extractors::auth::AuthenticatedUser;
use crate::models::jquants::{FinSummaryQuery, FinSummaryResponse};
use crate::services::jquants::{JQuantsService, FIN_SUMMARY_URL};
use crate::state::AppState;
use axum::{
    extract::{Query, State},
    response::Json,
};

/// 決算サマリーを取得（J-Quants API V2）
#[utoipa::path(
    get,
    path = "/jquants/fins/summary",
    operation_id = "jquants_fin_summary",
    params(
        ("code" = String, Query, description = "銘柄コード"),
        ("from" = Option<String>, Query, description = "開始日（YYYY-MM-DD）"),
        ("to" = Option<String>, Query, description = "終了日（YYYY-MM-DD）"),
    ),
    responses(
        (status = 200, body = FinSummaryResponse),
        (status = 400, body = ErrorResponse),
        (status = 401, body = ErrorResponse),
        (status = 429, body = ErrorResponse),
        (status = 502, body = ErrorResponse),
    ),
    security(("cookieAuth" = []))
)]
pub async fn get_fin_summary(
    State(state): State<AppState>,
    _auth_user: AuthenticatedUser,
    Query(params): Query<FinSummaryQuery>,
) -> Result<Json<FinSummaryResponse>, ApiError> {
    tracing::info!("決算サマリー取得パラメータ: {:?}", params);

    let api_key =
        state.secrets.jquants_api_key.as_ref().ok_or_else(|| {
            ApiError::ApiError("JQUANTS_API_KEY が設定されていません".to_string())
        })?;

    let response =
        JQuantsService::get_fin_summary(&state.client, params, api_key, FIN_SUMMARY_URL).await?;
    Ok(Json(response))
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
            .route("/jquants/fins/summary", get(get_fin_summary))
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
    async fn test_get_fin_summary_unauthorized() {
        let app = setup_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/jquants/fins/summary?code=1234")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
