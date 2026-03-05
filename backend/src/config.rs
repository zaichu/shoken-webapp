use axum::http::{HeaderValue, Method};
use std::env;
use tower_http::cors::CorsLayer;

pub struct Config {
    pub cors_origins: Vec<String>,
    pub database_max_connections: u32,
    /// `/auth/*` ルートへのレート制限（リクエスト/秒）。0 は無制限
    pub auth_rate_limit_rps: u32,
    /// `/jquants/*` ルートへのレート制限（リクエスト/秒）。0 は無制限
    pub jquants_rate_limit_rps: u32,
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

        if let Ok(rps) = env::var("AUTH_RATE_LIMIT_RPS") {
            if let Ok(v) = rps.parse::<u32>() {
                config.auth_rate_limit_rps = v;
            }
        }

        if let Ok(rps) = env::var("JQUANTS_RATE_LIMIT_RPS") {
            if let Ok(v) = rps.parse::<u32>() {
                config.jquants_rate_limit_rps = v;
            }
        }

        if is_production_env() {
            config
                .cors_origins
                .retain(|origin| !is_localhost_origin(origin));
        }

        config
    }

    pub fn build_cors_layer(&self) -> CorsLayer {
        let allowed_headers = vec![
            axum::http::header::CONTENT_TYPE,
            axum::http::header::ACCEPT,
            axum::http::header::ORIGIN,
            axum::http::header::AUTHORIZATION,
        ];

        let allowed_methods = vec![
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ];

        let cors_origins = self.cors_origins.clone();

        CorsLayer::new()
            .allow_origin(tower_http::cors::AllowOrigin::predicate(
                move |origin, _| {
                    cors_origins.iter().any(|allowed_origin| {
                        if let Ok(header_value) = allowed_origin.parse::<HeaderValue>() {
                            origin.eq(&header_value)
                        } else {
                            false
                        }
                    })
                },
            ))
            .allow_methods(allowed_methods)
            .allow_headers(allowed_headers)
            .allow_credentials(true)
    }
}

fn parse_cors_origins(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(|origin| origin.trim())
        .filter(|origin| !origin.is_empty())
        .map(|origin| origin.to_string())
        .collect()
}

fn is_localhost_origin(origin: &str) -> bool {
    let origin = origin.trim();
    let origin = origin
        .strip_prefix("http://")
        .or_else(|| origin.strip_prefix("https://"))
        .unwrap_or(origin);
    let origin = origin.split('/').next().unwrap_or(origin);

    if origin.starts_with('[') {
        if let Some(end) = origin.find(']') {
            let host = &origin[..=end];
            return host == "[::1]" || host == "[0:0:0:0:0:0:0:1]";
        }
        return false;
    }

    let host = origin.split(':').next().unwrap_or(origin);
    matches!(host, "localhost" | "localhost." | "127.0.0.1")
}

/// 本番環境かどうかを判定
/// RUST_ENV=production または APP_ENV=production の場合に true
/// 明示的なフラグがない場合のみ BACKEND_URL の https:// スキームで判定
pub fn is_production_env() -> bool {
    if let Ok(v) = env::var("RUST_ENV") {
        return v == "production";
    }
    if let Ok(v) = env::var("APP_ENV") {
        return v == "production";
    }
    env::var("BACKEND_URL")
        .map(|url| url.starts_with("https://"))
        .unwrap_or(false)
}

/// バックエンドのベースURLを取得
pub fn backend_url() -> String {
    env::var("BACKEND_URL").unwrap_or_else(|_| {
        let port = env::var("PORT").unwrap_or_else(|_| "3001".to_string());
        format!("http://localhost:{}", port)
    })
}

/// サーバーのバインドアドレスを取得
pub fn server_addr() -> String {
    let port = env::var("PORT").unwrap_or_else(|_| "3001".to_string());
    format!("0.0.0.0:{}", port)
}

/// CookieをSecureで発行するか判定
/// BACKEND_URL が https:// で始まる場合、または SECURE_COOKIE=true の場合に true
pub fn is_secure_cookie() -> bool {
    if let Ok(secure) = env::var("SECURE_COOKIE") {
        return secure == "true" || secure == "1";
    }

    env::var("BACKEND_URL")
        .map(|url| url.starts_with("https://"))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{routes::app_router, state::AppState};
    use axum::{
        body::Body,
        http::{header::ACCESS_CONTROL_ALLOW_ORIGIN, Method, Request, StatusCode},
        Router,
    };
    use reqwest::Client;
    use std::sync::{Arc, Mutex};
    use tower::ServiceExt;

    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    struct EnvGuard {
        key: &'static str,
        previous: Option<String>,
    }

    impl EnvGuard {
        fn set(key: &'static str, value: Option<&str>) -> Self {
            let previous = env::var(key).ok();
            match value {
                Some(value) => env::set_var(key, value),
                None => env::remove_var(key),
            }
            Self { key, previous }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            match &self.previous {
                Some(value) => env::set_var(self.key, value),
                None => env::remove_var(self.key),
            }
        }
    }

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
            background_task_running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
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

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.database_max_connections, 5);
        assert!(config
            .cors_origins
            .contains(&"https://shoken-webapp.vercel.app".to_string()));
        assert!(config
            .cors_origins
            .contains(&"http://localhost:8080".to_string()));
    }

    #[test]
    fn test_cors_layer_creation() {
        let config = Config::default();
        let _cors_layer = config.build_cors_layer();
        // CORSレイヤーが正常に作成されることを確認
    }

    #[test]
    fn test_custom_config() {
        let config = Config {
            cors_origins: vec!["http://example.com".to_string()],
            database_max_connections: 10,
            auth_rate_limit_rps: 10,
            jquants_rate_limit_rps: 5,
        };

        assert_eq!(config.database_max_connections, 10);
        assert_eq!(config.cors_origins.len(), 1);
        assert_eq!(config.cors_origins[0], "http://example.com");
    }

    #[test]
    fn test_backend_url_returns_valid_url() {
        let url = backend_url();
        // 環境変数の設定状況に応じて期待値を決定
        if let Ok(expected) = env::var("BACKEND_URL") {
            // BACKEND_URL が設定されている場合はその値を返す
            assert_eq!(url, expected);
        } else if let Ok(port) = env::var("PORT") {
            // PORT のみ設定されている場合はローカルホストURLを返す
            assert_eq!(url, format!("http://localhost:{}", port));
        } else {
            // 何も設定されていない場合はデフォルトポート3001を使用
            assert_eq!(url, "http://localhost:3001");
        }
    }

    #[test]
    fn test_server_addr_format() {
        let addr = server_addr();
        // 0.0.0.0:ポート番号 の形式であることを確認
        assert!(addr.starts_with("0.0.0.0:"));
    }

    #[test]
    fn test_is_secure_cookie_default() {
        // 環境変数未設定時はfalse
        // 注意: BACKEND_URL または SECURE_COOKIE が設定されている場合は
        // その値に依存する
        let _ = is_secure_cookie();
        // テストはパニックしないことを確認
    }

    #[tokio::test]
    async fn test_cors_filters_localhost_in_production() {
        let _lock = ENV_MUTEX.lock().unwrap();
        let _app_env = EnvGuard::set("APP_ENV", Some("production"));
        let _cors_origins = EnvGuard::set(
            "CORS_ORIGINS",
            Some("https://shoken-webapp.vercel.app,http://localhost:8080"),
        );

        let config = Config::from_env();
        let app = build_test_app(&config);

        let response = preflight(app.clone(), "https://shoken-webapp.vercel.app").await;
        let allowed_origin = response
            .headers()
            .get(ACCESS_CONTROL_ALLOW_ORIGIN)
            .and_then(|value| value.to_str().ok());
        assert_eq!(allowed_origin, Some("https://shoken-webapp.vercel.app"));

        let response = preflight(app, "http://localhost:8080").await;
        let allowed_origin = response
            .headers()
            .get(ACCESS_CONTROL_ALLOW_ORIGIN)
            .and_then(|value| value.to_str().ok());
        assert_eq!(allowed_origin, None);
    }

    /// CORS_ORIGINS で設定したオリジンが validate_origin ミドルウェアにも反映されることを確認する
    /// （POST リクエストに対して Config::cors_origins と validate_origin が同一リストを参照する）
    #[tokio::test]
    async fn test_validate_origin_respects_cors_origins_env() {
        let _lock = ENV_MUTEX.lock().unwrap();
        let _app_env = EnvGuard::set("APP_ENV", None);
        let _cors_origins = EnvGuard::set(
            "CORS_ORIGINS",
            Some("http://custom-origin.example.com:8080"),
        );

        let config = Config::from_env();
        let app = build_test_app(&config);

        // 許可オリジンからの POST → 通過（/health は GET のみだが validate_origin のチェックが目的）
        let req = Request::builder()
            .method(Method::POST)
            .uri("/health")
            .header("origin", "http://custom-origin.example.com:8080")
            .body(Body::empty())
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_ne!(
            resp.status(),
            StatusCode::FORBIDDEN,
            "許可されたカスタムオリジンは通過すべき"
        );

        // 許可されていないオリジンからの POST → 403
        let req = Request::builder()
            .method(Method::POST)
            .uri("/health")
            .header("origin", "http://disallowed-origin.example.com")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::FORBIDDEN,
            "許可されていないオリジンは拒否すべき"
        );
    }

    #[tokio::test]
    async fn test_cors_allows_localhost_in_non_production() {
        let _lock = ENV_MUTEX.lock().unwrap();
        let _app_env = EnvGuard::set("APP_ENV", None);
        let _cors_origins = EnvGuard::set("CORS_ORIGINS", Some("http://localhost:8080"));

        let config = Config::from_env();
        let app = build_test_app(&config);

        let response = preflight(app, "http://localhost:8080").await;
        let allowed_origin = response
            .headers()
            .get(ACCESS_CONTROL_ALLOW_ORIGIN)
            .and_then(|value| value.to_str().ok());
        assert_eq!(allowed_origin, Some("http://localhost:8080"));
    }

    /// 不正オリジンによる 403 にもセキュリティヘッダーが付くことを確認する
    #[tokio::test]
    async fn test_security_headers_on_403_response() {
        let _lock = ENV_MUTEX.lock().unwrap();
        let _app_env = EnvGuard::set("APP_ENV", None);
        let _cors_origins = EnvGuard::set("CORS_ORIGINS", Some("http://localhost:8080"));

        let config = Config::from_env();
        let app = build_test_app(&config);

        let req = Request::builder()
            .method(Method::POST)
            .uri("/health")
            .header("origin", "http://evil.example.com")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
        assert_eq!(
            resp.headers().get("X-Content-Type-Options").unwrap(),
            "nosniff"
        );
        assert_eq!(resp.headers().get("X-Frame-Options").unwrap(), "DENY");
    }
}
