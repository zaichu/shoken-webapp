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
                if is_pages_preview_origin(origin.to_str().unwrap_or_default()) {
                    return true;
                }
                cors_origins.iter().any(|allowed_origin| {
                    match allowed_origin.parse::<HeaderValue>() {
                        Ok(header_value) => origin.eq(&header_value),
                        _ => false,
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

/// Cloudflare Pages プレビューは `<hash>.<project>.pages.dev` (ハッシュ URL) と
/// `<branch>.<project>.pages.dev` (ブランチ エイリアス) の 2 形式。
/// `<project>` は `shoken-webapp` で固定。ホスト境界を厳密に検証し、偽装サブドメインを排除する。
pub fn is_pages_preview_origin(origin: &str) -> bool {
    let Some(host) = origin.strip_prefix("https://") else {
        return false;
    };
    const SUFFIX: &str = ".shoken-webapp.pages.dev";
    let Some(prefix) = host.strip_suffix(SUFFIX) else {
        return false;
    };
    // prefix が空でないこと（本番ドメイン shoken-webapp.pages.dev は除外）
    // かつドット・コロン・スラッシュを含まないこと（サブドメイン偽装防止）
    !prefix.is_empty() && !prefix.contains(['.', ':', '/'])
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
        super::{
            build_cors_layer, is_localhost_origin, is_pages_preview_origin, parse_cors_origins,
        },
        axum::{
            Router,
            body::Body,
            http::{Method, Request, header::ACCESS_CONTROL_ALLOW_ORIGIN},
            routing::get,
        },
        tower::ServiceExt,
    };

    fn strings(origins: &[&str]) -> Vec<String> {
        origins.iter().map(|origin| (*origin).to_string()).collect()
    }

    fn build_test_app(cors_origins: &[String]) -> Router {
        Router::new()
            .route("/health", get(|| async { "OK" }))
            .layer(build_cors_layer(cors_origins))
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

    #[test]
    fn test_is_pages_preview_origin() {
        for (origin, expected) in [
            // Hash-based preview URLs
            ("https://abc123.shoken-webapp.pages.dev", true),
            ("https://xyz789.shoken-webapp.pages.dev", true),
            // Branch alias preview URLs
            ("https://main.shoken-webapp.pages.dev", true),
            ("https://feature-branch.shoken-webapp.pages.dev", true),
            ("https://fix-123.shoken-webapp.pages.dev", true),
            // Production domain (not a preview)
            ("https://shoken-webapp.pages.dev", false),
            // Other projects
            ("https://abc123.other-project.pages.dev", false),
            ("https://main.other-project.pages.dev", false),
            // Subdomain spoofing attempts
            ("https://evil.shoken-webapp.pages.dev.evil.com", false),
            ("https://shoken-webapp.pages.dev.evil.com", false),
            // HTTP not allowed
            ("http://abc123.shoken-webapp.pages.dev", false),
            // Invalid characters in prefix
            ("https://abc.def.shoken-webapp.pages.dev", false),
            ("https://abc:def.shoken-webapp.pages.dev", false),
            ("https://abc/def.shoken-webapp.pages.dev", false),
        ] {
            assert_eq!(
                is_pages_preview_origin(origin),
                expected,
                "unexpected classification: {origin}"
            );
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
        assert!(
            response
                .headers()
                .get(ACCESS_CONTROL_ALLOW_ORIGIN)
                .is_none()
        );
    }

    #[tokio::test]
    async fn test_cors_predicate_pages_preview_origins() {
        let app = build_test_app(&[]);

        // Hash-based preview
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::OPTIONS)
                    .uri("/health")
                    .header("origin", "https://abc123.shoken-webapp.pages.dev")
                    .header("access-control-request-method", "GET")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(response.status().is_success());
        assert_eq!(
            response
                .headers()
                .get(ACCESS_CONTROL_ALLOW_ORIGIN)
                .and_then(|v| v.to_str().ok()),
            Some("https://abc123.shoken-webapp.pages.dev")
        );

        // Branch alias preview
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::OPTIONS)
                    .uri("/health")
                    .header("origin", "https://main.shoken-webapp.pages.dev")
                    .header("access-control-request-method", "GET")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(response.status().is_success());
        assert_eq!(
            response
                .headers()
                .get(ACCESS_CONTROL_ALLOW_ORIGIN)
                .and_then(|v| v.to_str().ok()),
            Some("https://main.shoken-webapp.pages.dev")
        );

        // Production domain (not a preview) should be rejected
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::OPTIONS)
                    .uri("/health")
                    .header("origin", "https://shoken-webapp.pages.dev")
                    .header("access-control-request-method", "GET")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(response.status().is_success());
        assert!(
            response
                .headers()
                .get(ACCESS_CONTROL_ALLOW_ORIGIN)
                .is_none()
        );
    }
}
