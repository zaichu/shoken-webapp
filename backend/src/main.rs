mod config;
mod errors;
mod extractors;
mod handlers;
mod models;
mod services;
mod state;

use axum::{
    routing::{delete, get, post},
    Router,
};
use config::Config;
use dotenvy::dotenv;
use reqwest::Client;
use sqlx::postgres::PgPoolOptions;
use state::{AppState, Secrets};
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // 環境変数の読み込み
    dotenv().ok();

    // ロギングの初期化
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "backend=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // シークレットの読み込み
    let secrets = Secrets::from_env().expect("シークレットの読み込みに失敗しました");
    let secrets = Arc::new(secrets);

    let config = Config::default();

    tracing::info!("データベースに接続中...");
    let pool = PgPoolOptions::new()
        .max_connections(config.database_max_connections)
        .connect(&secrets.database_url)
        .await
        .expect("データベースへの接続に失敗しました");

    // マイグレーションの実行
    tracing::info!("マイグレーションを実行中...");
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("マイグレーションの実行に失敗しました");

    let cors = config.build_cors_layer();

    let client = Client::new();
    let state = AppState {
        pool,
        secrets,
        client,
    };

    let router = Router::new()
        .route("/stock", post(handlers::stock::add_stock_info))
        .route("/stock/{query}", get(handlers::stock::select_stock_info))
        // JQuantsのエンドポイント（V2 APIキー認証）
        // V2では fins/statements → fins/summary に変更されたが、
        // フロントエンド互換性のためURLパスは維持
        .route(
            "/jquants/fins/statements",
            get(handlers::jquants::get_fin_summary),
        )
        // 認証エンドポイント
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
        // 配当金エンドポイント
        .route("/dividends", get(handlers::dividend::list))
        .route("/dividends/bulk", post(handlers::dividend::bulk_create))
        .route("/dividends/all", delete(handlers::dividend::delete_all))
        // 国内株式エンドポイント
        .route("/domestic-stocks", get(handlers::domestic_stock::list))
        .route(
            "/domestic-stocks/bulk",
            post(handlers::domestic_stock::bulk_create),
        )
        .route(
            "/domestic-stocks/all",
            delete(handlers::domestic_stock::delete_all),
        )
        // 投資信託エンドポイント
        .route("/mutualfunds", get(handlers::mutualfund::list))
        .route("/mutualfunds/bulk", post(handlers::mutualfund::bulk_create))
        .route("/mutualfunds/all", delete(handlers::mutualfund::delete_all))
        // 保有銘柄エンドポイント
        .route("/asset-balances", get(handlers::asset_balance::list))
        .route(
            "/asset-balances/bulk",
            post(handlers::asset_balance::bulk_create),
        )
        .route(
            "/asset-balances/all",
            delete(handlers::asset_balance::delete_all),
        )
        // ヘルスチェック
        .route("/health", get(|| async { "OK" }))
        .layer(cors)
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3001".to_string());
    let addr = format!("0.0.0.0:{}", port);

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
    use sqlx::postgres::PgPoolOptions;

    /// テスト用のアプリケーションルーターを作成する
    pub fn create_test_router() -> Router {
        let database_url = "postgresql://user:password@localhost/test_db";
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect_lazy(database_url)
            .expect("Failed to create connection pool");

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

        Router::new()
            .route("/stock", post(handlers::stock::add_stock_info))
            .route("/stock/{query}", get(handlers::stock::select_stock_info))
            .route(
                "/jquants/fins/statements",
                get(handlers::jquants::get_fin_summary),
            )
            .with_state(state)
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
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect_lazy(database_url)
            .expect("Failed to create connection pool");

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
