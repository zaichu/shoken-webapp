use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use axum::{http::StatusCode, middleware, response::IntoResponse, routing::get, Json, Router};
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

pub fn app_router(state: AppState, config: &Config, startup_ready: Arc<AtomicBool>) -> Router {
    // auth は IP 単位の keyed limiter（ブルートフォース/DoS 対策）
    let auth_limiter = build_keyed_rate_limiter(config.auth_rate_limit_rps);
    let csv_limiter = build_keyed_rate_limiter(config.csv_rate_limit_rps);
    let market_data_limiter = build_rate_limiter(config.market_data_rate_limit_rps);
    spawn_keyed_limiter_cleanup(&auth_limiter);
    spawn_keyed_limiter_cleanup(&csv_limiter);

    let allowed_origins = Arc::new(config.cors_origins.clone());
    let ready = Arc::clone(&startup_ready);
    let gated_domain =
        domain_routes(market_data_limiter, auth_limiter, csv_limiter).layer(middleware::from_fn(
            move |req: axum::extract::Request, next: axum::middleware::Next| {
                let ready = Arc::clone(&ready);
                async move {
                    if ready.load(Ordering::Acquire) {
                        next.run(req).await
                    } else {
                        StatusCode::SERVICE_UNAVAILABLE.into_response()
                    }
                }
            },
        ));

    // /ready と /health は TraceLayer の対象外にする（startup 中の expected 503 を ERROR ログから除外）
    let ready_for_check = Arc::clone(&startup_ready);
    let probe_routes = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route(
            "/ready",
            get(move || {
                let ready = Arc::clone(&ready_for_check);
                async move {
                    if ready.load(Ordering::Acquire) {
                        StatusCode::OK.into_response()
                    } else {
                        StatusCode::SERVICE_UNAVAILABLE.into_response()
                    }
                }
            }),
        );

    // リクエストトレース（パスのみ記録：クエリパラメータの機密情報漏洩を防ぐ）
    let traced_inner = gated_domain
        .merge(Router::new().route(
            "/api-docs/openapi.json",
            get(|| async { Json(ApiDoc::openapi()) }),
        ))
        .layer(TraceLayer::new_for_http().make_span_with(PathOnlyMakeSpan));

    traced_inner
        .merge(probe_routes)
        .layer(middleware::from_fn(move |req, next| {
            let origins = allowed_origins.clone();
            async move { validate_origin(origins, req, next).await }
        }))
        .layer(RequestBodyLimitLayer::new(REQUEST_BODY_LIMIT))
        .layer(build_cors_layer(&config.cors_origins))
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
    market_data_limiter: Option<Arc<governor::DefaultDirectRateLimiter>>,
    auth_limiter: Option<Arc<governor::DefaultKeyedRateLimiter<std::net::IpAddr>>>,
    csv_limiter: Option<Arc<governor::DefaultKeyedRateLimiter<std::net::IpAddr>>>,
) -> Router<AppState> {
    let market_data_routes = if let Some(l) = market_data_limiter {
        handlers::v1::market_data_routes().layer(middleware::from_fn(move |req, next| {
            let l = l.clone();
            async move { rate_limit(l, req, next).await }
        }))
    } else {
        handlers::v1::market_data_routes()
    };
    let auth_routes = if let Some(l) = auth_limiter {
        handlers::v1::auth_routes().layer(middleware::from_fn(move |req, next| {
            let l = l.clone();
            async move { keyed_rate_limit(l, req, next).await }
        }))
    } else {
        handlers::v1::auth_routes()
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
        .merge(market_data_routes)
        .merge(auth_routes)
        .merge(csv_upload_routes)
        .merge(handlers::csv_import::csv_import_routes())
        .merge(handlers::v1::data_routes())
}

fn csv_upload_routes() -> Router<AppState> {
    handlers::v1::csv_upload_routes()
}
#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::test_env::{EnvGuard, ENV_MUTEX},
        axum::{
            body::Body,
            http::{Method, Request},
        },
        std::sync::{atomic::AtomicBool, Arc},
        tower::ServiceExt,
    };
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
    fn test_all_routes_creation() {
        let _ = handlers::v1::auth_routes();
        let _ = handlers::v1::market_data_routes();
        let _ = handlers::v1::data_routes();
        let _ = handlers::v1::csv_upload_routes();
        let _ = handlers::csv_import::csv_import_routes();
    }
    #[tokio::test]
    async fn test_routes_rate_limit_returns_429() {
        let router = if let Some(l) = crate::middleware::build_keyed_rate_limiter(1) {
            handlers::v1::auth_routes().layer(middleware::from_fn(move |req, next| {
                let l = l.clone();
                async move { keyed_rate_limit(l, req, next).await }
            }))
        } else {
            handlers::v1::auth_routes()
        }
        .with_state(make_test_state());
        assert_rate_limited(router, Method::GET, "/api/v1/session", Some("1.2.3.4")).await;
        let router = if let Some(l) = crate::middleware::build_rate_limiter(1) {
            handlers::v1::market_data_routes().layer(middleware::from_fn(move |req, next| {
                let l = l.clone();
                async move { rate_limit(l, req, next).await }
            }))
        } else {
            handlers::v1::market_data_routes()
        }
        .with_state(make_test_state());
        assert_rate_limited(router, Method::GET, "/api/v1/financial-statements", None).await;
    }
    async fn assert_rate_limited(
        router: axum::Router,
        method: Method,
        uri: &str,
        ip: Option<&str>,
    ) {
        let make_req = || {
            let mut b = Request::builder().method(method.clone()).uri(uri);
            if let Some(ip) = ip {
                b = b.header("fly-client-ip", ip);
            }
            b.body(Body::empty()).unwrap()
        };
        let resp = router.clone().oneshot(make_req()).await.unwrap();
        assert_ne!(resp.status(), axum::http::StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(
            router.oneshot(make_req()).await.unwrap().status(),
            axum::http::StatusCode::TOO_MANY_REQUESTS
        );
    }
    async fn check_unauthorized(router: axum::Router, method: Method, uri: &str) {
        assert_eq!(
            router
                .oneshot(
                    Request::builder()
                        .method(method)
                        .uri(uri)
                        .body(Body::empty())
                        .unwrap()
                )
                .await
                .unwrap()
                .status(),
            axum::http::StatusCode::UNAUTHORIZED
        );
    }
    #[tokio::test]
    async fn test_all_endpoints_require_auth() {
        for (router, method, uri) in [
            (
                handlers::v1::market_data_routes(),
                Method::GET,
                "/api/v1/financial-statements?code=7203",
            ),
            (
                handlers::v1::data_routes(),
                Method::DELETE,
                "/api/v1/dividends",
            ),
            (
                handlers::v1::data_routes(),
                Method::GET,
                "/api/v1/dividends",
            ),
            (
                handlers::v1::data_routes(),
                Method::POST,
                "/api/v1/dividend-import-validations",
            ),
            (
                handlers::v1::data_routes(),
                Method::DELETE,
                "/api/v1/domestic-stock-transactions",
            ),
            (
                handlers::v1::data_routes(),
                Method::GET,
                "/api/v1/domestic-stock-transactions",
            ),
            (
                handlers::v1::data_routes(),
                Method::POST,
                "/api/v1/domestic-stock-import-validations",
            ),
            (
                handlers::v1::data_routes(),
                Method::DELETE,
                "/api/v1/mutual-fund-transactions",
            ),
            (
                handlers::v1::data_routes(),
                Method::GET,
                "/api/v1/mutual-fund-transactions",
            ),
            (
                handlers::v1::data_routes(),
                Method::POST,
                "/api/v1/mutual-fund-import-validations",
            ),
            (
                handlers::v1::data_routes(),
                Method::DELETE,
                "/api/v1/asset-balances",
            ),
            (
                handlers::v1::data_routes(),
                Method::PUT,
                "/api/v1/asset-balances",
            ),
            (
                handlers::v1::data_routes(),
                Method::POST,
                "/api/v1/asset-balance-import-validations",
            ),
            (
                handlers::v1::csv_upload_routes(),
                Method::POST,
                "/api/v1/dividend-imports",
            ),
            (
                handlers::v1::data_routes(),
                Method::POST,
                "/api/v1/dividend-per-share-estimates",
            ),
        ] {
            check_unauthorized(router.with_state(make_test_state()), method, uri).await;
        }
    }
    #[tokio::test]
    async fn test_csv_upload_routes_rate_limit_returns_429() {
        let _lock = ENV_MUTEX.lock().await;
        let _csv_rate_limit_rps = EnvGuard::set("CSV_RATE_LIMIT_RPS", Some("1"));
        for (path, ip) in [
            ("/api/v1/domestic-stock-imports", "1.2.3.4"),
            ("/api/v1/dividend-imports", "1.2.3.5"),
            ("/api/v1/mutual-fund-imports", "1.2.3.6"),
            ("/api/v1/asset-balance-imports", "1.2.3.7"),
        ] {
            assert_rate_limited(
                app_router(
                    make_test_state(),
                    &Config::from_env(),
                    Arc::new(AtomicBool::new(true)),
                ),
                Method::POST,
                path,
                Some(ip),
            )
            .await;
        }
        let resp = app_router(
            make_test_state(),
            &Config::from_env(),
            Arc::new(AtomicBool::new(true)),
        )
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/domestic-stock-import-validations")
                .header("fly-client-ip", "1.2.3.4")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
        assert_ne!(resp.status(), axum::http::StatusCode::TOO_MANY_REQUESTS);
    }
    #[tokio::test]
    async fn test_ready_returns_503_during_startup() {
        let startup_ready = Arc::new(AtomicBool::new(false));
        let router = app_router(
            make_test_state(),
            &Config::from_env(),
            Arc::clone(&startup_ready),
        );
        let resp = router
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/ready")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), axum::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_ready_returns_200_after_startup() {
        let startup_ready = Arc::new(AtomicBool::new(true));
        let router = app_router(
            make_test_state(),
            &Config::from_env(),
            Arc::clone(&startup_ready),
        );
        let resp = router
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/ready")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), axum::http::StatusCode::OK);
    }

    #[tokio::test]
    async fn test_health_returns_200_during_startup() {
        let startup_ready = Arc::new(AtomicBool::new(false));
        let router = app_router(
            make_test_state(),
            &Config::from_env(),
            Arc::clone(&startup_ready),
        );
        let resp = router
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), axum::http::StatusCode::OK);
    }

    #[tokio::test]
    async fn test_domain_route_returns_503_during_startup() {
        let startup_ready = Arc::new(AtomicBool::new(false));
        let router = app_router(
            make_test_state(),
            &Config::from_env(),
            Arc::clone(&startup_ready),
        );
        let resp = router
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/session")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), axum::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_domain_route_returns_401_after_startup() {
        let startup_ready = Arc::new(AtomicBool::new(true));
        let router = app_router(
            make_test_state(),
            &Config::from_env(),
            Arc::clone(&startup_ready),
        );
        let resp = router
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/session")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), axum::http::StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_legacy_paths_return_404() {
        let startup_ready = Arc::new(AtomicBool::new(true));
        let router = app_router(
            make_test_state(),
            &Config::from_env(),
            Arc::clone(&startup_ready),
        );
        for (method, uri) in [
            (Method::GET, "/auth/me"),
            (Method::GET, "/jquants/fins/summary"),
            (Method::GET, "/dividends"),
            (Method::GET, "/domestic-stocks"),
            (Method::GET, "/mutualfunds"),
            (Method::GET, "/asset-balances"),
        ] {
            let resp = router
                .clone()
                .oneshot(
                    Request::builder()
                        .method(method.clone())
                        .uri(uri)
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(
                resp.status(),
                axum::http::StatusCode::NOT_FOUND,
                "{method} {uri} should return 404"
            );
        }
    }

    #[tokio::test]
    async fn test_request_id_propagated_to_response() {
        use {
            axum::{routing::get, Router},
            tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
        };
        let router: Router = Router::new()
            .route("/health", get(|| async { "OK" }))
            .layer(PropagateRequestIdLayer::x_request_id())
            .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid));
        let health_req = |id: Option<&str>| {
            let mut b = Request::builder().method(Method::GET).uri("/health");
            if let Some(id) = id {
                b = b.header("x-request-id", id);
            }
            b.body(Body::empty()).unwrap()
        };
        assert!(
            router
                .clone()
                .oneshot(health_req(None))
                .await
                .unwrap()
                .headers()
                .contains_key("x-request-id"),
            "x-request-id should be auto-generated"
        );
        assert_eq!(
            router
                .oneshot(health_req(Some("my-custom-id")))
                .await
                .unwrap()
                .headers()
                .get("x-request-id")
                .and_then(|v| v.to_str().ok()),
            Some("my-custom-id"),
            "existing x-request-id should be propagated unchanged"
        );
    }
}
