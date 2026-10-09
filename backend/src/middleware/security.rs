use std::sync::Arc;

use axum::{
    body::Body,
    http::{HeaderValue, Method, Request, StatusCode},
    middleware::Next,
    response::Response,
};

use crate::errors::simple_error_response;

/// セキュリティヘッダー付与ミドルウェア
///
/// `secure_cookie`（Secure Cookie 環境か）だけ真のとき HSTS を付与する
pub async fn add_security_headers(secure_cookie: bool, req: Request<Body>, next: Next) -> Response {
    let mut response = next.run(req).await;
    let headers = response.headers_mut();
    headers.insert(
        "X-Content-Type-Options",
        HeaderValue::from_static("nosniff"),
    );
    headers.insert("X-Frame-Options", HeaderValue::from_static("DENY"));
    headers.insert(
        "Referrer-Policy",
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    headers.insert(
        "Content-Security-Policy",
        HeaderValue::from_static("default-src 'none'"),
    );
    if secure_cookie {
        headers.insert(
            "Strict-Transport-Security",
            HeaderValue::from_static("max-age=31536000; includeSubDomains"),
        );
    }
    response
}

/// URL 文字列からオリジン部分（scheme://host[:port]）を抽出する
///
/// `starts_with` での前方一致では `https://example.com.evil/` のような
/// 類似ドメインに対してバイパスされるため、ホスト境界まで厳密に切り出す。
fn extract_origin(url: &str) -> Option<&str> {
    let after_scheme = url.find("://")?;
    let authority_start = after_scheme + 3;
    let end = url[authority_start..]
        .find(['/', '?', '#'])
        .map_or(url.len(), |i| authority_start + i);
    Some(&url[..end])
}

/// CSRF 検証失敗レスポンスを生成する
fn csrf_error() -> Response {
    simple_error_response(
        StatusCode::FORBIDDEN,
        "CSRF_ERROR",
        "不正なリクエスト元です".to_string(),
    )
}

/// Origin検証ミドルウェア（CSRF対策）
/// POST/PUT/DELETE リクエストに対して Origin ヘッダーを検証し、
/// 許可されたオリジンからのリクエストのみ通過させる。
/// allowed_origins は Config::cors_origins と一致させる。
/// `strict_origin_check`（本番相当か）はルータ構築時に解決済みの値を渡す
pub async fn validate_origin(
    allowed_origins: Arc<Vec<String>>,
    strict_origin_check: bool,
    request: Request<Body>,
    next: Next,
) -> Response {
    let method = request.method().clone();

    // GET/HEAD/OPTIONS はスキップ
    if method == Method::GET || method == Method::HEAD || method == Method::OPTIONS {
        return next.run(request).await;
    }

    // Origin ヘッダーを取得
    let origin = request
        .headers()
        .get("origin")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    match origin {
        Some(ref o)
            if allowed_origins.iter().any(|a| a == o)
                || crate::config::cors::is_pages_preview_origin(o) =>
        {
            // 許可されたオリジン → 通過
            next.run(request).await
        }
        None => {
            // Origin なし: Referer フォールバック検証
            // ブラウザはクロスオリジンリクエストで Origin を付与するが、
            // 一部の環境（リダイレクト後など）では省略されることがある。
            // Referer が存在する場合はオリジン部分を切り出し、許可済みオリジンと完全一致で検証する。
            // Referer も存在しない場合は厳格な環境では拒否し、それ以外では通過する。
            let referer = request
                .headers()
                .get("referer")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());

            match referer {
                Some(ref r)
                    if extract_origin(r).is_some_and(|o| {
                        allowed_origins.iter().any(|a| a == o)
                            || crate::config::cors::is_pages_preview_origin(o)
                    }) =>
                {
                    // 許可済みオリジンの Referer → 通過
                    next.run(request).await
                }
                Some(_) => {
                    // 不正な Referer → 拒否
                    csrf_error()
                }
                None => {
                    // Origin も Referer もなし
                    // 本番・staging 等の明示設定済み環境では CSRF リスクがあるため拒否する
                    if strict_origin_check {
                        csrf_error()
                    } else {
                        next.run(request).await
                    }
                }
            }
        }
        Some(_) => {
            // 不正なオリジン → 拒否
            csrf_error()
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Router, middleware, routing::post};
    use tower::ServiceExt;

    fn test_app(strict_origin_check: bool) -> Router {
        let allowed_origins = Arc::new(vec![
            "https://shoken-webapp.pages.dev".to_string(),
            "http://localhost:8080".to_string(),
            "http://127.0.0.1:8080".to_string(),
            "http://[::1]:8080".to_string(),
            "http://localhost.:8080".to_string(),
        ]);

        Router::new()
            .route("/test", post(|| async { "ok" }))
            .layer(middleware::from_fn(move |req, next| {
                let origins = allowed_origins.clone();
                async move { validate_origin(origins, strict_origin_check, req, next).await }
            }))
    }

    fn security_headers_app(secure_cookie: bool) -> Router {
        Router::new()
            .route("/test", post(|| async { "ok" }))
            .layer(middleware::from_fn(move |req, next| {
                add_security_headers(secure_cookie, req, next)
            }))
    }

    async fn oneshot_status(app: Router, method: Method, headers: &[(&str, &str)]) -> StatusCode {
        let builder = headers.iter().fold(
            Request::builder().method(method).uri("/test"),
            |builder, (name, value)| builder.header(*name, *value),
        );

        app.oneshot(builder.body(Body::empty()).unwrap())
            .await
            .unwrap()
            .status()
    }

    #[tokio::test]
    async fn test_validate_origin() {
        for (input, expected) in [
            (
                "http://localhost:8080/some/page",
                Some("http://localhost:8080"),
            ),
            (
                "https://shoken-webapp.pages.dev",
                Some("https://shoken-webapp.pages.dev"),
            ),
            (
                "https://shoken-webapp.pages.dev.evil.com/steal",
                Some("https://shoken-webapp.pages.dev.evil.com"),
            ),
            (
                "http://localhost:8080?view=1",
                Some("http://localhost:8080"),
            ),
            (
                "http://localhost:8080#section",
                Some("http://localhost:8080"),
            ),
        ] {
            assert_eq!(extract_origin(input), expected);
        }

        assert_ne!(
            oneshot_status(test_app(false), Method::GET, &[]).await,
            StatusCode::FORBIDDEN
        );

        for &(headers, expected) in &[
            (&[("origin", "http://localhost:8080")][..], StatusCode::OK),
            (
                &[("origin", "https://evil.example.com")][..],
                StatusCode::FORBIDDEN,
            ),
            (&[][..], StatusCode::OK),
            (
                &[("referer", "http://localhost:8080/some/page")][..],
                StatusCode::OK,
            ),
            (
                &[("referer", "http://localhost:8080?view=1")][..],
                StatusCode::OK,
            ),
            (
                &[("referer", "http://localhost:8080#section")][..],
                StatusCode::OK,
            ),
            (
                &[("referer", "http://localhost:8080.evil.com?view=1")][..],
                StatusCode::FORBIDDEN,
            ),
            (
                &[("referer", "https://evil.example.com/attack")][..],
                StatusCode::FORBIDDEN,
            ),
            (
                &[("referer", "https://shoken-webapp.pages.dev.evil.com/steal")][..],
                StatusCode::FORBIDDEN,
            ),
            (
                &[("origin", "https://shoken-webapp.pages.dev")][..],
                StatusCode::OK,
            ),
            // Cloudflare Pages preview origins
            (
                &[("origin", "https://abc123.shoken-webapp.pages.dev")][..],
                StatusCode::OK,
            ),
            (
                &[("origin", "https://main.shoken-webapp.pages.dev")][..],
                StatusCode::OK,
            ),
            (
                &[("origin", "https://feature-branch.shoken-webapp.pages.dev")][..],
                StatusCode::OK,
            ),
            // 別プロジェクトの Pages preview は許可しない
            (
                &[("origin", "https://main.other-project.pages.dev")][..],
                StatusCode::FORBIDDEN,
            ),
            (
                &[("origin", "https://evil.shoken-webapp.pages.dev.evil.com")][..],
                StatusCode::FORBIDDEN,
            ),
            (
                &[("referer", "https://abc123.shoken-webapp.pages.dev/path")][..],
                StatusCode::OK,
            ),
            (
                &[("referer", "https://main.shoken-webapp.pages.dev/path")][..],
                StatusCode::OK,
            ),
        ] {
            assert_eq!(
                oneshot_status(test_app(false), Method::POST, headers).await,
                expected
            );
        }

        let allowed_origins = Arc::new(vec!["http://localhost:8080".to_string()]);
        let app = Router::new()
            .route("/test", axum::routing::delete(|| async { "ok" }))
            .layer(middleware::from_fn(move |req, next| {
                let origins = allowed_origins.clone();
                async move { validate_origin(origins, false, req, next).await }
            }));

        assert_eq!(
            oneshot_status(
                app,
                Method::DELETE,
                &[("origin", "https://evil.example.com")],
            )
            .await,
            StatusCode::FORBIDDEN
        );
    }

    #[tokio::test]
    async fn test_validate_origin_strict() {
        // strict では Origin/Referer なしの unsafe method は 403
        assert_eq!(
            oneshot_status(test_app(true), Method::POST, &[]).await,
            StatusCode::FORBIDDEN
        );
        // strict でも GET/HEAD/OPTIONS はスキップして通過
        for method in [Method::GET, Method::HEAD, Method::OPTIONS] {
            assert_ne!(
                oneshot_status(test_app(true), method, &[]).await,
                StatusCode::FORBIDDEN
            );
        }
        // 非 strict では Origin/Referer なしでも通過
        assert_eq!(
            oneshot_status(test_app(false), Method::POST, &[]).await,
            StatusCode::OK
        );
    }

    async fn security_headers_response(secure_cookie: bool) -> axum::response::Response {
        security_headers_app(secure_cookie)
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/test")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn test_security_headers() {
        let resp = security_headers_response(false).await;

        for (name, expected) in [
            ("X-Content-Type-Options", "nosniff"),
            ("X-Frame-Options", "DENY"),
            ("Referrer-Policy", "strict-origin-when-cross-origin"),
            ("Content-Security-Policy", "default-src 'none'"),
        ] {
            assert_eq!(resp.headers().get(name).unwrap(), expected);
        }

        assert!(
            resp.headers().get("Strict-Transport-Security").is_none(),
            "secure cookie 無効時は HSTS を付与しない"
        );

        let resp = security_headers_response(true).await;

        assert_eq!(
            resp.headers().get("Strict-Transport-Security").unwrap(),
            "max-age=31536000; includeSubDomains"
        );
    }

    proptest::proptest! {
        #[test]
        fn prop_extract_origin_is_idempotent_prefix(url in ".*") {
            match extract_origin(&url) {
                None => {
                    proptest::prop_assert!(!url.contains("://"), "url={url}");
                }
                Some(origin) => {
                    proptest::prop_assert!(url.starts_with(origin), "url={url}");
                    proptest::prop_assert!(origin.contains("://"), "origin={origin}");
                    proptest::prop_assert_eq!(extract_origin(origin), Some(origin));
                }
            }
        }

        #[test]
        fn prop_extract_origin_stops_at_authority_end(
            scheme in "https?|ftp|chrome-extension",
            authority in "[a-zA-Z0-9.:-]{1,40}",
            rest in "[/?#].*",
        ) {
            let url = format!("{scheme}://{authority}{rest}");
            let Some(origin) = extract_origin(&url) else {
                panic!("url={url} で origin が取れない");
            };
            let expected = format!("{scheme}://{authority}");
            proptest::prop_assert_eq!(origin, expected.as_str());
        }
    }
}
