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
#[cfg(test)] #[rustfmt::skip] mod tests {
    use super::{BulkCreateResponse, MessageResponse};
    #[test] fn test_serde_round_trips() {
        let r = BulkCreateResponse { inserted: 3, skipped: 2 }; let json = serde_json::to_string(&r).expect("BulkCreateResponse should serialize"); let d: BulkCreateResponse = serde_json::from_str(&json).expect("BulkCreateResponse should deserialize"); assert_eq!((d.inserted, d.skipped), (r.inserted, r.skipped));
        let r = MessageResponse { message: "ok".to_string() }; let json = serde_json::to_string(&r).expect("MessageResponse should serialize"); let d: MessageResponse = serde_json::from_str(&json).expect("MessageResponse should deserialize"); assert_eq!(d.message, r.message);
    }
}
