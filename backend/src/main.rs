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
use db::{connect_pool_lazy, connect_pool_with_retry, run_migrations};
use dotenvy::dotenv;
use reqwest::Client;
use routes::app_router;
use state::{AppState, DividendCacheState, Secrets};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Instant;
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

    let startup_ready = Arc::new(AtomicBool::new(false));

    // URL 検証のみ行い、実接続は行わない lazy pool
    let pool = connect_pool_lazy(&secrets.database_url, config.database_max_connections)
        .unwrap_or_else(|e| {
            tracing::error!("{e}");
            std::process::exit(1);
        });

    let client = Client::new();
    let state = AppState {
        pool,
        secrets: Arc::clone(&secrets),
        client,
        dividend_cache: DividendCacheState::default(),
    };

    let router = app_router(state, &config, Arc::clone(&startup_ready));

    let addr = config::server_addr();

    let t0 = Instant::now();

    let listener = TcpListener::bind(&addr)
        .await
        .expect("TCPリスナーのバインドに失敗しました");

    tracing::info!(
        elapsed_ms = t0.elapsed().as_millis(),
        "サーバーを {} で起動します [listener bind 完了]",
        addr
    );

    // DB 接続と migration をバックグラウンドで実行し、完了後に startup_ready を立てる
    let db_url = secrets.database_url.clone();
    let max_connections = config.database_max_connections;
    let startup_ready_bg = Arc::clone(&startup_ready);
    tokio::spawn(async move {
        tracing::info!("データベースに接続中...");
        let pool = connect_pool_with_retry(&db_url, max_connections)
            .await
            .unwrap_or_else(|e| {
                tracing::error!("{e}");
                std::process::exit(1);
            });

        tracing::info!(elapsed_ms = t0.elapsed().as_millis(), "DB connect 完了");

        tracing::info!("マイグレーションを実行中...");
        if let Err(e) = run_migrations(&pool).await {
            tracing::error!("マイグレーション失敗: {e}");
            std::process::exit(1);
        }

        tracing::info!(elapsed_ms = t0.elapsed().as_millis(), "migrations 完了");

        startup_ready_bg.store(true, Ordering::Release);
        tracing::info!(
            elapsed_ms = t0.elapsed().as_millis(),
            "startup_ready 反映完了 [起動完了]"
        );
    });

    axum::serve(listener, router)
        .await
        .expect("サーバーの起動に失敗しました");
}
