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

    #[test]
    fn test_secrets_fields() {
        // Secrets の各フィールドに値が正しく格納されることを確認
        let secrets = Secrets {
            database_url: "postgresql://user:password@localhost/test_db".to_string(),
            jquants_api_key: Some("test_api_key".to_string()),
            google_client_id: Some("client_id".to_string()),
            google_client_secret: Some("client_secret".to_string()),
            frontend_url: "http://localhost:8080".to_string(),
        };
        assert_eq!(
            secrets.database_url,
            "postgresql://user:password@localhost/test_db"
        );
        assert_eq!(secrets.jquants_api_key.as_deref(), Some("test_api_key"));
        assert_eq!(secrets.google_client_id.as_deref(), Some("client_id"));
        assert_eq!(
            secrets.google_client_secret.as_deref(),
            Some("client_secret")
        );
        assert_eq!(secrets.frontend_url, "http://localhost:8080");
    }

    #[test]
    fn test_secrets_optional_fields_can_be_none() {
        // jquants_api_key, google_client_id, google_client_secret は省略可能
        let secrets = Secrets {
            database_url: "postgresql://localhost/db".to_string(),
            jquants_api_key: None,
            google_client_id: None,
            google_client_secret: None,
            frontend_url: "http://localhost:8080".to_string(),
        };
        assert!(secrets.jquants_api_key.is_none());
        assert!(secrets.google_client_id.is_none());
        assert!(secrets.google_client_secret.is_none());
    }
}
