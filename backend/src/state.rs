use reqwest::Client;
use sqlx::PgPool;
use std::sync::{atomic::AtomicBool, Arc};

/// 環境変数から取得するシークレット情報
#[derive(Clone, Debug)]
pub struct Secrets {
    pub database_url: String,
    pub jquants_api_key: Option<String>,
    pub google_client_id: Option<String>,
    pub google_client_secret: Option<String>,
    pub frontend_url: String,
}

impl Secrets {
    /// 環境変数からシークレット情報を読み込む
    pub fn from_env() -> Result<Self, String> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL").map_err(|_| {
                "DATABASE_URL が設定されていません（backend/.env を確認してください）".to_string()
            })?,
            jquants_api_key: std::env::var("JQUANTS_API_KEY").ok(),
            google_client_id: std::env::var("GOOGLE_CLIENT_ID").ok(),
            google_client_secret: std::env::var("GOOGLE_CLIENT_SECRET").ok(),
            frontend_url: std::env::var("FRONTEND_URL")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
        })
    }
}

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub secrets: Arc<Secrets>,
    pub client: Client,
    /// 配当キャッシュのバックグラウンド更新タスクが実行中かどうか（多重起動防止）
    pub background_task_running: Arc<AtomicBool>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::postgres::PgPoolOptions;

    fn create_test_secrets() -> Secrets {
        Secrets {
            database_url: "postgresql://user:password@localhost/test_db".to_string(),
            jquants_api_key: Some("test_api_key".to_string()),
            google_client_id: None,
            google_client_secret: None,
            frontend_url: "http://localhost:8080".to_string(),
        }
    }

    #[tokio::test]
    async fn test_app_state_creation() {
        let database_url = "postgresql://user:password@localhost/test_db";

        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect_lazy(database_url)
            .expect("Failed to create connection pool");

        let secrets = Arc::new(create_test_secrets());
        let client = Client::new();

        let app_state = AppState {
            pool: pool.clone(),
            secrets: secrets.clone(),
            client: client.clone(),
            background_task_running: Arc::new(AtomicBool::new(false)),
        };

        // AppStateが正常に作成されることを確認
        let _ = &app_state.pool;
        let _ = &app_state.secrets;
        let _ = &app_state.client;

        // AppStateのクローンが正常に動作することを確認
        let cloned_state = app_state.clone();
        let _ = &cloned_state.pool;
        let _ = &cloned_state.secrets;
        let _ = &cloned_state.client;
    }

    #[tokio::test]
    async fn test_app_state_clone() {
        let database_url = "postgresql://user:password@localhost/test_db";
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect_lazy(database_url)
            .expect("Failed to create connection pool");

        let secrets = Arc::new(create_test_secrets());
        let client = Client::new();

        let original_state = AppState {
            pool,
            secrets,
            client,
            background_task_running: Arc::new(AtomicBool::new(false)),
        };

        let cloned_state = original_state.clone();

        let _ = &cloned_state.pool;
        let _ = &cloned_state.secrets;
        let _ = &cloned_state.client;
    }
}
