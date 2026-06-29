use crate::{
    errors::ApiError,
    extractors::auth::AuthenticatedUser,
    extractors::validated_json::ValidatedJson,
    models::dividend_cache::{DividendPerShareBatchRequest, DividendPerShareBatchResponse},
    services::dividend_cache as dividend_cache_service,
    AppState,
};
use axum::{extract::State, response::IntoResponse, Json};

pub async fn batch(
    State(state): State<AppState>,
    _auth_user: AuthenticatedUser,
    ValidatedJson(data): ValidatedJson<DividendPerShareBatchRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let api_key = state.secrets.jquants_api_key.as_deref();

    let items = dividend_cache_service::get_batch(
        &state.pool,
        &state.client,
        api_key,
        &data.security_codes,
        &state.dividend_cache.running,
    )
    .await?;

    Ok(Json(DividendPerShareBatchResponse { items }))
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        axum::{
            body::Body,
            http::{Request, StatusCode},
            routing::post,
            Router,
        },
        reqwest::Client,
        serde_json::json,
        std::sync::Arc,
        tower::ServiceExt,
    };

    fn setup_test_app() -> Router<()> {
        let pool =
            crate::db::connect_pool_lazy("postgresql://postgres:postgres@localhost/postgres", 1)
                .unwrap();
        Router::new()
            .route("/dividends/per-share/batch", post(batch))
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
    async fn test_batch_unauthorized() {
        let app = setup_test_app();
        let body = json!({ "security_codes": ["1234"] }).to_string();
        let request = Request::builder()
            .method("POST")
            .uri("/dividends/per-share/batch")
            .header("content-type", "application/json")
            .body(Body::from(body))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
