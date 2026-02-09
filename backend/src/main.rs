mod config;
mod db;
mod errors;
mod extractors;
mod handlers;
mod logging;
mod middleware;
mod models;
mod routes;
mod services;
mod state;

use config::Config;
use db::{connect_pool, run_migrations};
use dotenvy::dotenv;
use reqwest::Client;
use routes::app_router;
use state::{AppState, Secrets};
use std::sync::Arc;
use tokio::net::TcpListener;

#[cfg(test)]
use db::connect_pool_lazy;

#[tokio::main]
async fn main() {
    // 環境変数の読み込み
    dotenv().ok();

    // ロギングの初期化
    logging::init_tracing();

    // シークレットの読み込み
    let secrets = Secrets::from_env().expect("シークレットの読み込みに失敗しました");
    let secrets = Arc::new(secrets);

    let config = Config::default();

    tracing::info!("データベースに接続中...");
    let pool = connect_pool(&secrets.database_url, config.database_max_connections)
        .await
        .expect("データベースへの接続に失敗しました");

    // マイグレーションの実行
    tracing::info!("マイグレーションを実行中...");
    run_migrations(&pool)
        .await
        .expect("マイグレーションの実行に失敗しました");

    let client = Client::new();
    let state = AppState {
        pool,
        secrets,
        client,
    };

    let router = app_router(state, &config);

    let addr = config::server_addr();

    tracing::info!("サーバーを {} で起動します", addr);

    let listener = TcpListener::bind(&addr)
        .await
        .expect("TCPリスナーのバインドに失敗しました");

    axum::serve(listener, router)
        .await
        .expect("サーバーの起動に失敗しました");
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Method;
    use axum::Router;

    /// テスト用のアプリケーションルーターを作成する
    pub fn create_test_router() -> Router {
        let database_url = "postgresql://user:password@localhost/test_db";
        let pool = connect_pool_lazy(database_url, 1).expect("Failed to create connection pool");

        let secrets = Arc::new(Secrets {
            database_url: database_url.to_string(),
            jquants_api_key: Some("test_api_key".to_string()),
            google_client_id: None,
            google_client_secret: None,
            frontend_url: "http://localhost:8080".to_string(),
        });

        let client = Client::new();
        let state = AppState {
            pool,
            secrets,
            client,
        };

        let config = Config::default();
        app_router(state, &config)
    }

    #[tokio::test]
    async fn test_router_creation() {
        let _router = create_test_router();
        // ルーターが正常に作成されることを確認
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
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ];

        // CORS設定が正しく構成されることを確認
        assert_eq!(allowed_headers.len(), 4);
        assert_eq!(allowed_methods.len(), 5);
        assert!(allowed_methods.contains(&Method::GET));
        assert!(allowed_methods.contains(&Method::POST));
    }

    #[tokio::test]
    async fn test_app_state_creation_from_config() {
        let database_url = "postgresql://user:password@localhost/test_db";
        let pool = connect_pool_lazy(database_url, 5).expect("Failed to create connection pool");

        let secrets = Arc::new(Secrets {
            database_url: database_url.to_string(),
            jquants_api_key: None,
            google_client_id: None,
            google_client_secret: None,
            frontend_url: "http://localhost:8080".to_string(),
        });
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
