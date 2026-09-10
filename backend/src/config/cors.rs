use axum::http::{HeaderValue, Method};
use std::time::Duration;
use tower_http::cors::CorsLayer;

pub fn build_cors_layer(cors_origins: &[String]) -> CorsLayer {
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

    let cors_origins = cors_origins.to_vec();

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
        .max_age(Duration::from_secs(3600))
}

pub fn parse_cors_origins(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(|origin| origin.trim())
        .filter(|origin| !origin.is_empty())
        .map(|origin| origin.to_string())
        .collect()
}

pub fn is_localhost_origin(origin: &str) -> bool {
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
#[cfg(test)]
mod tests {
    use {
        super::{build_cors_layer, is_localhost_origin, parse_cors_origins},
        crate::state::AppState,
        axum::{
            body::Body,
            http::{header::ACCESS_CONTROL_ALLOW_ORIGIN, Method, Request},
            routing::get,
            Router,
        },
        reqwest::Client,
        std::sync::Arc,
        tower::ServiceExt,
    };

    fn strings(origins: &[&str]) -> Vec<String> {
        origins.iter().map(|origin| (*origin).to_string()).collect()
    }

    fn build_test_app(cors_origins: &[String]) -> Router {
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

        Router::new()
            .route("/health", get(|| async { "OK" }))
            .layer(build_cors_layer(cors_origins))
            .with_state(AppState {
                pool,
                secrets,
                client: Client::new(),
                dividend_cache: crate::state::DividendCacheState::default(),
            })
    }

    #[test]
    fn test_is_localhost_origin() {
        for (origin, expected) in [
            ("http://localhost", true),
            ("https://localhost:3000", true),
            ("http://localhost.:5173", true),
            ("http://127.0.0.1:8080", true),
            ("https://[::1]:3000", true),
            ("http://[0:0:0:0:0:0:0:1]:5173/path", true),
            ("http://[::1", false),
            ("https://localhost.example.com", false),
            ("https://127.0.0.1.example.com:3000", false),
            ("https://frontend.example.com", false),
        ] {
            assert_eq!(
                is_localhost_origin(origin),
                expected,
                "unexpected localhost classification: {origin}"
            );
        }
        for (input, expected) in [
            (
                "https://app.example.com,http://localhost:3000",
                &["https://app.example.com", "http://localhost:3000"][..],
            ),
            (
                " https://app.example.com , http://localhost:3000 ",
                &["https://app.example.com", "http://localhost:3000"][..],
            ),
            ("", &[][..]),
            (
                "https://app.example.com,http://localhost:3000,",
                &["https://app.example.com", "http://localhost:3000"][..],
            ),
        ] {
            assert_eq!(parse_cors_origins(input), strings(expected));
        }
    }

    #[tokio::test]
    async fn test_cors_predicate_with_invalid_configured_origin() {
        let app = build_test_app(&["https://frontend.example.com\n".to_string()]);
        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::OPTIONS)
                    .uri("/health")
                    .header("origin", "https://frontend.example.com")
                    .header("access-control-request-method", "GET")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert!(response.status().is_success());
        assert!(response
            .headers()
            .get(ACCESS_CONTROL_ALLOW_ORIGIN)
            .is_none());
    }
}
