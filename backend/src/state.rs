use reqwest::Client;
use sqlx::PgPool;
use std::sync::Arc;

/// 環境変数から取得するシークレット情報
#[derive(Clone, Debug)]
pub struct Secrets {
    pub database_url: String,
    pub jquants_email: Option<String>,
    pub jquants_password: Option<String>,
    pub google_client_id: Option<String>,
    pub google_client_secret: Option<String>,
    pub frontend_url: String,
    pub session_secret: String,
}

impl Secrets {
    /// 環境変数からシークレット情報を読み込む
    pub fn from_env() -> Result<Self, String> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL")
                .map_err(|_| "DATABASE_URL が設定されていません".to_string())?,
            jquants_email: std::env::var("JQUANTS_EMAIL").ok(),
            jquants_password: std::env::var("JQUANTS_PASSWORD").ok(),
            google_client_id: std::env::var("GOOGLE_CLIENT_ID").ok(),
            google_client_secret: std::env::var("GOOGLE_CLIENT_SECRET").ok(),
            frontend_url: std::env::var("FRONTEND_URL")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
            session_secret: std::env::var("SESSION_SECRET")
                .unwrap_or_else(|_| "dev-secret-key-change-in-production-12345678".to_string()),
        })
    }

    /// 指定したキーのシークレットを取得（後方互換性のため）
    pub fn get(&self, key: &str) -> Option<String> {
        match key {
            "DATABASE_URL" => Some(self.database_url.clone()),
            "JQUANTS_EMAIL" => self.jquants_email.clone(),
            "JQUANTS_PASSWORD" => self.jquants_password.clone(),
            "GOOGLE_CLIENT_ID" => self.google_client_id.clone(),
            "GOOGLE_CLIENT_SECRET" => self.google_client_secret.clone(),
            "FRONTEND_URL" => Some(self.frontend_url.clone()),
            "SESSION_SECRET" => Some(self.session_secret.clone()),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub secrets: Arc<Secrets>,
    pub client: Client,
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::postgres::PgPoolOptions;

    fn create_test_secrets() -> Secrets {
        Secrets {
            database_url: "postgresql://user:password@localhost/test_db".to_string(),
            jquants_email: Some("test@example.com".to_string()),
            jquants_password: Some("password123".to_string()),
            google_client_id: None,
            google_client_secret: None,
            frontend_url: "http://localhost:8080".to_string(),
            session_secret: "test-secret-key".to_string(),
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
        };

        let cloned_state = original_state.clone();

        let _ = &cloned_state.pool;
        let _ = &cloned_state.secrets;
        let _ = &cloned_state.client;
    }

    #[test]
    fn test_secrets_get() {
        let secrets = create_test_secrets();

        assert_eq!(
            secrets.get("DATABASE_URL"),
            Some("postgresql://user:password@localhost/test_db".to_string())
        );
        assert_eq!(
            secrets.get("JQUANTS_EMAIL"),
            Some("test@example.com".to_string())
        );
        assert_eq!(secrets.get("NONEXISTENT_KEY"), None);
    }
}
