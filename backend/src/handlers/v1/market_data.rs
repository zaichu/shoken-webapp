use crate::{
    errors::{ApiError, ErrorResponse},
    extractors::auth::AuthenticatedUser,
    models::market_data::financial_statement::{
        FinancialStatementsQuery, FinancialStatementsResponse,
    },
    services::market_data::providers::jquants::JQuantsClient,
    state::AppState,
};
use axum::{
    extract::{Query, State},
    Json,
};

// 財務データハンドラー
// ---------------------------------------------------------------------------

/// 決算サマリーを取得（v1）
#[utoipa::path(
    get,
    path = "/api/v1/financial-statements",
    operation_id = "v1_get_financial_statements",
    params(
        ("code" = String, Query, description = "銘柄コード"),
        ("from" = Option<String>, Query, description = "開始日（YYYY-MM-DD）"),
        ("to" = Option<String>, Query, description = "終了日（YYYY-MM-DD）"),
    ),
    responses(
        (status = 200, body = FinancialStatementsResponse),
        (status = 400, body = ErrorResponse),
        (status = 401, body = ErrorResponse),
        (status = 429, body = ErrorResponse),
        (status = 502, body = ErrorResponse),
    ),
    security(("cookieAuth" = []))
)]
pub async fn get_financial_statements(
    State(state): State<AppState>,
    _auth_user: AuthenticatedUser,
    Query(params): Query<FinancialStatementsQuery>,
) -> Result<Json<FinancialStatementsResponse>, ApiError> {
    tracing::info!("決算サマリー取得パラメータ: {:?}", params);

    let jquants_client = JQuantsClient::from_optional_api_key(
        state.client.clone(),
        state.secrets.jquants_api_key.as_deref(),
    )?;
    let response = jquants_client.get_fin_summary(params).await?;
    Ok(Json(response))
}

// ---------------------------------------------------------------------------

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
            .route(
                "/api/v1/financial-statements",
                get(get_financial_statements),
            )
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
    async fn test_get_financial_statements_unauthorized() {
        let app = setup_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/v1/financial-statements?code=1234")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
