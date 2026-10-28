use crate::db::{Bind, Db};
use crate::errors::ApiError;
use chrono::{DateTime, Utc};
use shared::csv_kind_stats::{CsvKindStat, CsvKindStatsResponse};
use shared::value::UserId;

/// 5表の件数・最終作成日時と履歴の最終取込日時を1回の問い合わせで取る。
/// 履歴結合・順序は `ORDER BY ord` で固定し、表示順の決定は SQL が担う
const STATS_SQL: &str = "SELECT agg.kind, agg.count, agg.max_created_at, h.last_imported_at \
    FROM ( \
        SELECT 'assetbalance' AS kind, 0 AS ord, COUNT(*) AS count, MAX(created_at) AS max_created_at FROM asset_balances WHERE user_id = $1 \
        UNION ALL \
        SELECT 'fundbalance', 1, COUNT(*), MAX(created_at) FROM fund_balances WHERE user_id = $1 \
        UNION ALL \
        SELECT 'dividend', 2, COUNT(*), MAX(created_at) FROM dividends WHERE user_id = $1 \
        UNION ALL \
        SELECT 'domesticstock', 3, COUNT(*), MAX(created_at) FROM domestic_stocks WHERE user_id = $1 \
        UNION ALL \
        SELECT 'mutualfund', 4, COUNT(*), MAX(created_at) FROM mutualfunds WHERE user_id = $1 \
    ) AS agg \
    LEFT JOIN csv_import_history h ON h.user_id = $1 AND h.kind = agg.kind \
    ORDER BY agg.ord";

/// 種類ごとの表示名。並びは `STATS_SQL` の `ORDER BY ord` と同じにする
const KIND_LABELS: [(&str, &str); 5] = [
    ("assetbalance", "資産管理(国内株式)"),
    ("fundbalance", "資産管理(投資信託)"),
    ("dividend", "取引明細(配当金)"),
    ("domesticstock", "取引明細(国内株式)"),
    ("mutualfund", "取引明細(投資信託)"),
];

struct CsvKindStatRow {
    kind: String,
    count: i64,
    max_created_at: Option<DateTime<Utc>>,
    last_imported_at: Option<DateTime<Utc>>,
}

impl crate::db::FromRow for CsvKindStatRow {
    fn from_row(row: &crate::db::Row) -> Result<Self, crate::db::DbError> {
        Ok(Self {
            kind: row.try_get("kind")?,
            count: row.try_get("count")?,
            max_created_at: row.try_get("max_created_at")?,
            last_imported_at: row.try_get("last_imported_at")?,
        })
    }
}

/// 履歴があれば履歴を優先し、なければ直近の作成日時を使う。
/// 件数0(未取込・全削除後)は履歴が残っていても日時なしにする
fn resolve_latest(
    count: i64,
    max_created_at: Option<DateTime<Utc>>,
    last_imported_at: Option<DateTime<Utc>>,
) -> Option<String> {
    if count == 0 {
        return None;
    }
    last_imported_at
        .or(max_created_at)
        .map(|timestamp| timestamp.to_rfc3339())
}

pub async fn collect_csv_kind_stats(
    pool: &Db,
    user_id: UserId,
) -> Result<CsvKindStatsResponse, ApiError> {
    let rows = crate::db::query(STATS_SQL, vec![Bind::from(user_id)])
        .fetch_all::<CsvKindStatRow, _>(pool)
        .await?;
    let labels: std::collections::HashMap<&str, &str> = KIND_LABELS.into_iter().collect();
    let kinds = rows
        .into_iter()
        .map(|row| CsvKindStat {
            label: labels
                .get(row.kind.as_str())
                .copied()
                .unwrap_or_default()
                .to_string(),
            kind: row.kind,
            latest_imported_at: resolve_latest(row.count, row.max_created_at, row.last_imported_at),
            count: row.count,
        })
        .collect();
    Ok(CsvKindStatsResponse { kinds })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn timestamp(value: &str) -> Option<DateTime<Utc>> {
        Some(
            chrono::DateTime::parse_from_rfc3339(value)
                .unwrap()
                .with_timezone(&Utc),
        )
    }

    #[test]
    fn history_takes_precedence_over_max_created_at() {
        let latest = resolve_latest(
            2,
            timestamp("2026-10-24T02:15:00Z"),
            timestamp("2026-10-27T04:30:00Z"),
        );
        assert_eq!(latest.as_deref(), Some("2026-10-27T04:30:00+00:00"));
    }

    #[test]
    fn falls_back_to_max_created_at_without_history() {
        let latest = resolve_latest(2, timestamp("2026-10-24T02:15:00Z"), None);
        assert_eq!(latest.as_deref(), Some("2026-10-24T02:15:00+00:00"));
    }

    #[test]
    fn zero_count_hides_timestamp_despite_history() {
        let latest = resolve_latest(0, None, timestamp("2026-10-24T02:15:00Z"));
        assert_eq!(latest, None);
    }

    #[test]
    fn zero_count_without_history_has_no_timestamp() {
        assert_eq!(resolve_latest(0, None, None), None);
    }
}
