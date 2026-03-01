use axum::{
    middleware,
    routing::{delete, get, post},
    Json, Router,
};
use tower_http::limit::RequestBodyLimitLayer;
use utoipa::OpenApi;

use crate::{
    config::Config, handlers, middleware::validate_origin, openapi::ApiDoc, state::AppState,
};

/// リクエストボディの上限サイズ（10MB）
const REQUEST_BODY_LIMIT: usize = 10 * 1024 * 1024;

pub fn app_router(state: AppState, config: &Config) -> Router {
    Router::new()
        .merge(stock_routes())
        .merge(jquants_routes())
        .merge(dividend_per_share_routes())
        .merge(auth_routes())
        .merge(dividend_routes())
        .merge(domestic_stock_routes())
        .merge(mutualfund_routes())
        .merge(asset_balance_routes())
        .route("/health", get(|| async { "OK" }))
        .route(
            "/api-docs/openapi.json",
            get(|| async { Json(ApiDoc::openapi()) }),
        )
        .layer(middleware::from_fn(validate_origin))
        .layer(RequestBodyLimitLayer::new(REQUEST_BODY_LIMIT))
        .layer(config.build_cors_layer())
        .with_state(state)
}

fn stock_routes() -> Router<AppState> {
    Router::new()
        .route("/stock", post(handlers::stock::add_stock_info))
        .route("/stock/{query}", get(handlers::stock::select_stock_info))
}

fn jquants_routes() -> Router<AppState> {
    Router::new().route(
        "/jquants/fins/statements",
        get(handlers::jquants::get_fin_summary),
    )
}

fn dividend_per_share_routes() -> Router<AppState> {
    Router::new().route(
        "/dividends/per-share/batch",
        post(handlers::dividend_per_share::batch),
    )
}

fn auth_routes() -> Router<AppState> {
    Router::new()
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
        )
}

fn dividend_routes() -> Router<AppState> {
    Router::new()
        .route("/dividends", get(handlers::dividend::list))
        .route("/dividends/csv", post(handlers::dividend::upload_csv))
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
            "/domestic-stocks/all",
            delete(handlers::domestic_stock::delete_all),
        )
}

fn mutualfund_routes() -> Router<AppState> {
    Router::new()
        .route("/mutualfunds", get(handlers::mutualfund::list))
        .route("/mutualfunds/csv", post(handlers::mutualfund::upload_csv))
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
            "/asset-balances/all",
            delete(handlers::asset_balance::delete_all),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stock_routes_creation() {
        let _router = stock_routes();
    }

    #[test]
    fn test_jquants_routes_creation() {
        let _router = jquants_routes();
    }

    #[test]
    fn test_auth_routes_creation() {
        let _router = auth_routes();
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
}
