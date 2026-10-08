#[cfg(not(target_arch = "wasm32"))]
use std::env;

use crate::services::domain::bulk::RowLimit;

pub mod cors;
pub mod environment;

pub use cors::{
    build_cors_layer, is_localhost_origin, is_pages_preview_origin, parse_cors_origins,
};
pub use environment::RuntimeEnv;

/// `USER_ROW_LIMIT` 未設定時の既定値（ユーザー1人あたりの登録行数上限）
pub const DEFAULT_USER_ROW_LIMIT: RowLimit = RowLimit::new(100_000);

#[derive(Debug, Clone)]
pub struct Config {
    pub cors_origins: Vec<String>,
    pub database_max_connections: u32,
    /// `/auth/*` ルートへのレート制限（リクエスト/秒）。0 は無制限
    pub auth_rate_limit_rps: u32,
    /// CSV アップロードルートへの IP 単位レート制限（リクエスト/秒）。0 は無制限
    pub csv_rate_limit_rps: u32,
    /// 銘柄検索ルート（`GET /api/v1/stocks`）への IP 単位レート制限（リクエスト/秒）。0 は無制限
    pub stock_search_rate_limit_rps: u32,
    /// 認証済みデータ系ルート(`/api/v1/*`)への IP 単位レート制限（リクエスト/秒）。0 は無制限
    pub data_rate_limit_rps: u32,
    /// 実行環境（起動時に一度だけ解決）
    pub runtime_env: RuntimeEnv,
    /// Cookie を Secure で発行するか。本番では SECURE_COOKIE の値に関わらず true
    pub secure_cookie: bool,
    /// OAuth リダイレクト等に使うバックエンドの外部 URL
    pub backend_url: String,
    /// サーバの待ち受けアドレス
    pub server_addr: String,
    /// ユーザー1人あたりの登録行数上限
    pub user_row_limit: RowLimit,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            cors_origins: vec![
                "https://shoken-webapp.vercel.app".to_string(),
                "https://shoken-webapp.pages.dev".to_string(),
                "http://localhost:8081".to_string(),
                "http://127.0.0.1:8081".to_string(),
                "http://[::1]:8081".to_string(),
                "http://localhost.:8081".to_string(),
            ],
            database_max_connections: 5,
            auth_rate_limit_rps: 10,
            csv_rate_limit_rps: 2,
            stock_search_rate_limit_rps: 10,
            data_rate_limit_rps: 10,
            // 未設定は本番扱いにする fail-safe と同じ既定値に揃える
            runtime_env: RuntimeEnv::Production,
            secure_cookie: true,
            backend_url: "http://localhost:3001".to_string(),
            server_addr: "0.0.0.0:3001".to_string(),
            user_row_limit: DEFAULT_USER_ROW_LIMIT,
        }
    }
}

impl Config {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn from_env() -> Self {
        let mut config = Config::default();

        config.runtime_env = RuntimeEnv::from_env();
        config.secure_cookie = config.is_production()
            || env::var("SECURE_COOKIE").is_ok_and(|v| v == "true" || v == "1");

        let port = env::var("PORT").unwrap_or_else(|_| "3001".to_string());
        config.backend_url =
            env::var("BACKEND_URL").unwrap_or_else(|_| format!("http://localhost:{port}"));
        config.server_addr = format!("0.0.0.0:{port}");

        if let Ok(v) = env::var("USER_ROW_LIMIT")
            && let Ok(n) = v.parse()
        {
            config.user_row_limit = RowLimit::new(n);
        }

        if let Ok(origins) = env::var("CORS_ORIGINS") {
            let parsed = parse_cors_origins(&origins);
            if !parsed.is_empty() {
                config.cors_origins = parsed;
            }
        }

        let parse_rps = |var: &str| env::var(var).ok().and_then(|v| v.parse::<u32>().ok());
        if let Some(v) = parse_rps("AUTH_RATE_LIMIT_RPS") {
            config.auth_rate_limit_rps = v;
        }
        if let Some(v) = parse_rps("STOCK_SEARCH_RATE_LIMIT_RPS") {
            config.stock_search_rate_limit_rps = v;
        }
        if let Some(v) = parse_rps("CSV_RATE_LIMIT_RPS") {
            config.csv_rate_limit_rps = v;
        }
        if let Some(v) = parse_rps("DATA_RATE_LIMIT_RPS") {
            config.data_rate_limit_rps = v;
        }

        if config.is_production() {
            config
                .cors_origins
                .retain(|origin| !is_localhost_origin(origin));
        }

        config
    }

    /// Workers 側の設定解決。値は `wrangler.toml` の [vars] / `.dev.vars` から取る。
    /// レート制限の上限値は [[ratelimits]] バインディング側が持つため、
    /// ここでは *_rate_limit_rps を解決しない
    #[cfg(target_arch = "wasm32")]
    pub fn from_worker_env(env: &worker::Env) -> Self {
        let get = |name: &str| env.var(name).ok().map(|v| v.to_string());
        let mut config = Config::default();

        config.runtime_env =
            RuntimeEnv::resolve(get("RUST_ENV").as_deref(), get("APP_ENV").as_deref());
        config.secure_cookie =
            config.is_production() || get("SECURE_COOKIE").is_some_and(|v| v == "true" || v == "1");

        if let Some(url) = get("BACKEND_URL") {
            config.backend_url = url;
        }
        if let Some(origins) = get("CORS_ORIGINS") {
            let parsed = parse_cors_origins(&origins);
            if !parsed.is_empty() {
                config.cors_origins = parsed;
            }
        }

        if config.is_production() {
            config
                .cors_origins
                .retain(|origin| !is_localhost_origin(origin));
        }

        config
    }

    pub fn is_production(&self) -> bool {
        self.runtime_env.is_production()
    }
}
#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::{
            routes::app_router,
            state::AppState,
            test_env::{ENV_MUTEX, EnvGuard},
        },
        axum::{
            Router,
            body::Body,
            http::{Method, Request, StatusCode, header::ACCESS_CONTROL_ALLOW_ORIGIN},
        },
        reqwest::Client,
        std::sync::Arc,
        tower::ServiceExt,
    };
    fn build_test_app(config: &Config) -> Router {
        let database_url = "postgresql://user:password@localhost/test_db";
        let pool = crate::db::connect_pool_lazy(database_url, 1)
            .expect("Failed to create connection pool");
        let secrets = Arc::new(crate::state::Secrets {
            database_url: database_url.to_string(),
            jquants_api_key: None,
            google_client_id: None,
            google_client_secret: None,
            frontend_url: "http://localhost:8080".to_string(),
        });
        app_router(
            AppState {
                pool,
                secrets,
                client: Client::new(),
                dividend_cache: crate::state::DividendCacheState::default(),
                config: Arc::new(config.clone()),
                google_oauth: None,
            },
            &Arc::new(std::sync::atomic::AtomicBool::new(true)),
        )
    }
    async fn preflight(app: Router, origin: &str) -> axum::response::Response {
        app.oneshot(
            Request::builder()
                .method(Method::OPTIONS)
                .uri("/health")
                .header("origin", origin)
                .header("access-control-request-method", "GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap()
    }
    async fn post_with_origin(app: Router, origin: &str) -> axum::response::Response {
        app.oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/health")
                .header("origin", origin)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap()
    }
    fn allowed_origin(response: &axum::response::Response) -> Option<&str> {
        response
            .headers()
            .get(ACCESS_CONTROL_ALLOW_ORIGIN)
            .and_then(|value| value.to_str().ok())
    }
    #[test]
    fn test_config_creation() {
        let _guard = ENV_MUTEX.blocking_lock();
        let config = Config::default();
        assert_eq!(
            (
                config.database_max_connections,
                config.csv_rate_limit_rps,
                config
                    .cors_origins
                    .contains(&"https://shoken-webapp.vercel.app".to_string()),
                config
                    .cors_origins
                    .contains(&"http://localhost:8081".to_string())
            ),
            (5, 2, true, true)
        );
        let _cors_layer = build_cors_layer(&config.cors_origins);
        let config = Config {
            cors_origins: vec!["http://example.com".to_string()],
            database_max_connections: 10,
            ..Config::default()
        };
        assert_eq!(
            (
                config.database_max_connections,
                config.cors_origins.len(),
                config.cors_origins[0].as_str(),
                config.csv_rate_limit_rps
            ),
            (10, 1, "http://example.com", 2)
        );
    }

    #[tokio::test]
    async fn test_default_frontend_cors_origins() {
        let _lock = ENV_MUTEX.lock().await;
        let _cors_origins = EnvGuard::set("CORS_ORIGINS", None);
        let _rust_env = EnvGuard::set("RUST_ENV", None);
        for app_env in ["development", "production"] {
            let _app_env = EnvGuard::set("APP_ENV", Some(app_env));
            let config = Config::from_env();
            let app = build_test_app(&config);
            for host in ["localhost", "127.0.0.1", "[::1]", "localhost."] {
                let origin = format!("http://{host}:8081");
                let response = preflight(app.clone(), &origin).await;
                assert_eq!(
                    allowed_origin(&response),
                    if app_env == "development" {
                        Some(origin.as_str())
                    } else {
                        None
                    },
                    "APP_ENV={app_env} origin={origin}"
                );
                let response = preflight(app.clone(), &format!("http://{host}:8080")).await;
                assert_eq!(allowed_origin(&response), None);
            }
        }
    }

    #[test]
    fn test_config_backend_url_and_server_addr_from_env() {
        let _guard = ENV_MUTEX.blocking_lock();
        // (BACKEND_URL, PORT, backend_url, server_addr)
        let cases: [(
            Option<&'static str>,
            Option<&'static str>,
            &'static str,
            &'static str,
        ); 4] = [
            (
                Some("https://api.example.com"),
                Some("9000"),
                "https://api.example.com",
                "0.0.0.0:9000",
            ),
            (
                Some("https://api.example.com"),
                None,
                "https://api.example.com",
                "0.0.0.0:3001",
            ),
            (None, Some("8080"), "http://localhost:8080", "0.0.0.0:8080"),
            (None, None, "http://localhost:3001", "0.0.0.0:3001"),
        ];
        for (backend_url, port, expected_url, expected_addr) in cases {
            temp_env::with_vars([("BACKEND_URL", backend_url), ("PORT", port)], || {
                let config = Config::from_env();
                assert_eq!(
                    (config.backend_url.as_str(), config.server_addr.as_str()),
                    (expected_url, expected_addr),
                    "BACKEND_URL={backend_url:?} PORT={port:?}"
                );
            });
        }
    }

    #[test]
    fn test_config_user_row_limit_from_env() {
        let _guard = ENV_MUTEX.blocking_lock();
        for (value, expected) in [
            (Some("50"), 50),
            (Some("abc"), 100_000),
            (Some("-5"), -5),
            (None, 100_000),
        ] {
            temp_env::with_var("USER_ROW_LIMIT", value, || {
                assert_eq!(
                    Config::from_env().user_row_limit.get(),
                    expected,
                    "USER_ROW_LIMIT={value:?}"
                );
            });
        }
    }
    #[test]
    fn test_config_secure_cookie_from_env() {
        let _guard = ENV_MUTEX.blocking_lock();
        // 本番(未設定・不明値を含む)では SECURE_COOKIE=false を明示しても無効にできない(fail-safe)
        type Case = (Option<&'static str>, Option<&'static str>, bool);
        let cases: [Case; 6] = [
            (None, None, true),
            (Some("false"), None, true),
            (Some("false"), Some("production"), true),
            (None, Some("development"), false),
            (Some("false"), Some("development"), false),
            (Some("true"), Some("development"), true),
        ];
        for (secure_cookie, app_env, expected) in cases {
            temp_env::with_vars(
                [
                    ("SECURE_COOKIE", secure_cookie),
                    ("APP_ENV", app_env),
                    ("RUST_ENV", None),
                ],
                || {
                    assert_eq!(
                        Config::from_env().secure_cookie,
                        expected,
                        "SECURE_COOKIE={secure_cookie:?} APP_ENV={app_env:?}"
                    );
                },
            );
        }
    }

    #[test]
    fn test_config_from_env_stock_search_rps() {
        let _guard = ENV_MUTEX.blocking_lock();
        assert_eq!(Config::default().stock_search_rate_limit_rps, 10);
        let _env = EnvGuard::set("STOCK_SEARCH_RATE_LIMIT_RPS", Some("3"));
        assert_eq!(Config::from_env().stock_search_rate_limit_rps, 3);
    }

    #[tokio::test]
    async fn test_config_from_env() {
        let _lock = ENV_MUTEX.lock().await;
        {
            let _csv_rate_limit_rps = EnvGuard::set("CSV_RATE_LIMIT_RPS", Some("7"));
            assert_eq!(Config::from_env().csv_rate_limit_rps, 7);
        }
        {
            let _app_env = EnvGuard::set("APP_ENV", Some("production"));
            let _cors_origins = EnvGuard::set(
                "CORS_ORIGINS",
                Some("https://shoken-webapp.vercel.app,http://localhost:8080"),
            );
            let app = build_test_app(&Config::from_env());
            assert_eq!(
                allowed_origin(&preflight(app.clone(), "https://shoken-webapp.vercel.app").await),
                Some("https://shoken-webapp.vercel.app")
            );
            assert_eq!(
                allowed_origin(&preflight(app, "http://localhost:8080").await),
                None
            );
        }
        {
            let _app_env = EnvGuard::set("APP_ENV", Some("development"));
            let _cors_origins = EnvGuard::set(
                "CORS_ORIGINS",
                Some("http://custom-origin.example.com:8080"),
            );
            let app = build_test_app(&Config::from_env());
            assert_ne!(
                post_with_origin(app.clone(), "http://custom-origin.example.com:8080")
                    .await
                    .status(),
                StatusCode::FORBIDDEN,
                "許可されたカスタムオリジンは通過すべき"
            );
            assert_eq!(
                post_with_origin(app, "http://disallowed-origin.example.com")
                    .await
                    .status(),
                StatusCode::FORBIDDEN,
                "許可されていないオリジンは拒否すべき"
            );
        }
        {
            // localhost オリジンの保持を検証するため開発用の値を明示する(未設定は本番扱いで除去される)
            let _app_env = EnvGuard::set("APP_ENV", Some("development"));
            let _cors_origins = EnvGuard::set("CORS_ORIGINS", Some("http://localhost:8080"));
            let app = build_test_app(&Config::from_env());
            assert_eq!(
                allowed_origin(&preflight(app.clone(), "http://localhost:8080").await),
                Some("http://localhost:8080")
            );
            let resp = post_with_origin(app, "http://evil.example.com").await;
            assert_eq!(
                (
                    resp.status(),
                    resp.headers()
                        .get("X-Content-Type-Options")
                        .and_then(|v| v.to_str().ok()),
                    resp.headers()
                        .get("X-Frame-Options")
                        .and_then(|v| v.to_str().ok())
                ),
                (StatusCode::FORBIDDEN, Some("nosniff"), Some("DENY"))
            );
        }
    }
}
