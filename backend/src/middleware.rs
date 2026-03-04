use std::sync::Arc;

use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};

use crate::errors::{ErrorDetails, ErrorResponse};

/// Origin検証ミドルウェア（CSRF対策）
/// POST/PUT/DELETE リクエストに対して Origin ヘッダーを検証し、
/// 許可されたオリジンからのリクエストのみ通過させる。
/// allowed_origins は Config::cors_origins と一致させる。
pub async fn validate_origin(
    allowed_origins: Arc<Vec<String>>,
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
        Some(ref o) if allowed_origins.iter().any(|a| a == o) => {
            // 許可されたオリジン → 通過
            next.run(request).await
        }
        None => {
            // Origin なし（同一オリジンリクエストまたはcurlなど）→ 通過
            // ブラウザはクロスオリジンPOSTに必ずOriginヘッダーを付与するため、
            // Originなしは同一オリジンまたは非ブラウザクライアント
            next.run(request).await
        }
        Some(_) => {
            // 不正なオリジン → 拒否
            let error_response = ErrorResponse {
                error: ErrorDetails {
                    code: "CSRF_ERROR".to_string(),
                    message: "不正なリクエスト元です".to_string(),
                    details: None,
                },
            };
            (StatusCode::FORBIDDEN, Json(error_response)).into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request, middleware, routing::post, Router};
    use tower::ServiceExt;

    fn test_app() -> Router {
        let allowed_origins = Arc::new(vec![
            "https://shoken-webapp.vercel.app".to_string(),
            "http://localhost:8080".to_string(),
            "http://127.0.0.1:8080".to_string(),
            "http://[::1]:8080".to_string(),
            "http://localhost.:8080".to_string(),
        ]);
        Router::new()
            .route("/test", post(|| async { "ok" }))
            .layer(middleware::from_fn(move |req, next| {
                let origins = allowed_origins.clone();
                async move { validate_origin(origins, req, next).await }
            }))
    }

    #[tokio::test]
    async fn test_get_request_passes() {
        let app = test_app();
        let req = Request::builder()
            .method(Method::GET)
            .uri("/test")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        // GET はルート定義がないので 405 だが、ミドルウェアは通過
        assert_ne!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_post_with_allowed_origin() {
        let app = test_app();
        let req = Request::builder()
            .method(Method::POST)
            .uri("/test")
            .header("origin", "http://localhost:8080")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_post_with_disallowed_origin() {
        let app = test_app();
        let req = Request::builder()
            .method(Method::POST)
            .uri("/test")
            .header("origin", "https://evil.example.com")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_post_without_origin() {
        let app = test_app();
        let req = Request::builder()
            .method(Method::POST)
            .uri("/test")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        // Origin なしは同一オリジンとみなし通過
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_delete_with_disallowed_origin() {
        let allowed_origins = Arc::new(vec!["http://localhost:8080".to_string()]);
        let app = Router::new()
            .route("/test", axum::routing::delete(|| async { "ok" }))
            .layer(middleware::from_fn(move |req, next| {
                let origins = allowed_origins.clone();
                async move { validate_origin(origins, req, next).await }
            }));

        let req = Request::builder()
            .method(Method::DELETE)
            .uri("/test")
            .header("origin", "https://evil.example.com")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_post_with_production_origin() {
        let app = test_app();
        let req = Request::builder()
            .method(Method::POST)
            .uri("/test")
            .header("origin", "https://shoken-webapp.vercel.app")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
