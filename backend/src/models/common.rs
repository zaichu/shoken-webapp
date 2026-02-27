use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// 一括作成レスポンス（全ドメイン共通）
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BulkCreateResponse {
    pub inserted: usize,
    pub skipped: usize,
}

/// メッセージレスポンス（削除・ログアウト等）
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MessageResponse {
    pub message: String,
}
