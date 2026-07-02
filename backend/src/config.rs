use std::env;

pub mod cors;
pub mod environment;

pub use cors::{build_cors_layer, is_localhost_origin, parse_cors_origins};
pub use environment::{
    backend_url, csv_rate_limit_rps, is_production_env, is_secure_cookie, server_addr,
};

pub struct Config {
    pub cors_origins: Vec<String>,
    pub database_max_connections: u32,
    /// `/auth/*` ルートへのレート制限（リクエスト/秒）。0 は無制限
    pub auth_rate_limit_rps: u32,
    /// market data 系ルート（`/api/v1/financial-statements` 等）へのレート制限（リクエスト/秒）。0 は無制限
    pub market_data_rate_limit_rps: u32,
    /// CSV アップロードルートへの IP 単位レート制限（リクエスト/秒）。0 は無制限
    pub csv_rate_limit_rps: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            cors_origins: vec![
                "https://shoken-webapp.vercel.app".to_string(),
                "http://localhost:8080".to_string(),
                "http://127.0.0.1:8080".to_string(),
                "http://[::1]:8080".to_string(),
                "http://localhost.:8080".to_string(),
            ],
            database_max_connections: 5,
            auth_rate_limit_rps: 10,
            market_data_rate_limit_rps: 5,
            csv_rate_limit_rps: 2,
        }
    }
}

impl Config {
    pub fn from_env() -> Self {
        let mut config = Config::default();

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
        // JQUANTS_RATE_LIMIT_RPS は後方互換のため環境変数名を維持
        if let Some(v) = parse_rps("JQUANTS_RATE_LIMIT_RPS") {
            config.market_data_rate_limit_rps = v;
        }

        config.csv_rate_limit_rps = csv_rate_limit_rps();

        if is_production_env() {
            config
                .cors_origins
                .retain(|origin| !is_localhost_origin(origin));
        }

        config
    }
}
#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::{
            routes::app_router,
            state::AppState,
            test_env::{EnvGuard, ENV_MUTEX},
        },
        axum::{
            body::Body,
            http::{header::ACCESS_CONTROL_ALLOW_ORIGIN, Method, Request, StatusCode},
            Router,
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
            },
            config,
            Arc::new(std::sync::atomic::AtomicBool::new(true)),
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
                    .contains(&"http://localhost:8080".to_string())
            ),
            (5, 2, true, true)
        );
        let _cors_layer = build_cors_layer(&config.cors_origins);
        let config = Config {
            cors_origins: vec!["http://example.com".to_string()],
            database_max_connections: 10,
            auth_rate_limit_rps: 10,
            market_data_rate_limit_rps: 5,
            csv_rate_limit_rps: 2,
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
        let url = backend_url();
        let expected_url = env::var("BACKEND_URL").unwrap_or_else(|_| {
            env::var("PORT")
                .map(|port| format!("http://localhost:{port}"))
                .unwrap_or_else(|_| "http://localhost:3001".to_string())
        });
        assert_eq!(url, expected_url);
        assert!(server_addr().starts_with("0.0.0.0:"));
        let _ = is_secure_cookie();
    }
    #[test]
    fn test_config_from_env_market_data_rps() {
        let _guard = ENV_MUTEX.blocking_lock();
        let _env = EnvGuard::set("JQUANTS_RATE_LIMIT_RPS", Some("7"));
        let _env2 = EnvGuard::set("FRONTEND_URL", None);
        let _env3 = EnvGuard::set("BACKEND_URL", None);

        let config = Config::from_env();
        assert_eq!(config.market_data_rate_limit_rps, 7);
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
            let _app_env = EnvGuard::set("APP_ENV", None);
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
            let _app_env = EnvGuard::set("APP_ENV", None);
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
