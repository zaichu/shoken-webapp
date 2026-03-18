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

#[cfg(test)]
mod tests {
    use super::{BulkCreateResponse, MessageResponse};

    #[test]
    fn bulk_create_response_round_trips_through_serde() {
        let response = BulkCreateResponse {
            inserted: 3,
            skipped: 2,
        };

        let json = serde_json::to_string(&response).expect("BulkCreateResponse should serialize");
        let deserialized: BulkCreateResponse =
            serde_json::from_str(&json).expect("BulkCreateResponse should deserialize");

        assert_eq!(deserialized.inserted, response.inserted);
        assert_eq!(deserialized.skipped, response.skipped);
    }

    #[test]
    fn message_response_round_trips_through_serde() {
        let response = MessageResponse {
            message: "ok".to_string(),
        };

        let json = serde_json::to_string(&response).expect("MessageResponse should serialize");
        let deserialized: MessageResponse =
            serde_json::from_str(&json).expect("MessageResponse should deserialize");

        assert_eq!(deserialized.message, response.message);
    }
}
