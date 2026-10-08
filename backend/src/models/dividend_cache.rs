use chrono::{DateTime, Utc};
use serde::Serialize;
use shared::{dividend_per_share::DividendCacheStatus, value::SecurityCode};

pub use shared::dividend_per_share::{
    DividendPerShareBatchRequest, DividendPerShareBatchResponse, DividendPerShareItem,
};

/// 配当キャッシュレコード
///
/// status × stale_at 整合ルール（is_stale / 再取得対象の判定基準）
/// | status  | stale_at    | is_stale | 再取得? | 理由                         |
/// |---------|-------------|----------|---------|------------------------------|
/// | pending | NULL        | false    | No      | 取得中のため再取得しない     |
/// | ok      | future      | false    | No      | 有効データ                   |
/// | ok      | NULL / past | true     | Yes     | stale（再取得待ち）          |
/// | zero    | future      | false    | No      | 配当なし（有効）             |
/// | zero    | NULL / past | true     | Yes     | stale（再取得待ち）          |
/// | error   | NULL        | true     | Yes     | 即再取得対象                 |
/// | error   | future      | false    | No      | 429 cooldown 中は再取得しない |
#[derive(Debug, Clone, Serialize)]
pub struct DividendCache {
    pub security_code: SecurityCode,
    pub dividend_per_share: Option<f64>,
    /// ok: 有配当、zero: ゼロ配当、error: 取得失敗、pending: 未取得/更新待ち
    pub status: DividendCacheStatus,
    pub fetched_at: Option<DateTime<Utc>>,
    /// TTL期限。この時刻を過ぎると再取得対象（NULL かつ pending 以外 = 即再取得対象）
    pub stale_at: Option<DateTime<Utc>>,
    pub provider: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl crate::db::FromRow for DividendCache {
    fn from_row(row: &crate::db::Row) -> Result<Self, crate::db::DbError> {
        Ok(Self {
            security_code: row.try_get("security_code")?,
            dividend_per_share: row.try_get("dividend_per_share")?,
            status: row.try_get("status")?,
            fetched_at: row.try_get("fetched_at")?,
            stale_at: row.try_get("stale_at")?,
            provider: row.try_get("provider")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}
