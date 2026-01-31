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

pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!().run(pool).await
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
