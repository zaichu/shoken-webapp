use sqlx::{postgres::PgPoolOptions, PgPool};

pub async fn connect_pool(database_url: &str, max_connections: u32) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(database_url)
        .await
}

#[allow(dead_code)]
pub fn connect_pool_lazy(database_url: &str, max_connections: u32) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(max_connections)
        .connect_lazy(database_url)
}

/// `migrations/` を適用してスキーマを最新化する。
/// 本番 DB・ローカル DB・テスト全て同じ経路を使用する。
pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!().run(pool).await
}
#[cfg(test)] #[rustfmt::skip] mod tests {
    use super::*;
    #[tokio::test] async fn test_connect_pool_lazy() { let database_url = "postgresql://user:password@localhost/test_db"; assert!(connect_pool_lazy(database_url, 5).is_ok()); assert!(connect_pool_lazy(database_url, 10).is_ok()); }
}
