#[cfg(not(target_arch = "wasm32"))]
use reqwest::Client;
use std::sync::{Arc, atomic::AtomicBool};

use crate::config::Config;
use crate::db::Db;
use crate::services::auth::GoogleOAuthClient;

/// 環境変数から取得するシークレット情報
#[derive(Clone, Debug)]
pub struct Secrets {
    /// DB 接続情報。Workers 側は Hyperdrive バインディングが保持するため wasm では持たない
    #[cfg(not(target_arch = "wasm32"))]
    pub database_url: String,
    pub jquants_api_key: Option<String>,
    pub google_client_id: Option<String>,
    pub google_client_secret: Option<String>,
    pub frontend_url: String,
}

impl Secrets {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn from_env() -> Result<Self, String> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL").map_err(|_| {
                "DATABASE_URL が設定されていません（backend/.env を確認してください）".to_string()
            })?,
            jquants_api_key: std::env::var("JQUANTS_API_KEY").ok(),
            google_client_id: Some(std::env::var("GOOGLE_CLIENT_ID").map_err(|_| {
                "GOOGLE_CLIENT_ID が設定されていません（Google OAuth 設定を確認してください）"
                    .to_string()
            })?),
            google_client_secret: Some(std::env::var("GOOGLE_CLIENT_SECRET").map_err(|_| {
                "GOOGLE_CLIENT_SECRET が設定されていません（Google OAuth 設定を確認してください）"
                    .to_string()
            })?),
            frontend_url: std::env::var("FRONTEND_URL")
                .unwrap_or_else(|_| "http://localhost:8081".to_string()),
        })
    }

    /// Workers 側のシークレット解決。`wrangler secret` / `.dev.vars` の値は
    /// `env.secret()` で、[vars] の値は `env.var()` で取れる
    #[cfg(target_arch = "wasm32")]
    pub fn from_worker_env(env: &worker::Env) -> Result<Self, String> {
        fn get(env: &worker::Env, name: &str) -> Option<String> {
            env.secret(name)
                .ok()
                .map(|v| v.to_string())
                .or_else(|| env.var(name).ok().map(|v| v.to_string()))
        }
        let required = |name: &str| {
            get(env, name).ok_or_else(|| {
                format!(
                    "{name} が設定されていません（wrangler secret / .dev.vars を確認してください）"
                )
            })
        };
        Ok(Self {
            jquants_api_key: get(env, "JQUANTS_API_KEY"),
            google_client_id: Some(required("GOOGLE_CLIENT_ID")?),
            google_client_secret: Some(required("GOOGLE_CLIENT_SECRET")?),
            frontend_url: get(env, "FRONTEND_URL")
                .unwrap_or_else(|| "http://localhost:8081".to_string()),
        })
    }
}

#[derive(Clone, Default)]
pub struct DividendCacheState {
    pub running: Arc<AtomicBool>,
}

#[derive(Clone)]
pub struct AppState {
    pub pool: Db,
    pub secrets: Arc<Secrets>,
    #[cfg(not(target_arch = "wasm32"))]
    pub client: Client,
    /// 配当キャッシュのバックグラウンド更新状態（多重起動防止）
    #[cfg(not(target_arch = "wasm32"))]
    pub dividend_cache: DividendCacheState,
    /// 起動時に解決した実行設定
    pub config: Arc<Config>,
    /// 起動時に構築した Google OAuth クライアント（認証情報が揃わない場合は None）
    pub google_oauth: Option<GoogleOAuthClient>,
    /// Workers 側の J-Quants クライアント（JQUANTS_API_KEY 未設定なら None）
    #[cfg(target_arch = "wasm32")]
    pub jquants_client: Option<crate::services::jquants::JQuantsClient>,
}
#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::test_env::{ENV_MUTEX, EnvGuard},
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
            let _google_client_id = EnvGuard::set("GOOGLE_CLIENT_ID", Some("client-id"));
            let _google_client_secret =
                EnvGuard::set("GOOGLE_CLIENT_SECRET", Some("client-secret"));
            for (frontend_url, expected) in [
                (None, "http://localhost:8081"),
                (Some("http://localhost:8080"), "http://localhost:8080"),
                (
                    Some("https://frontend.example.com"),
                    "https://frontend.example.com",
                ),
            ] {
                let _fe = EnvGuard::set("FRONTEND_URL", frontend_url);
                assert_eq!(Secrets::from_env().unwrap().frontend_url, expected);
            }
        }
        {
            let _db = EnvGuard::set(
                "DATABASE_URL",
                Some("postgresql://user:password@localhost/test"),
            );
            let _google_client_id = EnvGuard::set("GOOGLE_CLIENT_ID", Some("client-id"));
            let _google_client_secret =
                EnvGuard::set("GOOGLE_CLIENT_SECRET", Some("client-secret"));
            let _jq = EnvGuard::set("JQUANTS_API_KEY", None);
            assert!(Secrets::from_env().unwrap().jquants_api_key.is_none());
        }
        {
            let _db = EnvGuard::set("DATABASE_URL", Some("postgresql://user:password/test"));
            let _google_client_id = EnvGuard::set("GOOGLE_CLIENT_ID", None);
            let _google_client_secret =
                EnvGuard::set("GOOGLE_CLIENT_SECRET", Some("client-secret"));
            let err_msg = Secrets::from_env().unwrap_err();
            assert!(
                err_msg.contains("GOOGLE_CLIENT_ID"),
                "エラーメッセージに GOOGLE_CLIENT_ID が含まれること: {err_msg}"
            );
        }
        {
            let _db = EnvGuard::set("DATABASE_URL", Some("postgresql://user:password/test"));
            let _google_client_id = EnvGuard::set("GOOGLE_CLIENT_ID", Some("client-id"));
            let _google_client_secret = EnvGuard::set("GOOGLE_CLIENT_SECRET", None);
            let err_msg = Secrets::from_env().unwrap_err();
            assert!(
                err_msg.contains("GOOGLE_CLIENT_SECRET"),
                "エラーメッセージに GOOGLE_CLIENT_SECRET が含まれること: {err_msg}"
            );
        }
    }
}
