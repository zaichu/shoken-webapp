use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use validator::Validate;

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
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct DividendCache {
    pub security_code: String,
    pub dividend_per_share: Option<f64>,
    /// ok: 有配当、zero: ゼロ配当、error: 取得失敗、pending: 未取得/更新待ち
    pub status: String,
    pub fetched_at: Option<DateTime<Utc>>,
    /// TTL期限。この時刻を過ぎると再取得対象（NULL = 即再取得対象）
    pub stale_at: Option<DateTime<Utc>>,
    pub source: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// バッチリクエスト
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct DividendPerShareBatchRequest {
    #[validate(length(min = 1, max = 100))]
    pub security_codes: Vec<String>,
}

/// レスポンス内の1銘柄アイテム
#[derive(Debug, Serialize, ToSchema)]
pub struct DividendPerShareItem {
    pub security_code: String,
    pub dividend_per_share: Option<f64>,
    /// ok / zero / pending / error
    pub status: String,
    pub fetched_at: Option<DateTime<Utc>>,
    pub is_stale: bool,
}

/// バッチレスポンス
#[derive(Debug, Serialize, ToSchema)]
pub struct DividendPerShareBatchResponse {
    pub items: Vec<DividendPerShareItem>,
}

