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

#[derive(Clone, Default)]
pub struct DividendCacheState {
    pub running: Arc<AtomicBool>,
}

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub secrets: Arc<Secrets>,
    pub client: Client,
    /// 配当キャッシュのバックグラウンド更新状態（多重起動防止）
    pub dividend_cache: DividendCacheState,
}
#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::test_env::{EnvGuard, ENV_MUTEX},
    };
    #[tokio::test]
    async fn test_secrets_from_env() {
        let _lock = ENV_MUTEX.lock().await;
        {
            let _db = EnvGuard::set("DATABASE_URL", None);
            let err_msg = Secrets::from_env().unwrap_err();
            assert!(
                err_msg.contains("DATABASE_URL"),
                "エラーメッセージに DATABASE_URL が含まれること: {err_msg}"
            );
        }
        {
            let _db = EnvGuard::set(
                "DATABASE_URL",
                Some("postgresql://user:password@localhost/test"),
            );
            let _fe = EnvGuard::set("FRONTEND_URL", None);
            assert_eq!(
                Secrets::from_env().unwrap().frontend_url,
                "http://localhost:8080"
            );
        }
        {
            let _db = EnvGuard::set(
                "DATABASE_URL",
                Some("postgresql://user:password@localhost/test"),
            );
            let _jq = EnvGuard::set("JQUANTS_API_KEY", None);
            assert!(Secrets::from_env().unwrap().jquants_api_key.is_none());
        }
    }
}
