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
    /// 実行環境（起動時に一度だけ解決）
    pub runtime_env: RuntimeEnv,
    /// Cookie を Secure で発行するか。本番では SECURE_COOKIE の値に関わらず true
    pub secure_cookie: bool,
    /// OAuth リダイレクト等に使うバックエンドの外部 URL
    pub backend_url: String,
    /// ユーザー1人あたりの登録行数上限
    pub user_row_limit: RowLimit,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            cors_origins: vec![
                "https://shoken-webapp.pages.dev".to_string(),
                "http://localhost:8081".to_string(),
                "http://127.0.0.1:8081".to_string(),
                "http://[::1]:8081".to_string(),
                "http://localhost.:8081".to_string(),
            ],
            // 未設定は本番扱いにする fail-safe と同じ既定値に揃える
            runtime_env: RuntimeEnv::Production,
            secure_cookie: true,
            backend_url: "http://localhost:8787".to_string(),
            user_row_limit: DEFAULT_USER_ROW_LIMIT,
        }
    }
}

impl Config {
    /// Workers 側の設定解決。値は `wrangler.toml` の [vars] / `.dev.vars` から取る。
    /// レート制限の上限値は [[ratelimits]] バインディング側が持つ
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
        if let Some(v) = get("USER_ROW_LIMIT")
            && let Ok(n) = v.parse()
        {
            config.user_row_limit = RowLimit::new(n);
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
        crate::middleware::{add_security_headers, validate_origin},
        axum::{
            Router,
            body::Body,
            http::{Method, Request, StatusCode, header::ACCESS_CONTROL_ALLOW_ORIGIN},
            middleware,
            routing::get,
        },
        std::sync::Arc,
        tower::ServiceExt,
    };

    /// worker_entry の router と同じレイヤ順で CORS/origin 検証/セキュリティヘッダを
    /// 再現したテスト用ルータ
    fn build_test_app(config: &Config) -> Router {
        let allowed_origins = Arc::new(config.cors_origins.clone());
        let strict_origin_check = config.is_production();
        let secure_cookie = config.secure_cookie;
        Router::new()
            .route("/health", get(|| async { "OK" }))
            .layer(middleware::from_fn(move |req, next| {
                let origins = allowed_origins.clone();
                async move { validate_origin(origins, strict_origin_check, req, next).await }
            }))
            .layer(build_cors_layer(&config.cors_origins))
            .layer(middleware::from_fn(move |req, next| {
                add_security_headers(secure_cookie, req, next)
            }))
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
        assert!(
            config
                .cors_origins
                .contains(&"https://shoken-webapp.pages.dev".to_string())
        );
        assert!(
            config
                .cors_origins
                .contains(&"http://localhost:8081".to_string())
        );
        let _cors_layer = build_cors_layer(&config.cors_origins);
        let config = Config {
            cors_origins: vec!["http://example.com".to_string()],
            ..Config::default()
        };
        assert_eq!(config.cors_origins[0].as_str(), "http://example.com");
    }

    #[tokio::test]
    async fn test_default_frontend_cors_origins() {
        for runtime_env in [RuntimeEnv::Development, RuntimeEnv::Production] {
            let mut config = Config {
                runtime_env,
                ..Config::default()
            };
            // from_worker_env と同じく本番では localhost オリジンを除去する
            if config.is_production() {
                config
                    .cors_origins
                    .retain(|origin| !is_localhost_origin(origin));
            }
            let app = build_test_app(&config);
            for host in ["localhost", "127.0.0.1", "[::1]", "localhost."] {
                let origin = format!("http://{host}:8081");
                let response = preflight(app.clone(), &origin).await;
                assert_eq!(
                    allowed_origin(&response),
                    if runtime_env == RuntimeEnv::Development {
                        Some(origin.as_str())
                    } else {
                        None
                    },
                    "runtime_env={runtime_env:?} origin={origin}"
                );
                let response = preflight(app.clone(), &format!("http://{host}:8080")).await;
                assert_eq!(allowed_origin(&response), None);
            }
        }
    }

    #[tokio::test]
    async fn test_cors_and_origin_middleware() {
        {
            let mut config = Config {
                cors_origins: vec![
                    "https://shoken-webapp.pages.dev".to_string(),
                    "http://localhost:8080".to_string(),
                ],
                runtime_env: RuntimeEnv::Production,
                ..Config::default()
            };
            config
                .cors_origins
                .retain(|origin| !is_localhost_origin(origin));
            let app = build_test_app(&config);
            assert_eq!(
                allowed_origin(&preflight(app.clone(), "https://shoken-webapp.pages.dev").await),
                Some("https://shoken-webapp.pages.dev")
            );
            assert_eq!(
                allowed_origin(&preflight(app, "http://localhost:8080").await),
                None
            );
        }
        {
            let config = Config {
                cors_origins: vec!["http://custom-origin.example.com:8080".to_string()],
                runtime_env: RuntimeEnv::Development,
                secure_cookie: false,
                ..Config::default()
            };
            let app = build_test_app(&config);
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
            let config = Config {
                cors_origins: vec!["http://localhost:8080".to_string()],
                runtime_env: RuntimeEnv::Development,
                secure_cookie: false,
                ..Config::default()
            };
            let app = build_test_app(&config);
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
