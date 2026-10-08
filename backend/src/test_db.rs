use crate::db::Db;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres::Postgres;
use tokio::time::{Duration, sleep};

async fn connect_with_retry(database_url: &str) -> Db {
    let pool = crate::db::connect_pool_lazy(database_url, 2).expect("テスト DB pool の構築に失敗");
    let mut last_error = None;
    for _ in 0..20 {
        match crate::db::query("SELECT 1", vec![]).execute(&pool).await {
            Ok(_) => return pool,
            Err(err) => last_error = Some(err),
        }
        sleep(Duration::from_millis(500)).await;
    }
    panic!("DB 接続失敗: {last_error:?}");
}

/// Docker 上に独立した Postgres コンテナを起動し、migration 済みの pool を返す。
/// 各テストが別コンテナ・DB を持つ隔離は維持し、返す node でコンテナを生存させる。
pub async fn start_test_pool() -> (Db, impl Drop) {
    let node = Postgres::default().start().await.unwrap();
    let port = node.get_host_port_ipv4(5432).await.unwrap();
    let database_url = format!("postgres://postgres:postgres@127.0.0.1:{port}/postgres");
    let pool = connect_with_retry(&database_url).await;
    crate::db::run_migrations(&pool)
        .await
        .expect("マイグレーション失敗");
    (pool, node)
}
