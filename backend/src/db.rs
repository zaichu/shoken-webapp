use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;

// 起動時 DB 接続の retry パラメータ。fly.toml の grace_period と整合すること
pub(crate) const MAX_ATTEMPTS: u32 = 5;
pub(crate) const CONNECT_TIMEOUT_SECS: u64 = 5;
pub(crate) const RETRY_DELAY_SECS: u64 = 3;

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

/// URL の設定を事前検証し、一時的な接続失敗は bounded retry する。
/// 設定エラーは retry しない。秘密情報はエラー文字列に含めない。
pub async fn connect_pool_with_retry(
    database_url: &str,
    max_connections: u32,
) -> Result<PgPool, String> {
    let database_url = database_url.trim();
    validate_database_url(database_url)?;

    let connect_timeout = Duration::from_secs(CONNECT_TIMEOUT_SECS);
    let retry_delay = Duration::from_secs(RETRY_DELAY_SECS);

    for attempt in 1..=MAX_ATTEMPTS {
        match tokio::time::timeout(connect_timeout, connect_pool(database_url, max_connections))
            .await
        {
            Ok(Ok(pool)) => return Ok(pool),
            Ok(Err(e)) if is_transient_error(&e) => {
                tracing::warn!(
                    attempt,
                    max_attempts = MAX_ATTEMPTS,
                    "DB接続の一時的なエラー、リトライします"
                );
            }
            Ok(Err(_)) => {
                return Err("データベース接続の設定が不正です".to_string());
            }
            Err(_) => {
                tracing::warn!(
                    attempt,
                    max_attempts = MAX_ATTEMPTS,
                    "DB接続がタイムアウト、リトライします"
                );
            }
        }

        if attempt < MAX_ATTEMPTS {
            tokio::time::sleep(retry_delay).await;
        }
    }

    Err("データベースへの接続に失敗しました（リトライ上限超過）".to_string())
}

fn validate_database_url(url: &str) -> Result<(), String> {
    if url.is_empty() {
        return Err("DATABASE_URL が空です".to_string());
    }
    let lower = url.to_lowercase();
    if !lower.starts_with("postgres://") && !lower.starts_with("postgresql://") {
        return Err(
            "DATABASE_URL のスキームが不正です（postgres:// または postgresql:// が必要です）"
                .to_string(),
        );
    }
    Ok(())
}

fn is_transient_error(e: &sqlx::Error) -> bool {
    matches!(e, sqlx::Error::PoolTimedOut | sqlx::Error::Io(_))
}

/// `migrations/` を適用してスキーマを最新化する。
/// 本番 DB・ローカル DB・テスト全て同じ経路を使用する。
pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!().run(pool).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connect_pool_lazy() {
        let database_url = "postgresql://user:password@localhost/test_db";
        assert!(connect_pool_lazy(database_url, 5).is_ok());
        assert!(connect_pool_lazy(database_url, 10).is_ok());
    }

    #[test]
    fn test_validate_database_url_empty() {
        let err = validate_database_url("").unwrap_err();
        assert!(!err.contains("://"), "エラーに URL を含めない: {err}");
    }

    #[test]
    fn test_validate_database_url_relative() {
        let err = validate_database_url("/relative").unwrap_err();
        assert!(
            !err.contains("/relative"),
            "エラーに URL 値を含めない: {err}"
        );
    }

    #[test]
    fn test_validate_database_url_wrong_scheme() {
        let err = validate_database_url("http://example.com/db").unwrap_err();
        assert!(
            !err.contains("example.com"),
            "エラーにホスト名を含めない: {err}"
        );
    }

    #[test]
    fn test_validate_database_url_valid() {
        assert!(validate_database_url("postgres://user:password@localhost/db").is_ok());
        assert!(validate_database_url("postgresql://user:password@localhost/db").is_ok());
        assert!(validate_database_url("POSTGRES://localhost/db").is_ok());
    }

    #[test]
    fn test_is_transient_error() {
        assert!(is_transient_error(&sqlx::Error::PoolTimedOut));
        assert!(!is_transient_error(&sqlx::Error::RowNotFound));
        assert!(!is_transient_error(&sqlx::Error::ColumnNotFound(
            "col".to_string()
        )));
    }

    #[tokio::test]
    async fn test_connect_pool_with_retry_invalid_url() {
        // 空
        let err = connect_pool_with_retry("", 5).await.unwrap_err();
        assert!(err.contains("空"), "空URLのエラーメッセージが適切: {err}");
        assert!(!err.contains("://"), "URLをエラーに含めない: {err}");

        // 空白のみ（trim 後に空として拒否）
        let err = connect_pool_with_retry("   ", 5).await.unwrap_err();
        assert!(err.contains("空"), "空白のみは空として拒否: {err}");

        // 相対 URL
        let err = connect_pool_with_retry("/relative", 5).await.unwrap_err();
        assert!(
            !err.contains("/relative"),
            "相対URLをエラーに含めない: {err}"
        );

        // HTTP スキーム
        let err = connect_pool_with_retry("http://example.com/db", 5)
            .await
            .unwrap_err();
        assert!(
            !err.contains("example.com"),
            "ホスト名をエラーに含めない: {err}"
        );

        // 前後空白 + HTTP スキーム（trim 後にスキームエラー）
        let err = connect_pool_with_retry("  http://example.com/db  ", 5)
            .await
            .unwrap_err();
        assert!(
            !err.contains("example.com"),
            "trim後のURLをエラーに含めない: {err}"
        );
    }

    /// 最大待機時間が fly.toml の grace_period を超えないことを保証する。
    /// grace_period を変更した場合はこのテストも更新すること。
    #[test]
    fn test_retry_budget_fits_grace_period() {
        const GRACE_PERIOD_SECS: u64 = 40; // fly.toml [[http_service.checks]] grace_period と同期
        let max_wait_secs = MAX_ATTEMPTS as u64 * CONNECT_TIMEOUT_SECS
            + (MAX_ATTEMPTS as u64 - 1) * RETRY_DELAY_SECS;
        assert!(
            max_wait_secs < GRACE_PERIOD_SECS,
            "最大待機時間 {max_wait_secs}s が grace_period {GRACE_PERIOD_SECS}s を超える"
        );
    }
}
