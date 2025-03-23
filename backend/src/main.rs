mod errors;
mod extractors;
mod handlers;
mod models;
mod state;

use axum::{
    routing::{get, post},
    Router,
};
use dotenvy::dotenv;
use shuttle_runtime::SecretStore;
use sqlx::postgres::PgPoolOptions;
use state::AppState;
use tower_http::cors::{Any, CorsLayer};

#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres] postgres_connection: String,
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> shuttle_axum::ShuttleAxum {
    dotenv().ok();

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&postgres_connection)
        .await
        .expect("Failed to connect to Postgres");

    // sqlx::migrate!()
    //     .run(&pool)
    //     .await
    //     .expect("Failed to run migrations");

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let state = AppState { pool, secrets };
    let router = Router::new()
        .route("/oauth/google", get(handlers::oauth_google::google_oauth))
        .route("/stock", post(handlers::stock::add_stock_info))
        .route(
            "/stock/:code_or_name",
            get(handlers::stock::select_stock_info),
        )
        .layer(cors)
        .with_state(state);

    Ok(router.into())
}
