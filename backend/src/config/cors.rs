use axum::http::{HeaderValue, Method};
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
    use super::{is_localhost_origin, parse_cors_origins};

    #[test]
    fn test_is_localhost_origin() {
        for origin in [
            "http://localhost",
            "https://localhost:3000",
            "http://localhost.:5173",
            "http://127.0.0.1:8080",
            "https://[::1]:3000",
            "http://[0:0:0:0:0:0:0:1]:5173/path",
        ] {
            assert!(
                is_localhost_origin(origin),
                "expected {origin} to be treated as localhost"
            );
        }
        for origin in [
            "https://localhost.example.com",
            "https://127.0.0.1.example.com:3000",
            "https://frontend.example.com",
        ] {
            assert!(
                !is_localhost_origin(origin),
                "expected {origin} to be rejected as non-localhost"
            );
        }
        assert_eq!(
            parse_cors_origins("https://app.example.com,http://localhost:3000"),
            vec![
                "https://app.example.com".to_string(),
                "http://localhost:3000".to_string(),
            ]
        );
        assert_eq!(
            parse_cors_origins(" https://app.example.com , http://localhost:3000 "),
            vec![
                "https://app.example.com".to_string(),
                "http://localhost:3000".to_string(),
            ]
        );
        assert!(parse_cors_origins("").is_empty());
        assert_eq!(
            parse_cors_origins("https://app.example.com,http://localhost:3000,"),
            vec![
                "https://app.example.com".to_string(),
                "http://localhost:3000".to_string(),
            ]
        );
    }
}
