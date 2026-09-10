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
use db::{connect_pool_lazy, run_migrations, wait_for_pool_with_retry};
use dotenvy::dotenv;
use reqwest::Client;
use routes::app_router;
use sqlx::PgPool;
use state::{AppState, DividendCacheState, Secrets};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Instant;
use tokio::net::TcpListener;

fn build_startup_state(
    secrets: Arc<Secrets>,
    config: &Config,
) -> Result<(PgPool, AppState), String> {
    // URL 検証のみ行い、実接続は background startup task で行う。
    let pool = connect_pool_lazy(&secrets.database_url, config.database_max_connections)?;
    // OAuthトークン交換用のHTTPクライアントを起動時に1つ構築し、以降使い回す。
    services::auth::init_oauth_http_client().map_err(|e| e.to_string())?;
    let state = AppState {
        pool: pool.clone(),
        secrets,
        client: Client::new(),
        dividend_cache: DividendCacheState::default(),
    };

    Ok((pool, state))
}

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
                sentry::ClientOptions::new().maybe_release(sentry::release_name!()),
            ))
        });

    // ロギングの初期化
    logging::init_tracing();

    // シークレットの読み込み
    let secrets = Secrets::from_env().expect("シークレットの読み込みに失敗しました");
    let secrets = Arc::new(secrets);

    let config = Config::from_env();

    let startup_ready = Arc::new(AtomicBool::new(false));

    let (startup_pool, state) =
        build_startup_state(Arc::clone(&secrets), &config).unwrap_or_else(|e| {
            tracing::error!("{e}");
            std::process::exit(1);
        });

    let router = app_router(state, &config, Arc::clone(&startup_ready));

    let addr = config::server_addr();

    let t0 = Instant::now();

    let listener = TcpListener::bind(&addr)
        .await
        .expect("TCPリスナーのバインドに失敗しました");

    let listener_bind_us = t0.elapsed().as_micros();
    tracing::info!(
        target: "startup",
        elapsed_ms = listener_bind_us / 1000,
        "サーバーを {} で起動します [listener bind 完了]",
        addr
    );

    // DB 接続と migration をバックグラウンドで実行し、完了後に startup_ready を立てる
    let startup_ready_bg = Arc::clone(&startup_ready);
    tokio::spawn(async move {
        tracing::info!(target: "startup", "データベースに接続中...");
        wait_for_pool_with_retry(&startup_pool)
            .await
            .unwrap_or_else(|e| {
                tracing::error!("{e}");
                std::process::exit(1);
            });

        let db_connect_ms = t0.elapsed().as_millis();
        tracing::info!(target: "startup", elapsed_ms = db_connect_ms, "DB connect 完了");

        tracing::info!(target: "startup", "マイグレーションを実行中...");
        if let Err(e) = run_migrations(&startup_pool).await {
            tracing::error!("マイグレーション失敗: {e}");
            std::process::exit(1);
        }

        let migrations_ms = t0.elapsed().as_millis();
        tracing::info!(target: "startup", elapsed_ms = migrations_ms, "migrations 完了");

        startup_ready_bg.store(true, Ordering::Release);
        let startup_ready_ms = t0.elapsed().as_millis();
        tracing::info!(
            target: "startup",
            elapsed_ms = startup_ready_ms,
            "startup_ready 反映完了 [起動完了]"
        );

        tracing::info!(
            target: "startup",
            listener_bind_phase_us = listener_bind_us,
            db_connect_phase_ms = db_connect_ms - listener_bind_us / 1000,
            db_connect_total_ms = db_connect_ms,
            migrations_phase_ms = migrations_ms - db_connect_ms,
            migrations_total_ms = migrations_ms,
            startup_ready_total_ms = startup_ready_ms,
            "[startup summary] 全フェーズ完了"
        );
    });

    axum::serve(listener, router)
        .await
        .expect("サーバーの起動に失敗しました");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_secrets() -> Arc<Secrets> {
        Arc::new(Secrets {
            database_url: "postgresql://user:password@localhost/test_db".to_string(),
            jquants_api_key: None,
            google_client_id: Some("client-id".to_string()),
            google_client_secret: Some("client-secret".to_string()),
            frontend_url: "http://localhost:8080".to_string(),
        })
    }

    #[tokio::test]
    async fn build_startup_state_reuses_pool_for_runtime_and_startup_tasks() {
        let config = Config::default();
        let (startup_pool, state) =
            build_startup_state(test_secrets(), &config).expect("startup state");

        startup_pool.close().await;

        assert!(
            state.pool.is_closed(),
            "runtime state pool should share the startup pool handle"
        );
    }
}
