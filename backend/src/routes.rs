use std::sync::Arc;

use axum::{
    middleware,
    routing::{delete, get, post},
    Json, Router,
};
use tower_http::{
    limit::RequestBodyLimitLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::TraceLayer,
};
use utoipa::OpenApi;

use crate::{
    config::{build_cors_layer, Config},
    handlers,
    middleware::{
        add_security_headers, build_keyed_rate_limiter, build_rate_limiter, keyed_rate_limit,
        rate_limit, validate_origin, PathOnlyMakeSpan,
    },
    openapi::ApiDoc,
    state::AppState,
};

/// リクエストボディの上限サイズ（10MB）
const REQUEST_BODY_LIMIT: usize = 10 * 1024 * 1024;

pub fn app_router(state: AppState, config: &Config) -> Router {
    // auth は IP 単位の keyed limiter（ブルートフォース/DoS 対策）
    let auth_limiter = build_keyed_rate_limiter(config.auth_rate_limit_rps);
    let jquants_limiter = build_rate_limiter(config.jquants_rate_limit_rps);
    if let Some(ref limiter) = auth_limiter {
        spawn_rate_limiter_cleanup(limiter.clone());
    }

    let allowed_origins = Arc::new(config.cors_origins.clone());
    domain_routes(jquants_limiter, auth_limiter)
        .merge(utility_routes())
        .layer(middleware::from_fn(move |req, next| {
            let origins = allowed_origins.clone();
            async move { validate_origin(origins, req, next).await }
        }))
        .layer(RequestBodyLimitLayer::new(REQUEST_BODY_LIMIT))
        .layer(build_cors_layer(&config.cors_origins))
        // リクエストトレース（パスのみ記録：クエリパラメータの機密情報漏洩を防ぐ）
        .layer(TraceLayer::new_for_http().make_span_with(PathOnlyMakeSpan))
        // x-request-id をレスポンスに伝播
        .layer(PropagateRequestIdLayer::x_request_id())
        // x-request-id が未設定の場合は UUID v4 を自動付与
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        // セキュリティヘッダーは最外層: 403/413 を含む全レスポンスに付与する
        .layer(middleware::from_fn(add_security_headers))
        .with_state(state)
}

/// keyed limiter のキー増加を抑制するため、60 秒ごとに retain_recent を実行
fn spawn_rate_limiter_cleanup(limiter: Arc<governor::DefaultKeyedRateLimiter<std::net::IpAddr>>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            limiter.retain_recent();
        }
    });
}

/// ドメインルートをまとめたルーター（認証・コア機能）
fn domain_routes(
    jquants_limiter: Option<Arc<governor::DefaultDirectRateLimiter>>,
    auth_limiter: Option<Arc<governor::DefaultKeyedRateLimiter<std::net::IpAddr>>>,
) -> Router<AppState> {
    Router::new()
        .merge(stock_routes())
        .merge(jquants_routes(jquants_limiter))
        .merge(dividend_per_share_routes())
        .merge(auth_routes(auth_limiter))
        .merge(dividend_routes())
        .merge(domestic_stock_routes())
        .merge(mutualfund_routes())
        .merge(asset_balance_routes())
}

/// ヘルスチェックと API ドキュメントのルーター
fn utility_routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(|| async { "OK" }))
        .route(
            "/api-docs/openapi.json",
            get(|| async { Json(ApiDoc::openapi()) }),
        )
}

fn stock_routes() -> Router<AppState> {
    Router::new()
        .route("/stock", post(handlers::stock::add_stock_info))
        .route("/stock/{query}", get(handlers::stock::select_stock_info))
}

fn jquants_routes(limiter: Option<Arc<governor::DefaultDirectRateLimiter>>) -> Router<AppState> {
    let router = Router::new().route(
        "/jquants/fins/statements",
        get(handlers::jquants::get_fin_summary),
    );
    if let Some(l) = limiter {
        router.layer(middleware::from_fn(move |req, next| {
            let l = l.clone();
            async move { rate_limit(l, req, next).await }
        }))
    } else {
        router
    }
}

fn dividend_per_share_routes() -> Router<AppState> {
    Router::new().route(
        "/dividends/per-share/batch",
        post(handlers::dividend_per_share::batch),
    )
}

fn auth_routes(
    limiter: Option<Arc<governor::DefaultKeyedRateLimiter<std::net::IpAddr>>>,
) -> Router<AppState> {
    let router = Router::new()
        .route("/auth/google", get(handlers::auth::google_auth))
        .route(
            "/auth/google/callback",
            get(handlers::auth::google_callback),
        )
        .route("/auth/me", get(handlers::auth::get_current_user))
        .route("/auth/logout", post(handlers::auth::logout))
        .route(
            "/auth/delete-account",
            delete(handlers::auth::delete_account),
        );
    if let Some(l) = limiter {
        router.layer(middleware::from_fn(move |req, next| {
            let l = l.clone();
            async move { keyed_rate_limit(l, req, next).await }
        }))
    } else {
        router
    }
}

fn dividend_routes() -> Router<AppState> {
    Router::new()
        .route("/dividends", get(handlers::dividend::list))
        .route("/dividends/csv", post(handlers::dividend::upload_csv))
        .route(
            "/dividends/csv/preview",
            post(handlers::dividend::preview_csv),
        )
        .route("/dividends/all", delete(handlers::dividend::delete_all))
}

fn domestic_stock_routes() -> Router<AppState> {
    Router::new()
        .route("/domestic-stocks", get(handlers::domestic_stock::list))
        .route(
            "/domestic-stocks/csv",
            post(handlers::domestic_stock::upload_csv),
        )
        .route(
            "/domestic-stocks/csv/preview",
            post(handlers::domestic_stock::preview_csv),
        )
        .route(
            "/domestic-stocks/all",
            delete(handlers::domestic_stock::delete_all),
        )
}

fn mutualfund_routes() -> Router<AppState> {
    Router::new()
        .route("/mutualfunds", get(handlers::mutualfund::list))
        .route("/mutualfunds/csv", post(handlers::mutualfund::upload_csv))
        .route(
            "/mutualfunds/csv/preview",
            post(handlers::mutualfund::preview_csv),
        )
        .route("/mutualfunds/all", delete(handlers::mutualfund::delete_all))
}

fn asset_balance_routes() -> Router<AppState> {
    Router::new()
        .route("/asset-balances", get(handlers::asset_balance::list))
        .route(
            "/asset-balances/bulk",
            post(handlers::asset_balance::bulk_create),
        )
        .route(
            "/asset-balances/csv",
            post(handlers::asset_balance::upload_csv),
        )
        .route(
            "/asset-balances/csv/preview",
            post(handlers::asset_balance::preview_csv),
        )
        .route(
            "/asset-balances/all",
            delete(handlers::asset_balance::delete_all),
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn make_test_state() -> AppState {
        let database_url = "postgresql://user:password@localhost/test_db";
        let pool = crate::db::connect_pool_lazy(database_url, 1).expect("pool");
        let secrets = Arc::new(crate::state::Secrets {
            database_url: database_url.to_string(),
            jquants_api_key: None,
            google_client_id: None,
            google_client_secret: None,
            frontend_url: "http://localhost:8080".to_string(),
        });
        AppState {
            pool,
            secrets,
            client: reqwest::Client::new(),
            background_task_running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    #[test]
    fn test_stock_routes_creation() {
        let _router = stock_routes();
    }

    #[test]
    fn test_jquants_routes_creation() {
        let _router = jquants_routes(None);
    }

    #[test]
    fn test_auth_routes_creation() {
        let _router = auth_routes(None);
    }

    #[test]
    fn test_dividend_routes_creation() {
        let _router = dividend_routes();
    }

    #[test]
    fn test_domestic_stock_routes_creation() {
        let _router = domestic_stock_routes();
    }

    #[test]
    fn test_mutualfund_routes_creation() {
        let _router = mutualfund_routes();
    }

    #[test]
    fn test_asset_balance_routes_creation() {
        let _router = asset_balance_routes();
    }

    /// auth ルートが rps=1 制限を超えると 429 を返すことを確認
    #[tokio::test]
    async fn test_auth_routes_rate_limit_returns_429() {
        use axum::{body::Body, http::Request};
        use tower::ServiceExt;

        let limiter = crate::middleware::build_keyed_rate_limiter(1);
        let state = make_test_state();
        let router = auth_routes(limiter).with_state(state);

        // 1 回目は通過（ルートが見つからず 200/401/500 になるが 429 ではない）
        let req = Request::builder()
            .method(axum::http::Method::GET)
            .uri("/auth/me")
            .header("fly-client-ip", "1.2.3.4")
            .body(Body::empty())
            .unwrap();
        let resp = router.clone().oneshot(req).await.unwrap();
        assert_ne!(resp.status(), axum::http::StatusCode::TOO_MANY_REQUESTS);

        // 2 回目（同一 IP）は 429
        let req = Request::builder()
            .method(axum::http::Method::GET)
            .uri("/auth/me")
            .header("fly-client-ip", "1.2.3.4")
            .body(Body::empty())
            .unwrap();
        let resp = router.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), axum::http::StatusCode::TOO_MANY_REQUESTS);
    }

    /// jquants ルートが rps=1 制限を超えると 429 を返すことを確認
    #[tokio::test]
    async fn test_jquants_routes_rate_limit_returns_429() {
        use axum::{body::Body, http::Request};
        use tower::ServiceExt;

        let limiter = crate::middleware::build_rate_limiter(1);
        let state = make_test_state();
        let router = jquants_routes(limiter).with_state(state);

        // 1 回目は通過
        let req = Request::builder()
            .method(axum::http::Method::GET)
            .uri("/jquants/fins/statements")
            .body(Body::empty())
            .unwrap();
        let resp = router.clone().oneshot(req).await.unwrap();
        assert_ne!(resp.status(), axum::http::StatusCode::TOO_MANY_REQUESTS);

        // 2 回目は 429
        let req = Request::builder()
            .method(axum::http::Method::GET)
            .uri("/jquants/fins/statements")
            .body(Body::empty())
            .unwrap();
        let resp = router.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), axum::http::StatusCode::TOO_MANY_REQUESTS);
    }

    /// x-request-id がレスポンスに伝播されることを確認
    #[tokio::test]
    async fn test_request_id_propagated_to_response() {
        use axum::{body::Body, http::Request, routing::get, Router};
        use tower::ServiceExt;
        use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};

        let router: Router = Router::new()
            .route("/health", get(|| async { "OK" }))
            .layer(PropagateRequestIdLayer::x_request_id())
            .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid));

        // x-request-id ヘッダーなし → 自動生成されてレスポンスに付与される
        let req = Request::builder()
            .method(axum::http::Method::GET)
            .uri("/health")
            .body(Body::empty())
            .unwrap();
        let resp = router.clone().oneshot(req).await.unwrap();
        assert!(
            resp.headers().contains_key("x-request-id"),
            "x-request-id should be auto-generated"
        );

        // x-request-id ヘッダーあり → 既存値がそのままレスポンスに伝播される
        let req = Request::builder()
            .method(axum::http::Method::GET)
            .uri("/health")
            .header("x-request-id", "my-custom-id")
            .body(Body::empty())
            .unwrap();
        let resp = router.oneshot(req).await.unwrap();
        assert_eq!(
            resp.headers()
                .get("x-request-id")
                .and_then(|v| v.to_str().ok()),
            Some("my-custom-id"),
            "existing x-request-id should be propagated unchanged"
        );
    }
}
