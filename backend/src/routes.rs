use std::sync::Arc;

use axum::{middleware, routing::get, Json, Router};
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

fn spawn_keyed_limiter_cleanup(
    limiter: &Option<Arc<governor::DefaultKeyedRateLimiter<std::net::IpAddr>>>,
) {
    if let Some(limiter) = limiter {
        let limiter = limiter.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
            loop {
                interval.tick().await;
                limiter.retain_recent();
            }
        });
    }
}

pub fn app_router(state: AppState, config: &Config) -> Router {
    // auth は IP 単位の keyed limiter（ブルートフォース/DoS 対策）
    let auth_limiter = build_keyed_rate_limiter(config.auth_rate_limit_rps);
    let csv_limiter = build_keyed_rate_limiter(config.csv_rate_limit_rps);
    let jquants_limiter = build_rate_limiter(config.jquants_rate_limit_rps);
    spawn_keyed_limiter_cleanup(&auth_limiter);
    spawn_keyed_limiter_cleanup(&csv_limiter);

    let allowed_origins = Arc::new(config.cors_origins.clone());
    domain_routes(jquants_limiter, auth_limiter, csv_limiter)
        .merge(
            Router::new()
                .route("/health", get(|| async { "OK" }))
                .route(
                    "/api-docs/openapi.json",
                    get(|| async { Json(ApiDoc::openapi()) }),
                ),
        )
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

/// ドメインルートをまとめたルーター（認証・コア機能）
fn domain_routes(
    jquants_limiter: Option<Arc<governor::DefaultDirectRateLimiter>>,
    auth_limiter: Option<Arc<governor::DefaultKeyedRateLimiter<std::net::IpAddr>>>,
    csv_limiter: Option<Arc<governor::DefaultKeyedRateLimiter<std::net::IpAddr>>>,
) -> Router<AppState> {
    let jquants_routes = if let Some(l) = jquants_limiter {
        handlers::jquants::jquants_routes().layer(middleware::from_fn(move |req, next| {
            let l = l.clone();
            async move { rate_limit(l, req, next).await }
        }))
    } else {
        handlers::jquants::jquants_routes()
    };
    let auth_routes = if let Some(l) = auth_limiter {
        handlers::auth::auth_routes().layer(middleware::from_fn(move |req, next| {
            let l = l.clone();
            async move { keyed_rate_limit(l, req, next).await }
        }))
    } else {
        handlers::auth::auth_routes()
    };
    let csv_upload_routes = if let Some(l) = csv_limiter {
        csv_upload_routes().layer(middleware::from_fn(move |req, next| {
            let l = l.clone();
            async move { keyed_rate_limit(l, req, next).await }
        }))
    } else {
        csv_upload_routes()
    };

    Router::new()
        .merge(handlers::auth::stock_routes())
        .merge(jquants_routes)
        .merge(handlers::dividend_per_share::dividend_per_share_routes())
        .merge(auth_routes)
        .merge(csv_upload_routes)
        .merge(handlers::dividend::dividend_routes())
        .merge(handlers::domestic_stock::domestic_stock_routes())
        .merge(handlers::mutualfund::mutualfund_routes())
        .merge(handlers::asset_balance::asset_balance_routes())
        .merge(handlers::csv_import::csv_import_routes())
}

fn csv_upload_routes() -> Router<AppState> {
    Router::new()
        .merge(handlers::dividend::dividend_csv_upload_routes())
        .merge(handlers::domestic_stock::domestic_stock_csv_upload_routes())
        .merge(handlers::mutualfund::mutualfund_csv_upload_routes())
        .merge(handlers::asset_balance::asset_balance_csv_upload_routes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_env::{EnvGuard, ENV_MUTEX};
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
            dividend_cache: crate::state::DividendCacheState::default(),
        }
    }

    #[test]
    fn test_stock_routes_creation() {
        let _router = handlers::auth::stock_routes();
    }

    #[test]
    fn test_jquants_routes_creation() {
        let _router = handlers::jquants::jquants_routes();
    }

    #[test]
    fn test_auth_routes_creation() {
        let _router = handlers::auth::auth_routes();
    }

    #[test]
    fn test_dividend_routes_creation() {
        let _router = handlers::dividend::dividend_routes();
    }

    #[test]
    fn test_domestic_stock_routes_creation() {
        let _router = handlers::domestic_stock::domestic_stock_routes();
    }

    #[test]
    fn test_mutualfund_routes_creation() {
        let _router = handlers::mutualfund::mutualfund_routes();
    }

    #[test]
    fn test_asset_balance_routes_creation() {
        let _router = handlers::asset_balance::asset_balance_routes();
    }

    /// auth ルートが rps=1 制限を超えると 429 を返すことを確認
    #[tokio::test]
    async fn test_auth_routes_rate_limit_returns_429() {
        use axum::{body::Body, http::Request};
        use tower::ServiceExt;

        let limiter = crate::middleware::build_keyed_rate_limiter(1);
        let state = make_test_state();
        let router = if let Some(l) = limiter {
            handlers::auth::auth_routes().layer(middleware::from_fn(move |req, next| {
                let l = l.clone();
                async move { keyed_rate_limit(l, req, next).await }
            }))
        } else {
            handlers::auth::auth_routes()
        }
        .with_state(state);

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
        let router = if let Some(l) = limiter {
            handlers::jquants::jquants_routes().layer(middleware::from_fn(move |req, next| {
                let l = l.clone();
                async move { rate_limit(l, req, next).await }
            }))
        } else {
            handlers::jquants::jquants_routes()
        }
        .with_state(state);

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

    /// CSV upload ルートだけが IP 単位レート制限の対象になることを確認
    #[tokio::test]
    async fn test_csv_upload_routes_rate_limit_returns_429() {
        use axum::{body::Body, http::Request};
        use tower::ServiceExt;

        let _lock = ENV_MUTEX.lock().await;
        let _csv_rate_limit_rps = EnvGuard::set("CSV_RATE_LIMIT_RPS", Some("1"));

        for (path, ip) in [
            ("/domestic-stocks/csv", "1.2.3.4"),
            ("/dividends/csv", "1.2.3.5"),
            ("/mutualfunds/csv", "1.2.3.6"),
            ("/asset-balances/csv", "1.2.3.7"),
        ] {
            let state = make_test_state();
            let config = Config::from_env();
            let router = app_router(state, &config);

            let req = Request::builder()
                .method(axum::http::Method::POST)
                .uri(path)
                .header("fly-client-ip", ip)
                .body(Body::empty())
                .unwrap();
            let resp = router.clone().oneshot(req).await.unwrap();
            assert_ne!(resp.status(), axum::http::StatusCode::TOO_MANY_REQUESTS);

            let req = Request::builder()
                .method(axum::http::Method::POST)
                .uri(path)
                .header("fly-client-ip", ip)
                .body(Body::empty())
                .unwrap();
            let resp = router.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), axum::http::StatusCode::TOO_MANY_REQUESTS);
        }

        let state = make_test_state();
        let config = Config::from_env();
        let router = app_router(state, &config);

        let req = Request::builder()
            .method(axum::http::Method::POST)
            .uri("/domestic-stocks/csv/preview")
            .header("fly-client-ip", "1.2.3.4")
            .body(Body::empty())
            .unwrap();
        let resp = router.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), axum::http::StatusCode::TOO_MANY_REQUESTS);
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
