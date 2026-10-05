use sqlx::{postgres::PgPoolOptions, PgPool};
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres::Postgres;
use tokio::time::{sleep, timeout, Duration};

async fn connect_with_retry(database_url: &str) -> PgPool {
    let mut last_error = None;
    for _ in 0..20 {
        match timeout(
            Duration::from_secs(2),
            PgPoolOptions::new().connect(database_url),
        )
        .await
        {
            Ok(Ok(pool)) => return pool,
            Ok(Err(err)) => last_error = Some(err),
            Err(_) => {}
        }
        sleep(Duration::from_millis(500)).await;
    }
    panic!("DB 接続失敗: {last_error:?}");
}

/// Docker 上に独立した Postgres コンテナを起動し、migration 済みの pool を返す。
/// 各テストが別コンテナ・DB を持つ隔離は維持し、返す node でコンテナを生存させる。
pub async fn start_test_pool() -> (PgPool, impl Drop) {
    let node = Postgres::default().start().await.unwrap();
    let port = node.get_host_port_ipv4(5432).await.unwrap();
    let database_url = format!("postgres://postgres:postgres@127.0.0.1:{port}/postgres");
    let pool = connect_with_retry(&database_url).await;
    crate::db::run_migrations(&pool)
        .await
        .expect("マイグレーション失敗");
    (pool, node)
}
