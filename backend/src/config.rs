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
    /// `/jquants/*` ルートへのレート制限（リクエスト/秒）。0 は無制限
    pub jquants_rate_limit_rps: u32,
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
            jquants_rate_limit_rps: 5,
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
        if let Some(v) = parse_rps("JQUANTS_RATE_LIMIT_RPS") {
            config.jquants_rate_limit_rps = v;
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
    use super::*;
    use crate::test_env::{EnvGuard, ENV_MUTEX};
    use crate::{routes::app_router, state::AppState};
    use axum::{
        body::Body,
        http::{header::ACCESS_CONTROL_ALLOW_ORIGIN, Method, Request, StatusCode},
        Router,
    };
    use reqwest::Client;
    use std::sync::Arc;
    use tower::ServiceExt;

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
        let client = Client::new();
        let state = AppState {
            pool,
            secrets,
            client,
            dividend_cache: crate::state::DividendCacheState::default(),
        };
        app_router(state, config)
    }

    async fn preflight(app: Router, origin: &str) -> axum::response::Response {
        let req = Request::builder()
            .method(Method::OPTIONS)
            .uri("/health")
            .header("origin", origin)
            .header("access-control-request-method", "GET")
            .body(Body::empty())
            .unwrap();
        app.oneshot(req).await.unwrap()
    }

    async fn post_with_origin(app: Router, origin: &str) -> axum::response::Response {
        let req = Request::builder()
            .method(Method::POST)
            .uri("/health")
            .header("origin", origin)
            .body(Body::empty())
            .unwrap();
        app.oneshot(req).await.unwrap()
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
        assert_eq!(config.database_max_connections, 5);
        assert_eq!(config.csv_rate_limit_rps, 2);
        #[rustfmt::skip]
        assert!(config.cors_origins.contains(&"https://shoken-webapp.vercel.app".to_string()));
        #[rustfmt::skip]
        assert!(config.cors_origins.contains(&"http://localhost:8080".to_string()));
        let _cors_layer = build_cors_layer(&config.cors_origins);

        let config = Config {
            cors_origins: vec!["http://example.com".to_string()],
            database_max_connections: 10,
            auth_rate_limit_rps: 10,
            jquants_rate_limit_rps: 5,
            csv_rate_limit_rps: 2,
        };

        assert_eq!(config.database_max_connections, 10);
        assert_eq!(config.cors_origins.len(), 1);
        assert_eq!(config.cors_origins[0], "http://example.com");
        assert_eq!(config.csv_rate_limit_rps, 2);
    }

    #[tokio::test]
    async fn test_config_from_env_reads_csv_rate_limit_rps() {
        let _lock = ENV_MUTEX.lock().await;
        let _csv_rate_limit_rps = EnvGuard::set("CSV_RATE_LIMIT_RPS", Some("7"));

        let config = Config::from_env();

        assert_eq!(config.csv_rate_limit_rps, 7);
    }

    #[test]
    fn test_utility_functions() {
        let url = backend_url();
        if let Ok(expected) = env::var("BACKEND_URL") {
            assert_eq!(url, expected);
        } else if let Ok(port) = env::var("PORT") {
            assert_eq!(url, format!("http://localhost:{}", port));
        } else {
            assert_eq!(url, "http://localhost:3001");
        }
        assert!(server_addr().starts_with("0.0.0.0:"));
        let _ = is_secure_cookie();
    }

    #[tokio::test]
    async fn test_cors_filters_localhost_in_production() {
        let _lock = ENV_MUTEX.lock().await;
        let _app_env = EnvGuard::set("APP_ENV", Some("production"));
        let _cors_origins = EnvGuard::set(
            "CORS_ORIGINS",
            Some("https://shoken-webapp.vercel.app,http://localhost:8080"),
        );

        let config = Config::from_env();
        let app = build_test_app(&config);

        assert_eq!(
            allowed_origin(&preflight(app.clone(), "https://shoken-webapp.vercel.app").await),
            Some("https://shoken-webapp.vercel.app")
        );
        assert_eq!(
            allowed_origin(&preflight(app, "http://localhost:8080").await),
            None
        );
    }

    #[tokio::test]
    async fn test_validate_origin_respects_cors_origins_env() {
        let _lock = ENV_MUTEX.lock().await;
        let _app_env = EnvGuard::set("APP_ENV", None);
        let _cors_origins = EnvGuard::set(
            "CORS_ORIGINS",
            Some("http://custom-origin.example.com:8080"),
        );

        let config = Config::from_env();
        let app = build_test_app(&config);

        let resp = post_with_origin(app.clone(), "http://custom-origin.example.com:8080").await;
        assert_ne!(
            resp.status(),
            StatusCode::FORBIDDEN,
            "許可されたカスタムオリジンは通過すべき"
        );

        let resp = post_with_origin(app, "http://disallowed-origin.example.com").await;
        assert_eq!(
            resp.status(),
            StatusCode::FORBIDDEN,
            "許可されていないオリジンは拒否すべき"
        );
    }

    #[tokio::test]
    async fn test_cors_allows_localhost_in_non_production() {
        let _lock = ENV_MUTEX.lock().await;
        let _app_env = EnvGuard::set("APP_ENV", None);
        let _cors_origins = EnvGuard::set("CORS_ORIGINS", Some("http://localhost:8080"));

        let config = Config::from_env();
        let app = build_test_app(&config);

        // 非本番環境では localhost が許可される
        assert_eq!(
            allowed_origin(&preflight(app.clone(), "http://localhost:8080").await),
            Some("http://localhost:8080")
        );

        // 許可されていないオリジンは 403 + セキュリティヘッダーが付与される
        let resp = post_with_origin(app, "http://evil.example.com").await;
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
        assert_eq!(
            resp.headers().get("X-Content-Type-Options").unwrap(),
            "nosniff"
        );
        assert_eq!(resp.headers().get("X-Frame-Options").unwrap(), "DENY");
    }
}
