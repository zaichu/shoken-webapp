mod config;
mod db;
mod errors;
mod extractors;
mod handlers;
mod logging;
mod middleware;
mod models;
mod openapi;
mod routes;
mod services;
mod state;

#[cfg(test)]
mod test_env;

use config::Config;
use db::{connect_pool, run_migrations};
use dotenvy::dotenv;
use reqwest::Client;
use routes::app_router;
use state::{AppState, Secrets};
use std::sync::{atomic::AtomicBool, Arc};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    // 環境変数の読み込み
    dotenv().ok();

    let _sentry_guard = std::env::var("SENTRY_DSN")
        .ok()
        .filter(|dsn| !dsn.is_empty())
        .map(|dsn| {
            sentry::init((
                dsn,
                sentry::ClientOptions {
                    release: sentry::release_name!(),
                    ..Default::default()
                },
            ))
        });

    // ロギングの初期化
    logging::init_tracing();

    // シークレットの読み込み
    let secrets = Secrets::from_env().expect("シークレットの読み込みに失敗しました");
    let secrets = Arc::new(secrets);

    let config = Config::from_env();

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
        background_task_running: Arc::new(AtomicBool::new(false)),
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
