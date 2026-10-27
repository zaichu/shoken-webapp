//! `sqlx::migrate` 相当のマイグレーションランナー。`_sqlx_migrations` テーブルと
//! 互換（version/description/checksum=SHA-384/success/execution_time）で、
//! `-- no-transaction` ディレクティブと dirty 検知も同じ意味で処理する。
//!
//! advisory lock は接続（セッション）単位なので、プールから取り出した同一接続上で
//! lock → 適用 → unlock を行う。

use super::{Db, DbInner};
use postgres_types::ToSql;
use sha2::{Digest, Sha384};
use std::collections::HashMap;
use std::time::Instant;
use tokio_postgres::error::ErrorPosition;

const MIGRATION_FILES: &[(&str, &str)] = &[
    (
        "0000_extensions",
        include_str!("../../migrations/0000_extensions.sql"),
    ),
    (
        "0001_users",
        include_str!("../../migrations/0001_users.sql"),
    ),
    (
        "0002_sessions",
        include_str!("../../migrations/0002_sessions.sql"),
    ),
    (
        "0003_dividends",
        include_str!("../../migrations/0003_dividends.sql"),
    ),
    (
        "0004_domestic_stocks",
        include_str!("../../migrations/0004_domestic_stocks.sql"),
    ),
    (
        "0005_mutualfunds",
        include_str!("../../migrations/0005_mutualfunds.sql"),
    ),
    (
        "0006_asset_balances",
        include_str!("../../migrations/0006_asset_balances.sql"),
    ),
    (
        "0007_stock",
        include_str!("../../migrations/0007_stock.sql"),
    ),
    (
        "0008_dividend_per_share_cache",
        include_str!("../../migrations/0008_dividend_per_share_cache.sql"),
    ),
    (
        "0009_market_data_provider_rate_control",
        include_str!("../../migrations/0009_market_data_provider_rate_control.sql"),
    ),
    (
        "0010_security_prices",
        include_str!("../../migrations/0010_security_prices.sql"),
    ),
    (
        "0011_fund_balances",
        include_str!("../../migrations/0011_fund_balances.sql"),
    ),
    (
        "0012_asset_balances_view_prev_close",
        include_str!("../../migrations/0012_asset_balances_view_prev_close.sql"),
    ),
    (
        "0013_fund_nav",
        include_str!("../../migrations/0013_fund_nav.sql"),
    ),
    (
        "0014_asset_balances_account",
        include_str!("../../migrations/0014_asset_balances_account.sql"),
    ),
    (
        "0015_fund_nav_fetch_status",
        include_str!("../../migrations/0015_fund_nav_fetch_status.sql"),
    ),
    (
        "0016_comments",
        include_str!("../../migrations/0016_comments.sql"),
    ),
    (
        "0017_csv_import_history",
        include_str!("../../migrations/0017_csv_import_history.sql"),
    ),
];

struct Migration {
    version: i64,
    description: String,
    sql: &'static str,
    /// 先頭が `-- no-transaction` の場合、トランザクション外で実行する
    /// （CREATE INDEX CONCURRENTLY などトランザクション禁止の文用）
    no_tx: bool,
}

fn migrations() -> Vec<Migration> {
    MIGRATION_FILES
        .iter()
        .map(|(name, sql)| {
            let (version, description) = name.split_once('_').expect("migration 名の形式");
            Migration {
                version: version.parse().expect("migration version"),
                description: description.replace('_', " "),
                sql,
                no_tx: sql.starts_with("-- no-transaction"),
            }
        })
        .collect()
}

/// sqlx と同じ lock ID: `0x3d32ad9e * CRC32_ISO_HDLC(db名)`
fn lock_id(database: &str) -> i64 {
    0x3d32ad9e_i64.wrapping_mul(
        crc::Crc::<u32>::new(&crc::CRC_32_ISO_HDLC).checksum(database.as_bytes()) as i64,
    )
}

fn checksum(sql: &str) -> Vec<u8> {
    Sha384::digest(sql.as_bytes()).to_vec()
}

fn sql_line_at_position(sql: &str, position: u32) -> Option<&str> {
    let character_index = usize::try_from(position).ok()?.checked_sub(1)?;
    let byte_index = sql.char_indices().nth(character_index)?.0;
    let start = sql[..byte_index]
        .rfind('\n')
        .map_or(0, |newline| newline + 1);
    let end = sql[byte_index..]
        .find('\n')
        .map_or(sql.len(), |newline| byte_index + newline);
    let line = sql[start..end].trim();
    (!line.is_empty()).then_some(line)
}

fn format_migration_error(error: tokio_postgres::Error, sql: &str) -> String {
    let Some(db_error) = error.as_db_error() else {
        return error.to_string();
    };
    let mut message = format!("{db_error} (SQLSTATE {})", db_error.code().code());
    let sql_line = match db_error.position() {
        Some(ErrorPosition::Original(position)) => sql_line_at_position(sql, *position),
        Some(ErrorPosition::Internal { position, query }) => sql_line_at_position(query, *position),
        None => None,
    };
    if let Some(sql_line) = sql_line {
        message.push_str("\nSQL: ");
        message.push_str(sql_line);
    }
    message
}

pub async fn run_migrations(db: &Db) -> Result<(), String> {
    let DbInner::Pool(pool) = &*db.inner;
    let mut obj = pool
        .get()
        .await
        .map_err(|e| format!("マイグレーション用接続の取得に失敗: {e}"))?;
    let client = &mut *obj;

    let db_name: String = client
        .query_one("SELECT current_database()", &[])
        .await
        .map_err(|e| format!("current_database の取得に失敗: {e}"))?
        .get(0);
    let lock = lock_id(&db_name);

    client
        .execute("SELECT pg_advisory_lock($1)", &[&lock])
        .await
        .map_err(|e| format!("マイグレーション lock の取得に失敗: {e}"))?;

    let result = run_all(client).await;

    if let Err(e) = client
        .execute("SELECT pg_advisory_unlock($1)", &[&lock])
        .await
    {
        tracing::warn!("マイグレーション lock の解放に失敗: {e}");
    }
    result
}

async fn run_all(client: &mut tokio_postgres::Client) -> Result<(), String> {
    client
        .batch_execute(
            "CREATE TABLE IF NOT EXISTS _sqlx_migrations (
                version BIGINT PRIMARY KEY,
                description TEXT NOT NULL,
                installed_on TIMESTAMPTZ NOT NULL DEFAULT now(),
                success BOOLEAN NOT NULL,
                checksum BYTEA NOT NULL,
                execution_time BIGINT NOT NULL
            )",
        )
        .await
        .map_err(|e| format!("_sqlx_migrations テーブル作成に失敗: {e}"))?;

    let dirty = client
        .query(
            "SELECT version FROM _sqlx_migrations WHERE success = false",
            &[],
        )
        .await
        .map_err(|e| format!("マイグレーション状態の確認に失敗: {e}"))?;
    if let Some(row) = dirty.first() {
        let version: i64 = row.get(0);
        return Err(format!(
            "マイグレーション {version} が dirty です（前回適用が中断）。_sqlx_migrations の当該行を確認して手動で修復してください"
        ));
    }

    let applied: HashMap<i64, Vec<u8>> = client
        .query(
            "SELECT version, checksum FROM _sqlx_migrations WHERE success = true",
            &[],
        )
        .await
        .map_err(|e| format!("適用済みマイグレーションの取得に失敗: {e}"))?
        .iter()
        .map(|row| (row.get(0), row.get(1)))
        .collect();

    let local = migrations();

    for applied_version in applied.keys() {
        if !local.iter().any(|m| m.version == *applied_version) {
            return Err(format!(
                "適用済みマイグレーション {applied_version} に対応するファイルがありません\n\
                 `make db-reset` で DB を作り直してください"
            ));
        }
    }

    for m in &local {
        match applied.get(&m.version) {
            Some(sum) if *sum != checksum(m.sql) => {
                return Err(format!(
                    "マイグレーション {} の適用済み内容が現行ファイルと一致しません\n\
                     `make db-reset` で DB を作り直してください",
                    m.version
                ));
            }
            Some(_) => continue,
            None => {
                tracing::info!(
                    version = m.version,
                    description = %m.description,
                    "マイグレーション適用中"
                );
                apply(client, m)
                    .await
                    .map_err(|e| format!("マイグレーション {} の適用に失敗: {e}", m.version))?;
            }
        }
    }
    Ok(())
}

async fn apply(client: &mut tokio_postgres::Client, m: &Migration) -> Result<(), String> {
    let sum = checksum(m.sql);
    if m.no_tx {
        insert_migration(client, m, &sum, false, 0).await?;
        let started = Instant::now();
        client
            .batch_execute(m.sql)
            .await
            .map_err(|e| format_migration_error(e, m.sql))?;
        let elapsed = i64::try_from(started.elapsed().as_nanos()).unwrap_or(i64::MAX);
        client
            .execute(
                "UPDATE _sqlx_migrations SET success = true, execution_time = $2 WHERE version = $1",
                &[
                    &m.version as &(dyn ToSql + Sync),
                    &elapsed,
                ],
            )
            .await
            .map_err(|e| format!("マイグレーション成功記録の更新に失敗: {e}"))?;
        return Ok(());
    }

    let tx = client
        .transaction()
        .await
        .map_err(|e| format!("トランザクション開始に失敗: {e}"))?;
    let started = Instant::now();
    tx.batch_execute(m.sql)
        .await
        .map_err(|e| format_migration_error(e, m.sql))?;
    let elapsed = i64::try_from(started.elapsed().as_nanos()).unwrap_or(i64::MAX);
    insert_migration_tx(&tx, m, &sum, elapsed).await?;
    tx.commit()
        .await
        .map_err(|e| format!("コミットに失敗: {e}"))?;
    Ok(())
}

async fn insert_migration(
    client: &tokio_postgres::Client,
    m: &Migration,
    sum: &[u8],
    success: bool,
    execution_time: i64,
) -> Result<(), String> {
    client
        .execute(
            "INSERT INTO _sqlx_migrations
                (version, description, success, checksum, execution_time)
            VALUES ($1, $2, $3, $4, $5)",
            &[
                &m.version as &(dyn ToSql + Sync),
                &m.description,
                &success,
                &sum,
                &execution_time,
            ],
        )
        .await
        .map(|_| ())
        .map_err(|e| format!("マイグレーション記録の挿入に失敗: {e}"))
}

async fn insert_migration_tx(
    tx: &tokio_postgres::Transaction<'_>,
    m: &Migration,
    sum: &[u8],
    execution_time: i64,
) -> Result<(), String> {
    tx.execute(
        "INSERT INTO _sqlx_migrations
            (version, description, success, checksum, execution_time)
        VALUES ($1, $2, $3, $4, $5)",
        &[
            &m.version as &(dyn ToSql + Sync),
            &m.description,
            &true,
            &sum,
            &execution_time,
        ],
    )
    .await
    .map(|_| ())
    .map_err(|e| format!("マイグレーション記録の挿入に失敗: {e}"))
}

#[cfg(test)]
mod tests {
    use super::sql_line_at_position;

    #[test]
    fn extracts_sql_line_from_postgres_character_position() {
        let sql = "-- 日本語\nCREATE INDEX broken";
        let position = sql
            .chars()
            .position(|character| character == 'b')
            .expect("対象文字がある") as u32
            + 1;

        assert_eq!(
            sql_line_at_position(sql, position),
            Some("CREATE INDEX broken")
        );
    }
}
