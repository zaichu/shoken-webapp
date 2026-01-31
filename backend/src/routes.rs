use axum::{
    routing::{delete, get, post},
    Router,
};

use crate::{config::Config, handlers, state::AppState};

pub fn app_router(state: AppState, config: &Config) -> Router {
    Router::new()
        .merge(stock_routes())
        .merge(jquants_routes())
        .merge(auth_routes())
        .merge(dividend_routes())
        .merge(domestic_stock_routes())
        .merge(mutualfund_routes())
        .merge(asset_balance_routes())
        .route("/health", get(|| async { "OK" }))
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
        .route("/dividends/bulk", post(handlers::dividend::bulk_create))
        .route("/dividends/all", delete(handlers::dividend::delete_all))
}

fn domestic_stock_routes() -> Router<AppState> {
    Router::new()
        .route("/domestic-stocks", get(handlers::domestic_stock::list))
        .route(
            "/domestic-stocks/bulk",
            post(handlers::domestic_stock::bulk_create),
        )
        .route(
            "/domestic-stocks/all",
            delete(handlers::domestic_stock::delete_all),
        )
}

fn mutualfund_routes() -> Router<AppState> {
    Router::new()
        .route("/mutualfunds", get(handlers::mutualfund::list))
        .route("/mutualfunds/bulk", post(handlers::mutualfund::bulk_create))
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
