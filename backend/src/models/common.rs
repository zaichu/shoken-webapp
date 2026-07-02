use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use utoipa::ToSchema;
use validator::{ValidateLength, ValidationError, ValidationErrors};

/// `validator` derive の length rule 相当を手実装するヘルパー。
/// `String` / `Option<String>` / `Vec<T>` など `ValidateLength` 実装型に共通で使う。
pub(crate) fn validate_length_field<T>(
    errors: &mut ValidationErrors,
    field: &'static str,
    value: T,
    min: Option<u64>,
    max: Option<u64>,
) where
    T: ValidateLength<u64>,
{
    if !value.validate_length(min, max, None) {
        let mut error = ValidationError::new("length");
        if let Some(min) = min {
            error.add_param(Cow::from("min"), &min);
        }
        if let Some(max) = max {
            error.add_param(Cow::from("max"), &max);
        }
        errors.add(field, error);
    }
}

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
