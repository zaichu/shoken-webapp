use serde::{Deserialize, Serialize};

/// 一括作成レスポンス（全ドメイン共通）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkCreateResponse {
    pub inserted: usize,
    pub skipped: usize,
}
