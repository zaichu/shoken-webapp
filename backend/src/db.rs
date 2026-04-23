use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;
use url::Url;

// 起動時 DB 接続の retry パラメータ。fly.toml の grace_period と整合すること
pub(crate) const MAX_ATTEMPTS: u32 = 5;
pub(crate) const CONNECT_TIMEOUT_SECS: u64 = 5;
pub(crate) const RETRY_DELAY_SECS: u64 = 3;

// runtime の Pool::acquire() slow warning 閾値。Fly + 外部 PG ではアイドル後再接続が
// CONNECT_TIMEOUT_SECS を超えることがあるため、起動 retry timeout とは別定数にする。
pub(crate) const ACQUIRE_SLOW_THRESHOLD_SECS: u64 = 10;

fn pool_options(max_connections: u32) -> PgPoolOptions {
    PgPoolOptions::new()
        .max_connections(max_connections)
        .acquire_slow_threshold(Duration::from_secs(ACQUIRE_SLOW_THRESHOLD_SECS))
}

pub async fn connect_pool(database_url: &str, max_connections: u32) -> Result<PgPool, sqlx::Error> {
    pool_options(max_connections).connect(database_url).await
}

#[allow(dead_code)]
pub fn connect_pool_lazy(database_url: &str, max_connections: u32) -> Result<PgPool, sqlx::Error> {
    pool_options(max_connections).connect_lazy(database_url)
}

/// URL の設定を事前検証し、一時的な接続失敗は bounded retry する。
/// 設定エラーは retry しない。秘密情報はエラー文字列に含めない。
pub async fn connect_pool_with_retry(
    database_url: &str,
    max_connections: u32,
) -> Result<PgPool, String> {
    let database_url = database_url.trim();
    validate_database_url(database_url)?;
    let sanitized_url = sanitize_database_url_for_sqlx(database_url)?;

    let connect_timeout = Duration::from_secs(CONNECT_TIMEOUT_SECS);
    let retry_delay = Duration::from_secs(RETRY_DELAY_SECS);

    for attempt in 1..=MAX_ATTEMPTS {
        match tokio::time::timeout(
            connect_timeout,
            connect_pool(&sanitized_url, max_connections),
        )
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

/// SQLx に渡す前に `channel_binding` query parameter を除去する。
/// `channel_binding` がない場合は入力文字列をそのまま返す。
/// 保持する query pair は raw 文字列を再利用し percent-encoding を変換しない。
fn sanitize_database_url_for_sqlx(url: &str) -> Result<String, String> {
    let parsed = Url::parse(url).map_err(|_| "DATABASE_URL の解析に失敗しました".to_string())?;

    if !parsed.query_pairs().any(|(k, _)| k == "channel_binding") {
        return Ok(url.to_string());
    }

    // raw pair と decoded pair を zip し、decoded key が channel_binding の raw pair を除去する
    let filtered_query = parsed
        .query()
        .map(|q| {
            let raw_pairs: Vec<&str> = q.split('&').collect();
            let decoded_pairs: Vec<_> = parsed.query_pairs().collect();
            raw_pairs
                .iter()
                .zip(decoded_pairs.iter())
                .filter(|(_, (k, _))| k != "channel_binding")
                .map(|(raw, _)| *raw)
                .collect::<Vec<_>>()
                .join("&")
        })
        .unwrap_or_default();

    let fragment = parsed
        .fragment()
        .map(|f| format!("#{f}"))
        .unwrap_or_default();
    let base = url.find('?').map_or(url, |pos| &url[..pos]);
    if filtered_query.is_empty() {
        Ok(format!("{base}{fragment}"))
    } else {
        Ok(format!("{base}?{filtered_query}{fragment}"))
    }
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

    // 定数の大小関係をコンパイル時に保証する
    const _: () = assert!(
        ACQUIRE_SLOW_THRESHOLD_SECS > CONNECT_TIMEOUT_SECS,
        "ACQUIRE_SLOW_THRESHOLD_SECS は CONNECT_TIMEOUT_SECS より大きくなければならない"
    );

    #[test]
    fn test_db_pool_options_use_acquire_slow_threshold() {
        let opts5 = pool_options(5);
        assert_eq!(
            opts5.get_acquire_slow_threshold(),
            Duration::from_secs(ACQUIRE_SLOW_THRESHOLD_SECS),
            "slow acquire threshold は ACQUIRE_SLOW_THRESHOLD_SECS であること"
        );
        assert_ne!(
            opts5.get_acquire_slow_threshold(),
            Duration::from_secs(CONNECT_TIMEOUT_SECS),
            "slow acquire threshold は CONNECT_TIMEOUT_SECS と異なること"
        );
        assert_eq!(
            opts5.get_max_connections(),
            5,
            "max_connections が一致すること"
        );

        let opts10 = pool_options(10);
        assert_eq!(
            opts10.get_acquire_slow_threshold(),
            Duration::from_secs(ACQUIRE_SLOW_THRESHOLD_SECS),
        );
        assert_eq!(opts10.get_max_connections(), 10);
    }

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

    #[test]
    fn test_sanitize_database_url_removes_channel_binding() {
        // channel_binding だけを除去する
        let url = "postgres://localhost/db?channel_binding=require";
        let sanitized = sanitize_database_url_for_sqlx(url).unwrap();
        let parsed = url::Url::parse(&sanitized).unwrap();
        let params: Vec<_> = parsed.query_pairs().collect();
        assert!(
            params.iter().all(|(k, _)| k != "channel_binding"),
            "channel_binding が残っている"
        );
    }

    #[test]
    fn test_sanitize_database_url_preserves_other_params() {
        // sslmode など他の query parameter は保持する
        let url = "postgres://localhost/db?sslmode=require&channel_binding=require";
        let sanitized = sanitize_database_url_for_sqlx(url).unwrap();
        let parsed = url::Url::parse(&sanitized).unwrap();
        let params: Vec<_> = parsed.query_pairs().collect();
        assert!(
            params.iter().any(|(k, _)| k == "sslmode"),
            "sslmode が除去されている"
        );
        assert!(
            params.iter().all(|(k, _)| k != "channel_binding"),
            "channel_binding が残っている"
        );
    }

    #[test]
    fn test_sanitize_database_url_only_channel_binding_param() {
        // channel_binding だけの場合 query が空になる
        let url = "postgres://localhost/db?channel_binding=require";
        let sanitized = sanitize_database_url_for_sqlx(url).unwrap();
        let parsed = url::Url::parse(&sanitized).unwrap();
        assert!(
            parsed.query().is_none() || parsed.query() == Some(""),
            "query が空でない: {:?}",
            parsed.query()
        );
    }

    #[test]
    fn test_sanitize_database_url_preserves_percent_encoding() {
        // %20 が + などに変換されず raw encoding のまま保持される
        let url =
            "postgres://localhost/db?application_name=shoken%20backend&channel_binding=require";
        let sanitized = sanitize_database_url_for_sqlx(url).unwrap();
        assert!(
            sanitized.contains("application_name=shoken%20backend"),
            "%20 が + などに変換されている"
        );
        assert!(
            !sanitized.contains("channel_binding"),
            "channel_binding が残っている"
        );
    }

    #[test]
    fn test_sanitize_database_url_no_change_without_channel_binding() {
        // channel_binding がない URL は入力文字列をそのまま返す
        let url = "postgres://localhost/db?sslmode=require&application_name=shoken%20backend";
        let sanitized = sanitize_database_url_for_sqlx(url).unwrap();
        assert!(sanitized == url, "入力文字列が変化した");
    }

    #[test]
    fn test_sanitize_database_url_removes_percent_encoded_key() {
        // decoded key が channel_binding になる percent-encoded raw key も除去される
        // %5F は '_' なので channel%5Fbinding は channel_binding に decode される
        let url = "postgres://localhost/db?channel%5Fbinding=require&sslmode=require";
        let sanitized = sanitize_database_url_for_sqlx(url).unwrap();
        assert!(
            !sanitized.contains("channel"),
            "percent-encoded channel_binding key が残っている"
        );
        assert!(
            sanitized.contains("sslmode=require"),
            "sslmode が除去されている"
        );
    }

    #[test]
    fn test_sanitize_database_url_preserves_fragment() {
        // fragment は channel_binding 除去後も保持される
        let url = "postgres://localhost/db?sslmode=require&channel_binding=require#frag";
        let sanitized = sanitize_database_url_for_sqlx(url).unwrap();
        assert!(sanitized.ends_with("#frag"), "fragment が消えている");
        assert!(
            !sanitized.contains("channel_binding"),
            "channel_binding が残っている"
        );
        assert!(
            sanitized.contains("sslmode=require"),
            "sslmode が除去されている"
        );
    }

    #[test]
    fn test_sanitize_database_url_invalid_url_error_safe() {
        // invalid URL のエラーに URL 本体・user・password・host を含まない
        let err = sanitize_database_url_for_sqlx("not-a-valid-url").unwrap_err();
        assert!(
            !err.contains("not-a-valid-url"),
            "エラーに URL 値を含めない: {err}"
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
