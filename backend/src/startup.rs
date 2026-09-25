use crate::config::Config;
use crate::db::connect_pool_lazy;
use crate::services;
use crate::state::{AppState, DividendCacheState, Secrets};
use reqwest::Client;
use sqlx::PgPool;
use std::sync::Arc;

pub fn build_startup_state(
    secrets: Arc<Secrets>,
    config: &Config,
) -> Result<(PgPool, AppState), String> {
    // URL 検証のみ行い、実接続は background startup task で行う。
    let pool = connect_pool_lazy(&secrets.database_url, config.database_max_connections)?;
    // OAuthトークン交換用のHTTPクライアントを起動時に1つ構築し、以降使い回す。
    services::auth::init_oauth_http_client().map_err(|e| e.to_string())?;
    let state = AppState {
        pool: pool.clone(),
        secrets,
        client: Client::new(),
        dividend_cache: DividendCacheState::default(),
    };

    Ok((pool, state))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_secrets() -> Arc<Secrets> {
        Arc::new(Secrets {
            database_url: "postgresql://user:password@localhost/test_db".to_string(),
            jquants_api_key: None,
            google_client_id: Some("client-id".to_string()),
            google_client_secret: Some("client-secret".to_string()),
            frontend_url: "http://localhost:8080".to_string(),
        })
    }

    #[tokio::test]
    async fn build_startup_state_reuses_pool_for_runtime_and_startup_tasks() {
        let config = Config::default();
        let (startup_pool, state) =
            build_startup_state(test_secrets(), &config).expect("startup state");

        startup_pool.close().await;

        assert!(
            state.pool.is_closed(),
            "runtime state pool should share the startup pool handle"
        );
    }
}
