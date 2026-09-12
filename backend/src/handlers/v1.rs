use crate::state::AppState;
use axum::{
    routing::{delete, get, post},
    Router,
};

pub mod asset_balances;
pub mod auth;
pub mod dividends;
pub mod domestic_stocks;
pub mod mutual_funds;
pub mod stocks;

/// 認証系 v1 ルート（auth_limiter 対象）
pub fn auth_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/session",
            get(auth::get_session).delete(auth::delete_session),
        )
        .route(
            "/api/v1/account-deletion-confirmations",
            post(auth::create_account_deletion_confirmation),
        )
        .route("/api/v1/account", delete(auth::delete_account))
        .route(
            "/api/v1/oauth/google/authorize",
            get(auth::google_authorize),
        )
        .route("/api/v1/oauth/google/callback", get(auth::google_callback))
}

/// データ系 v1 ルート（レート制限なし）
pub fn data_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/stocks", post(stocks::create))
        .route(
            "/api/v1/dividends",
            get(dividends::list).delete(dividends::delete_all),
        )
        .route(
            "/api/v1/dividend-import-validations",
            post(dividends::validate_import),
        )
        .route(
            "/api/v1/dividend-per-share-estimates",
            post(dividends::estimate_per_share),
        )
        .route(
            "/api/v1/domestic-stock-transactions",
            get(domestic_stocks::list_transactions).delete(domestic_stocks::delete_transactions),
        )
        .route(
            "/api/v1/domestic-stock-import-validations",
            post(domestic_stocks::validate_import),
        )
        .route(
            "/api/v1/mutual-fund-transactions",
            get(mutual_funds::list_transactions).delete(mutual_funds::delete_transactions),
        )
        .route(
            "/api/v1/mutual-fund-import-validations",
            post(mutual_funds::validate_import),
        )
        .route(
            "/api/v1/asset-balances",
            get(asset_balances::list)
                .put(asset_balances::replace)
                .delete(asset_balances::delete_all),
        )
        .route(
            "/api/v1/asset-balance-import-validations",
            post(asset_balances::validate_import),
        )
}

/// 銘柄検索 v1 ルート（stock_search_limiter 対象、未認証）
pub fn stock_search_routes() -> Router<AppState> {
    Router::new().route("/api/v1/stocks", get(stocks::search))
}

/// CSV アップロード系 v1 ルート（csv_limiter 対象）
pub fn csv_upload_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/dividend-imports", post(dividends::import))
        .route(
            "/api/v1/domestic-stock-imports",
            post(domestic_stocks::import),
        )
        .route("/api/v1/mutual-fund-imports", post(mutual_funds::import))
        .route(
            "/api/v1/asset-balance-imports",
            post(asset_balances::import),
        )
}

// テスト
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::state::Secrets,
        axum::{
            body::Body,
            http::{Method, Request, StatusCode},
        },
        std::sync::Arc,
        tower::ServiceExt,
    };

    fn make_test_state() -> AppState {
        let database_url = "postgresql://user:password@localhost/test_db";
        let pool = crate::db::connect_pool_lazy(database_url, 1).expect("pool");
        AppState {
            pool,
            secrets: Arc::new(Secrets {
                database_url: database_url.to_string(),
                jquants_api_key: None,
                google_client_id: None,
                google_client_secret: None,
                frontend_url: "http://localhost:8080".to_string(),
            }),
            client: reqwest::Client::new(),
            dividend_cache: crate::state::DividendCacheState::default(),
        }
    }

    async fn check_status(router: axum::Router, method: Method, uri: &str) -> StatusCode {
        router
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(uri)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap()
            .status()
    }

    /// /api/v1/session が未認証で 401 を返す
    #[tokio::test]
    async fn test_v1_session_returns_401_without_auth() {
        let router = auth_routes().with_state(make_test_state());
        assert_eq!(
            check_status(router, Method::GET, "/api/v1/session").await,
            StatusCode::UNAUTHORIZED
        );
    }

    /// DELETE /api/v1/session は認証不要で 200 を返す（logout と同じ挙動）
    #[tokio::test]
    async fn test_delete_session_returns_200_without_auth() {
        let router = auth_routes().with_state(make_test_state());
        assert_eq!(
            check_status(router, Method::DELETE, "/api/v1/session").await,
            StatusCode::OK
        );
    }

    /// DELETE /api/v1/account はセッションがあっても確認 cookie がない場合 400 を返す
    #[tokio::test]
    async fn test_delete_account_requires_confirmation_cookie() {
        let router = auth_routes().with_state(make_test_state());
        let response = router
            .oneshot(
                Request::builder()
                    .method(Method::DELETE)
                    .uri("/api/v1/account")
                    .header("cookie", format!("session_token={}", uuid::Uuid::new_v4()))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    /// DELETE /api/v1/account は確認 cookie の値が UUID でない場合 400 を返す
    #[tokio::test]
    async fn test_delete_account_rejects_invalid_confirmation_cookie_uuid() {
        let router = auth_routes().with_state(make_test_state());
        let response = router
            .oneshot(
                Request::builder()
                    .method(Method::DELETE)
                    .uri("/api/v1/account")
                    .header(
                        "cookie",
                        format!(
                            "session_token={}; account_delete_confirmation=not-a-uuid",
                            uuid::Uuid::new_v4()
                        ),
                    )
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    /// GET /api/v1/stocks?query=... は未認証でも認証エラーにしない
    #[tokio::test]
    async fn test_v1_stocks_search_allows_anonymous() {
        let router = stock_search_routes().with_state(make_test_state());
        let status = check_status(router, Method::GET, "/api/v1/stocks?query=7203").await;
        assert_ne!(status, StatusCode::UNAUTHORIZED);
        assert_ne!(status, StatusCode::NOT_FOUND);
        assert_ne!(status, StatusCode::METHOD_NOT_ALLOWED);
    }
}
