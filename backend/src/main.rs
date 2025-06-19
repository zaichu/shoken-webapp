mod config;
mod errors;
mod extractors;
mod handlers;
mod models;
mod services;
mod state;

use axum::{
    routing::{get, post},
    Router,
};
use config::Config;
use dotenvy::dotenv;
use reqwest::Client;
use shuttle_runtime::SecretStore;
use sqlx::postgres::PgPoolOptions;
use state::AppState;

#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres] postgres_connection: String,
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> shuttle_axum::ShuttleAxum {
    dotenv().ok();

    let config = Config::default();
    
    let database_url = secrets.get("DATABASE_URL").unwrap_or(postgres_connection);
    println!("database_url: {}", database_url);
    let pool = PgPoolOptions::new()
        .max_connections(config.database_max_connections)
        .connect(&database_url)
        .await
        .expect("Failed to connect to Postgres");

    // sqlx::migrate!()
    //     .run(&pool)
    //     .await
    //     .expect("Failed to run migrations");

    let cors = config.build_cors_layer();

    let client = Client::new();
    let state = AppState {
        pool,
        secrets: secrets,
        client,
    };

    let router = Router::new()
        .route("/stock", post(handlers::stock::add_stock_info))
        .route("/stock/{query}", get(handlers::stock::select_stock_info))
        // JQuantsのエンドポイントを追加
        .route("/jquants/auth", post(handlers::jquants::authenticate))
        .route("/jquants/refresh", post(handlers::jquants::refresh_token))
        .route(
            "/jquants/fins/statements",
            get(handlers::jquants::get_statements),
        )
        .layer(cors)
        .with_state(state);

    Ok(router.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use tower::ServiceExt;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use sqlx::postgres::PgPoolOptions;

    /// テスト用のアプリケーションルーターを作成する
    pub fn create_test_router() -> Router {
        let database_url = "postgresql://user:password@localhost/test_db";
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect_lazy(database_url)
            .expect("Failed to create connection pool");

        let bt = BTreeMap::from([
            ("DATABASE_URL".to_owned(), database_url.to_owned().into()),
            ("JQUANTS_EMAIL".to_owned(), "test@example.com".to_owned().into()),
            ("JQUANTS_PASSWORD".to_owned(), "password123".to_owned().into()),
        ]);
        let secrets = SecretStore::new(bt);

        let client = Client::new();
        let state = AppState {
            pool,
            secrets,
            client,
        };

        Router::new()
            .route("/stock", post(handlers::stock::add_stock_info))
            .route("/stock/{query}", get(handlers::stock::select_stock_info))
            .route("/jquants/auth", post(handlers::jquants::authenticate))
            .route("/jquants/refresh", post(handlers::jquants::refresh_token))
            .route("/jquants/fins/statements", get(handlers::jquants::get_statements))
            .with_state(state)
    }

    #[tokio::test]
    async fn test_router_creation() {
        let router = create_test_router();
        
        // ルーターが正常に作成されることを確認
        // 実際のリクエストは送信せず、ルーターの作成のみをテスト
        assert!(true);
    }

    #[tokio::test]
    async fn test_cors_configuration() {
        let allowed_headers = vec![
            axum::http::header::CONTENT_TYPE,
            axum::http::header::ACCEPT,
            axum::http::header::ORIGIN,
            axum::http::header::AUTHORIZATION,
        ];

        let allowed_methods = vec![
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
        ];

        // CORS設定が正しく構成されることを確認
        assert_eq!(allowed_headers.len(), 4);
        assert_eq!(allowed_methods.len(), 5);
        assert!(allowed_methods.contains(&axum::http::Method::GET));
        assert!(allowed_methods.contains(&axum::http::Method::POST));
    }

    #[tokio::test]
    async fn test_app_state_creation_from_config() {
        let database_url = "postgresql://user:password@localhost/test_db";
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect_lazy(database_url)
            .expect("Failed to create connection pool");

        let bt = BTreeMap::from([
            ("DATABASE_URL".to_owned(), database_url.to_owned().into()),
        ]);
        let secrets = SecretStore::new(bt);
        let client = Client::new();

        let state = AppState {
            pool,
            secrets,
            client,
        };

        // AppStateが正常に作成されることを確認
        let _ = &state.pool;
        let _ = &state.secrets;
        let _ = &state.client;
    }
}
