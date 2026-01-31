use axum::http::{HeaderValue, Method};
use std::env;
use tower_http::cors::CorsLayer;

pub struct Config {
    pub cors_origins: Vec<String>,
    pub database_max_connections: u32,
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
        }
    }
}

impl Config {
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
        };

        assert_eq!(config.database_max_connections, 10);
        assert_eq!(config.cors_origins.len(), 1);
        assert_eq!(config.cors_origins[0], "http://example.com");
    }
}
