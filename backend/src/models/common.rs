use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// ページネーションクエリパラメータ（全ドメイン共通）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationParams {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

impl PaginationParams {
    pub fn page(&self) -> i64 {
        self.page.unwrap_or(1).max(1)
    }
    pub fn per_page(&self) -> i64 {
        self.per_page.unwrap_or(200).clamp(1, 1000)
    }
    pub fn offset(&self) -> i64 {
        (self.page() - 1) * self.per_page()
    }
}

/// ページネーションレスポンス（全ドメイン共通）
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PaginatedResponse<T: ToSchema + 'static> {
    pub data: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

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
    fn test_serde_round_trips() {
        let r = BulkCreateResponse {
            inserted: 3,
            skipped: 2,
        };
        let json = serde_json::to_string(&r).expect("BulkCreateResponse should serialize");
        let d: BulkCreateResponse =
            serde_json::from_str(&json).expect("BulkCreateResponse should deserialize");
        assert_eq!((d.inserted, d.skipped), (r.inserted, r.skipped));
        let r = MessageResponse {
            message: "ok".to_string(),
        };
        let json = serde_json::to_string(&r).expect("MessageResponse should serialize");
        let d: MessageResponse =
            serde_json::from_str(&json).expect("MessageResponse should deserialize");
        assert_eq!(d.message, r.message);
    }
}
