mod errors;
mod extractors;
mod handlers;
mod models;
mod state;

use axum::{
    http::HeaderValue,
    routing::{get, post},
    Router,
};
use dotenvy::dotenv;
use shuttle_runtime::SecretStore;
use sqlx::postgres::PgPoolOptions;
use state::AppState;
use tower_http::cors::CorsLayer;

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

    let allowed_headers = vec![
        axum::http::header::CONTENT_TYPE,
        axum::http::header::ACCEPT,
        axum::http::header::ORIGIN,
        axum::http::header::AUTHORIZATION,
    ];

    // 許可するメソッドリストを定義
    let allowed_methods = vec![
        axum::http::Method::GET,
        axum::http::Method::POST,
        axum::http::Method::PUT,
        axum::http::Method::DELETE,
        axum::http::Method::OPTIONS,
    ];

    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::AllowOrigin::predicate(|origin, _| {
            origin.eq(&"https://zaichu.github.io".parse::<HeaderValue>().unwrap())
                || origin.eq(&"http://localhost:8080".parse::<HeaderValue>().unwrap())
        }))
        .allow_methods(allowed_methods)
        .allow_headers(allowed_headers)
        .allow_credentials(true);

    let state = AppState {
        pool,
        secrets: secrets,
    };
    let router = Router::new()
        .route("/stock", post(handlers::stock::add_stock_info))
        .route("/stock/{query}", get(handlers::stock::select_stock_info))
        .layer(cors)
        .with_state(state);

    Ok(router.into())
}
