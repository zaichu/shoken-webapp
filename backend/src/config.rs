use axum::http::{HeaderValue, Method};
use tower_http::cors::CorsLayer;

pub struct Config {
    pub cors_origins: Vec<String>,
    pub database_max_connections: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            cors_origins: vec![
                "https://zaichu.github.io".to_string(),
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
            .allow_origin(tower_http::cors::AllowOrigin::predicate(move |origin, _| {
                cors_origins.iter().any(|allowed_origin| {
                    if let Ok(header_value) = allowed_origin.parse::<HeaderValue>() {
                        origin.eq(&header_value)
                    } else {
                        false
                    }
                })
            }))
            .allow_methods(allowed_methods)
            .allow_headers(allowed_headers)
            .allow_credentials(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.database_max_connections, 5);
        assert!(config.cors_origins.contains(&"https://zaichu.github.io".to_string()));
        assert!(config.cors_origins.contains(&"http://localhost:8080".to_string()));
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