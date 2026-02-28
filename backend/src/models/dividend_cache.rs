use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use validator::Validate;

/// 配当キャッシュレコード
///
/// status × stale_at 整合ルール
/// | status  | stale_at         | 意味               | 再取得? |
/// |---------|------------------|--------------------|---------|
/// | pending | NULL             | 初回取得中         | No      |
/// | ok      | future           | 有効データ         | No      |
/// | ok      | NULL / past      | stale（再取得待ち）| Yes     |
/// | zero    | future           | 配当なし（有効）   | No      |
/// | zero    | NULL / past      | stale（再取得待ち）| Yes     |
/// | error   | NULL（即再取得） | エラー             | Yes     |
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

