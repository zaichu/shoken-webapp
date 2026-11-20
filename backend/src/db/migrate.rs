//! `sqlx::migrate` 相当のマイグレーションランナー。`_sqlx_migrations` テーブルと
//! 互換（version/description/checksum=SHA-384/success/execution_time）で、
//! `-- no-transaction` ディレクティブと dirty 検知も同じ意味で処理する。
//!
//! 旧 migration(`migrations/archive`)を全て適用済みの DB では、土台を SQL を流さずに
//! 適用済みとして記録する(`migrations/README.md` の「_sqlx_migrations の履歴と土台」)。
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
        "0100_common",
        include_str!("../../migrations/0100_common.sql"),
    ),
    (
        "0101_users_and_auth",
        include_str!("../../migrations/0101_users_and_auth.sql"),
    ),
    (
        "0102_transactions",
        include_str!("../../migrations/0102_transactions.sql"),
    ),
    (
        "0103_shared_masters_and_caches",
        include_str!("../../migrations/0103_shared_masters_and_caches.sql"),
    ),
    (
        "0104_holdings",
        include_str!("../../migrations/0104_holdings.sql"),
    ),
    (
        "0105_grants",
        include_str!("../../migrations/0105_grants.sql"),
    ),
    (
        "0106_limited_functions",
        include_str!("../../migrations/0106_limited_functions.sql"),
    ),
    (
        "0107_session_idle_timeout",
        include_str!("../../migrations/0107_session_idle_timeout.sql"),
    ),
];

/// 土台に畳んだ旧 migration(`migrations/archive`)の最後の版
const LEGACY_LAST_VERSION: i64 = 37;

/// 旧 migration の (版, checksum) を版の順に連結した SHA-384(`legacy_history_digest`)。
/// 本番の履歴がこれと一致するときだけ、土台を適用済みとして記録する
const LEGACY_HISTORY_SHA384: &str = "16ddad330354063dc4b02f6094a3a638c95702125ef42d4549da817eb2059e010f98a0ddbac3678b7090abf31a070d57";

/// 旧 migration の最終状態を表す土台(0100〜0106)の最後の版
const BASELINE_LAST_VERSION: i64 = 106;

struct Migration {
    version: i64,
    description: String,
    sql: &'static str,
    /// 先頭が `-- no-transaction` の場合、トランザクション外で実行する
    /// （CREATE INDEX CONCURRENTLY などトランザクション禁止の文用）
    no_tx: bool,
}

fn migrations() -> Result<Vec<Migration>, String> {
    MIGRATION_FILES
        .iter()
        .map(|(name, sql)| {
            let (version, description) = name
                .split_once('_')
                .ok_or_else(|| format!("マイグレーション名の形式が不正です: {name}"))?;
            Ok(Migration {
                version: version.parse().map_err(|_| {
                    format!("マイグレーションのバージョンが数値ではありません: {name}")
                })?,
                description: description.replace('_', " "),
                sql,
                no_tx: sql.starts_with("-- no-transaction"),
            })
        })
        .collect()
}

/// sqlx と同じ lock ID: `0x3d32ad9e * CRC32_ISO_HDLC(db名)`
const fn lock_id(database: &str) -> i64 {
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
        .map_or_default(|newline| newline + 1);
    let end = sql[byte_index..]
        .find('\n')
        .map_or(sql.len(), |newline| byte_index + newline);
    let line = sql[start..end].trim();
    (!line.is_empty()).then_some(line)
}

fn format_migration_error(error: &tokio_postgres::Error, sql: &str) -> String {
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

    let local = migrations()?;
    let plan = plan(&applied, &local)?;

    if !plan.record_only.is_empty() {
        tracing::info!(
            "旧マイグレーション 0000〜{LEGACY_LAST_VERSION:04} の適用済み DB のため、土台を適用済みとして記録する"
        );
        record_only(client, &plan.record_only).await?;
    }

    for m in plan.pending {
        tracing::info!(
            version = m.version,
            description = %m.description,
            "マイグレーション適用中"
        );
        apply(client, m)
            .await
            .map_err(|e| format!("マイグレーション {} の適用に失敗: {e}", m.version))?;
    }
    Ok(())
}

struct Plan<'a> {
    /// 旧マイグレーションを全て適用済みの DB で、SQL を流さず履歴だけ足す土台
    record_only: Vec<&'a Migration>,
    pending: Vec<&'a Migration>,
}

fn plan<'a>(applied: &HashMap<i64, Vec<u8>>, local: &'a [Migration]) -> Result<Plan<'a>, String> {
    let baseline: Vec<&Migration> = local
        .iter()
        .filter(|m| m.version <= BASELINE_LAST_VERSION)
        .collect();
    let mut record_only = Vec::new();
    if applied
        .keys()
        .any(|version| *version <= LEGACY_LAST_VERSION)
    {
        check_legacy_history(applied)?;
        let recorded = baseline
            .iter()
            .filter(|m| applied.contains_key(&m.version))
            .count();
        if recorded == 0 {
            record_only.clone_from(&baseline);
        } else if recorded != baseline.len() {
            return Err(format!(
                "旧マイグレーションの履歴がある DB で、土台の記録が {recorded}/{} 件しかありません\n\
                 _sqlx_migrations を確認して手動で修復してください",
                baseline.len()
            ));
        }
    }

    for applied_version in applied.keys() {
        if *applied_version > LEGACY_LAST_VERSION
            && !local.iter().any(|m| m.version == *applied_version)
        {
            return Err(format!(
                "適用済みマイグレーション {applied_version} に対応するファイルがありません\n\
                 `make db-reset` で DB を作り直してください"
            ));
        }
    }

    let mut pending = Vec::new();
    for m in local {
        match applied.get(&m.version) {
            Some(sum) if *sum != checksum(m.sql) => {
                return Err(format!(
                    "マイグレーション {} の適用済み内容が現行ファイルと一致しません\n\
                     `make db-reset` で DB を作り直してください",
                    m.version
                ));
            }
            Some(_) => {}
            None if record_only.iter().any(|r| r.version == m.version) => {}
            None => pending.push(m),
        }
    }
    Ok(Plan {
        record_only,
        pending,
    })
}

/// 旧マイグレーションの履歴(版と checksum)が archive と同じかを確かめる。
/// 途中までの DB は旧ファイルの続きを当てられないため止める
fn check_legacy_history(applied: &HashMap<i64, Vec<u8>>) -> Result<(), String> {
    let mut legacy: Vec<(i64, &Vec<u8>)> = applied
        .iter()
        .filter(|(version, _)| **version <= LEGACY_LAST_VERSION)
        .map(|(version, sum)| (*version, sum))
        .collect();
    legacy.sort_unstable_by_key(|(version, _)| *version);
    if let Some((last, _)) = legacy.last()
        && *last < LEGACY_LAST_VERSION
    {
        return Err(format!(
            "旧マイグレーションが {last:04} までしか適用されていません(最後は {LEGACY_LAST_VERSION:04})。\
             旧マイグレーションは土台に畳んだため、途中まで適用された DB は続きを当てられません\n\
             `make db-reset` で DB を作り直してください"
        ));
    }
    if legacy_history_digest(&legacy) != LEGACY_HISTORY_SHA384 {
        return Err(
            "旧マイグレーションの適用済み履歴が migrations/archive と一致しません。土台を記録せずに止めます\n\
             _sqlx_migrations の版・checksum と archive の差を確認してください(ローカルなら `make db-reset`)"
                .to_string(),
        );
    }
    Ok(())
}

fn legacy_history_digest(legacy: &[(i64, &Vec<u8>)]) -> String {
    let mut hasher = Sha384::new();
    for (version, sum) in legacy {
        hasher.update(version.to_be_bytes());
        hasher.update(sum);
    }
    hex::encode(hasher.finalize())
}

async fn record_only(
    client: &mut tokio_postgres::Client,
    baseline: &[&Migration],
) -> Result<(), String> {
    let tx = client
        .transaction()
        .await
        .map_err(|e| format!("トランザクション開始に失敗: {e}"))?;
    for m in baseline {
        insert_migration_tx(&tx, m, &checksum(m.sql), 0)
            .await
            .map_err(|e| format!("土台 {} の記録に失敗(記録は全て取り消し): {e}", m.version))?;
    }
    tx.commit().await.map_err(|e| {
        format!(
            "土台 {}〜{} の記録のコミットに失敗(記録は全て取り消し): {e}",
            baseline.first().map_or(0, |m| m.version),
            baseline.last().map_or(0, |m| m.version)
        )
    })
}

async fn apply(client: &mut tokio_postgres::Client, m: &Migration) -> Result<(), String> {
    let sum = checksum(m.sql);
    if m.no_tx {
        insert_migration(client, m, &sum, false, 0).await?;
        let started = Instant::now();
        client
            .batch_execute(m.sql)
            .await
            .map_err(|e| format_migration_error(&e, m.sql))?;
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
        .map_err(|e| format_migration_error(&e, m.sql))?;
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
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use std::collections::HashMap;

    fn archive_checksums() -> Vec<(i64, Vec<u8>)> {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations/archive");
        let mut files: Vec<std::path::PathBuf> = std::fs::read_dir(&dir)
            .expect("archive の読み取りに失敗")
            .map(|entry| entry.expect("archive の要素の取得に失敗").path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "sql"))
            .collect();
        files.sort();
        files
            .iter()
            .map(|path| {
                let name = path.file_name().unwrap().to_string_lossy();
                let version: i64 = name[..4].parse().expect("版は数値");
                let sql = std::fs::read_to_string(path).expect("archive の読み取りに失敗");
                (version, super::checksum(&sql))
            })
            .collect()
    }

    fn legacy_applied() -> HashMap<i64, Vec<u8>> {
        archive_checksums().into_iter().collect()
    }

    fn versions(migrations: &[&super::Migration]) -> Vec<i64> {
        migrations.iter().map(|m| m.version).collect()
    }

    fn baseline_versions() -> Vec<i64> {
        super::migrations()
            .unwrap()
            .iter()
            .map(|m| m.version)
            .filter(|version| *version <= super::BASELINE_LAST_VERSION)
            .collect()
    }

    #[test]
    fn legacy_history_digest_matches_archive() {
        let checksums = archive_checksums();
        assert_eq!(
            checksums.last().map(|(version, _)| *version),
            Some(super::LEGACY_LAST_VERSION)
        );
        let legacy: Vec<(i64, &Vec<u8>)> = checksums.iter().map(|(v, sum)| (*v, sum)).collect();
        assert_eq!(
            super::legacy_history_digest(&legacy),
            super::LEGACY_HISTORY_SHA384
        );
    }

    #[test]
    fn baseline_follows_legacy_and_ends_at_last_version() {
        let baseline = baseline_versions();
        assert!(baseline.iter().all(|v| *v > super::LEGACY_LAST_VERSION));
        assert_eq!(baseline.last(), Some(&super::BASELINE_LAST_VERSION));
    }

    #[test]
    fn empty_database_applies_baseline() {
        let local = super::migrations().unwrap();
        let plan = super::plan(&HashMap::new(), &local).unwrap();
        assert!(plan.record_only.is_empty());
        assert_eq!(
            versions(&plan.pending),
            local.iter().map(|m| m.version).collect::<Vec<_>>()
        );
    }

    #[test]
    fn full_legacy_history_records_baseline_without_applying() {
        let local = super::migrations().unwrap();
        let plan = super::plan(&legacy_applied(), &local).unwrap();
        assert_eq!(versions(&plan.record_only), baseline_versions());
        assert_eq!(
            versions(&plan.pending),
            local
                .iter()
                .map(|m| m.version)
                .filter(|version| *version > super::BASELINE_LAST_VERSION)
                .collect::<Vec<_>>(),
            "土台の後ろの migration だけを当てる"
        );
    }

    #[test]
    fn recorded_baseline_on_legacy_database_is_not_recorded_again() {
        let local = super::migrations().unwrap();
        let mut applied = legacy_applied();
        for m in &local {
            applied.insert(m.version, super::checksum(m.sql));
        }
        let plan = super::plan(&applied, &local).unwrap();
        assert!(plan.record_only.is_empty());
        assert!(plan.pending.is_empty());
    }

    #[test]
    fn partially_applied_legacy_history_stops() {
        let local = super::migrations().unwrap();
        let mut applied = legacy_applied();
        applied.retain(|version, _| *version <= 20);
        let err = super::plan(&applied, &local).err().unwrap();
        assert!(err.contains("0020 までしか適用されていません"), "{err}");
    }

    #[test]
    fn legacy_history_with_other_content_stops() {
        let local = super::migrations().unwrap();
        let mut applied = legacy_applied();
        applied.insert(5, vec![0]);
        let err = super::plan(&applied, &local).err().unwrap();
        assert!(err.contains("archive と一致しません"), "{err}");
    }

    #[test]
    fn legacy_history_with_missing_version_stops() {
        let local = super::migrations().unwrap();
        let mut applied = legacy_applied();
        applied.remove(&5);
        let err = super::plan(&applied, &local).err().unwrap();
        assert!(err.contains("archive と一致しません"), "{err}");
    }

    #[test]
    fn partially_recorded_baseline_on_legacy_database_stops() {
        let local = super::migrations().unwrap();
        let mut applied = legacy_applied();
        let first = &local[0];
        applied.insert(first.version, super::checksum(first.sql));
        let err = super::plan(&applied, &local).err().unwrap();
        assert!(err.contains("土台の記録が 1/"), "{err}");
    }

    #[test]
    fn unknown_version_stops() {
        let local = super::migrations().unwrap();
        let applied = HashMap::from([(99, vec![0])]);
        let err = super::plan(&applied, &local).err().unwrap();
        assert!(err.contains("対応するファイルがありません"), "{err}");
    }

    #[test]
    fn all_migration_files_are_registered() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations");
        let mut files: Vec<String> = std::fs::read_dir(&dir)
            .expect("migrations ディレクトリの読み取りに失敗")
            .map(|entry| {
                entry
                    .expect("migrations の要素の取得に失敗")
                    .file_name()
                    .to_string_lossy()
                    .into_owned()
            })
            .filter(|name| name.ends_with(".sql"))
            .collect();
        files.sort();
        let registered: Vec<String> = super::MIGRATION_FILES
            .iter()
            .map(|(name, _)| format!("{name}.sql"))
            .collect();
        assert_eq!(
            files, registered,
            "migrations の .sql が MIGRATION_FILES に過不足なく登録されているはず"
        );
    }

    #[test]
    fn extracts_sql_line_from_postgres_character_position() {
        let sql = "-- 日本語\nCREATE INDEX broken";
        let position = u32::try_from(
            sql.chars()
                .position(|character| character == 'b')
                .expect("対象文字がある"),
        )
        .expect("対象文字の位置は u32 に収まる")
            + 1;

        assert_eq!(
            super::sql_line_at_position(sql, position),
            Some("CREATE INDEX broken")
        );
    }
}
