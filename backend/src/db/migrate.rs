//! `sqlx::migrate` 相当のマイグレーションランナー。`_sqlx_migrations` テーブルと
//! 互換（version/description/checksum=SHA-384/success/execution_time）で、
//! `-- no-transaction` ディレクティブと dirty 検知も同じ意味で処理する。
//!
//! advisory lock は接続（セッション）単位なので、プールから取り出した同一接続上で
//! lock → 適用 → unlock を行う。

use super::{Db, DbInner};
use postgres_types::ToSql;
use sha2::{Digest, Sha384};
use std::collections::{BTreeSet, HashMap};
use std::time::Instant;

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
];

/// ベースラインファイル群の最大バージョン。これ以下の version は旧履歴置き換え時に
/// 「適用済み」として記録される。新しい migration は 0011 以降に採番する
const BASELINE_MAX_VERSION: i64 = 10;

/// 旧ファイル名で運用していた頃の最大バージョン(状態A: 旧運用履歴 1〜17)。
/// この時点のスキーマをベースラインにまとめたため、これより大きい version の旧履歴は存在しない
const LEGACY_MAX_VERSION: i64 = 17;

/// 現行の baseline + 増分を全適用した状態(状態B: 履歴 0〜18)の最大バージョン。
/// 状態Aと状態Bの schema は同じなので、両方の履歴を rotate の許可対象にする
const INCREMENT_MAX_VERSION: i64 = 18;

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

    let mut applied: HashMap<i64, Vec<u8>> = client
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
    squash_legacy_history(client, &mut applied, &local).await?;

    for applied_version in applied.keys() {
        if !local.iter().any(|m| m.version == *applied_version) {
            return Err(format!(
                "適用済みマイグレーション {applied_version} に対応するファイルがありません"
            ));
        }
    }

    for m in &local {
        match applied.get(&m.version) {
            Some(sum) if *sum != checksum(m.sql) => {
                return Err(format!(
                    "マイグレーション {} の checksum が一致しません（適用後にファイルが編集された可能性）",
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

/// 旧運用履歴(状態A: 1〜LEGACY_MAX_VERSION)または旧 baseline+増分の履歴
/// (状態B: 0〜INCREMENT_MAX_VERSION)が残る DB の `_sqlx_migrations` を、
/// ベースラインファイル群(0〜BASELINE_MAX_VERSION)の記録へ一度だけ置き換える。
/// テーブル・データには触れない。どちらの状態も schema は現行と同じ。
/// それ以外の中途半端な履歴は schema が現行と異なる可能性があるためエラーで止める
async fn squash_legacy_history(
    client: &mut tokio_postgres::Client,
    applied: &mut HashMap<i64, Vec<u8>>,
    local: &[Migration],
) -> Result<(), String> {
    let baseline_v1 = local
        .iter()
        .find(|m| m.version == 1)
        .expect("baseline (version=1) の migration が必要です");
    // 旧運用の version=1 は 0001_initial_schema.sql だったため、その checksum と
    // 新 0001_users.sql の差だけを見て判定する。version 2 以降の存在では判定しない
    // (新形式の追加 migration と番号が被り、履歴を誤って消すため)
    let has_legacy = applied
        .get(&1)
        .is_some_and(|sum| *sum != checksum(baseline_v1.sql));
    if !has_legacy {
        return Ok(());
    }

    let applied_versions: BTreeSet<i64> = applied.keys().copied().collect();
    let legacy_all: BTreeSet<i64> = (1..=LEGACY_MAX_VERSION).collect();
    let linear_all: BTreeSet<i64> = (0..=INCREMENT_MAX_VERSION).collect();
    if applied_versions != legacy_all && applied_versions != linear_all {
        return Err(format!(
            "旧マイグレーション履歴(1〜{LEGACY_MAX_VERSION} または 0〜{INCREMENT_MAX_VERSION})\
             が揃っていません (現在: {applied_versions:?})。\
             schema が現行と異なる可能性があるため適用を中止しました"
        ));
    }

    let baseline: Vec<(&Migration, Vec<u8>)> = local
        .iter()
        .filter(|m| m.version <= BASELINE_MAX_VERSION)
        .map(|m| (m, checksum(m.sql)))
        .collect();
    let tx = client
        .transaction()
        .await
        .map_err(|e| format!("トランザクション開始に失敗: {e}"))?;
    tx.execute("DELETE FROM _sqlx_migrations", &[])
        .await
        .map_err(|e| format!("旧マイグレーション履歴の削除に失敗: {e}"))?;
    for (m, sum) in &baseline {
        insert_migration_tx(&tx, m, sum, 0).await?;
    }
    tx.commit()
        .await
        .map_err(|e| format!("コミットに失敗: {e}"))?;
    tracing::info!(
        "旧マイグレーション履歴をベースライン 0〜{BASELINE_MAX_VERSION} の記録に置き換えました"
    );
    applied.clear();
    applied.extend(baseline.into_iter().map(|(m, sum)| (m.version, sum)));
    Ok(())
}

async fn apply(client: &mut tokio_postgres::Client, m: &Migration) -> Result<(), String> {
    let sum = checksum(m.sql);
    if m.no_tx {
        insert_migration(client, m, &sum, false, 0).await?;
        let started = Instant::now();
        if let Err(e) = client.batch_execute(m.sql).await {
            return Err(format!("{e}"));
        }
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
    tx.batch_execute(m.sql).await.map_err(|e| format!("{e}"))?;
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
