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

/// 既存環境向け: `migrations/0001〜0017` を逐次適用してスキーマを最新化する。
/// 本番 DB・ローカル開発 DB のアップグレードに使用する。
pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!().run(pool).await
}

/// 新規環境向け: `migrations_baseline/0001_baseline.sql` で一発初期化する。
/// 0001〜0017 を集約した最終スキーマを直接適用するため、テストや CI で高速化できる。
/// 既存 DB には使用しないこと（migration 履歴が変わるため）。
///
/// 統合テスト（`tests/db_integration.rs`）から使用する。バイナリは `run_migrations` を使用。
#[allow(dead_code)]
pub async fn run_baseline_migrations(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations_baseline").run(pool).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connect_pool_lazy_creates_pool() {
        let database_url = "postgresql://user:password@localhost/test_db";
        let result = connect_pool_lazy(database_url, 5);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_connect_pool_lazy_with_max_connections() {
        let database_url = "postgresql://user:password@localhost/test_db";
        let result = connect_pool_lazy(database_url, 10);
        assert!(result.is_ok());
    }
}
