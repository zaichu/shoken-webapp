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
    use std::env;
    use tokio::sync::Mutex;

    // env var 操作テストを直列化するためのロック（config.rs と同じパターン）
    static ENV_MUTEX: Mutex<()> = Mutex::const_new(());

    /// env var を操作し、Drop 時に元の値へ自動復元するガード
    struct EnvGuard {
        key: &'static str,
        previous: Option<String>,
    }

    impl EnvGuard {
        fn set(key: &'static str, value: Option<&str>) -> Self {
            let previous = env::var(key).ok();
            match value {
                Some(v) => env::set_var(key, v),
                None => env::remove_var(key),
            }
            Self { key, previous }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            match &self.previous {
                Some(v) => env::set_var(self.key, v),
                None => env::remove_var(self.key),
            }
        }
    }

    #[tokio::test]
    async fn test_secrets_from_env_error_without_database_url() {
        let _lock = ENV_MUTEX.lock().await;
        let _db = EnvGuard::set("DATABASE_URL", None);

        let result = Secrets::from_env();

        assert!(result.is_err());
        let err_msg = result.unwrap_err();
        assert!(
            err_msg.contains("DATABASE_URL"),
            "エラーメッセージに DATABASE_URL が含まれること: {err_msg}"
        );
    }

    #[tokio::test]
    async fn test_secrets_from_env_frontend_url_default() {
        // FRONTEND_URL 未設定時はデフォルト値 "http://localhost:8080" を使用する
        let _lock = ENV_MUTEX.lock().await;
        let _db = EnvGuard::set(
            "DATABASE_URL",
            Some("postgresql://user:password@localhost/test"),
        );
        let _fe = EnvGuard::set("FRONTEND_URL", None);

        let result = Secrets::from_env();

        assert!(result.is_ok());
        assert_eq!(result.unwrap().frontend_url, "http://localhost:8080");
    }

    #[tokio::test]
    async fn test_secrets_from_env_jquants_key_is_optional() {
        // JQUANTS_API_KEY は省略可能で None になる
        let _lock = ENV_MUTEX.lock().await;
        let _db = EnvGuard::set(
            "DATABASE_URL",
            Some("postgresql://user:password@localhost/test"),
        );
        let _jq = EnvGuard::set("JQUANTS_API_KEY", None);

        let result = Secrets::from_env();

        assert!(result.is_ok());
        assert!(result.unwrap().jquants_api_key.is_none());
    }
}
