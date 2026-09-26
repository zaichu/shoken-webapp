use backend::config::{self, Config};
use backend::db::{run_migrations, wait_for_pool_with_retry};
use backend::logging;
use backend::routes::app_router;
use backend::startup::build_startup_state;
use backend::state::Secrets;
use dotenvy::dotenv;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Instant;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
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

    logging::init_tracing();

    let secrets = Secrets::from_env().expect("シークレットの読み込みに失敗しました");
    let secrets = Arc::new(secrets);

    let config = Config::from_env();

    let startup_ready = Arc::new(AtomicBool::new(false));

    let (startup_pool, state) =
        build_startup_state(Arc::clone(&secrets), &config).unwrap_or_else(|e| {
            tracing::error!("{e}");
            std::process::exit(1);
        });

    let router = app_router(state, &config, &startup_ready);

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
