use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// 投資信託モデル（DB + APIレスポンス兼用）
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Mutualfund {
    pub id: Uuid,
    #[serde(skip_serializing)]
    #[allow(dead_code)]
    pub user_id: Uuid,
    pub trade_date: NaiveDate,
    pub settlement_date: NaiveDate,
    pub fund_name: String,
    pub dividends: Option<String>,
    pub account: String,
    pub shares: f64,
    pub exchange_rate: f64,
    pub cancellation_unit_price_yen: f64,
    pub cancellation_amount_yen: f64,
    pub average_acquisition_price_yen: f64,
    pub realized_profit_and_loss: f64,
    pub taxes: f64,
    pub realized_profit_and_loss_after_tax: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 投資信託作成リクエスト
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateMutualfundRequest {
    pub trade_date: NaiveDate,
    pub settlement_date: NaiveDate,
    #[validate(length(min = 1, max = 300))]
    pub fund_name: String,
    #[validate(length(max = 100))]
    pub dividends: Option<String>,
    #[validate(length(min = 1, max = 100))]
    pub account: String,
    pub shares: f64,
    pub exchange_rate: f64,
    pub cancellation_unit_price_yen: f64,
    pub cancellation_amount_yen: f64,
    pub average_acquisition_price_yen: f64,
    pub realized_profit_and_loss: f64,
    pub taxes: f64,
    pub realized_profit_and_loss_after_tax: f64,
}

/// 投資信託一括作成リクエスト
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct BulkCreateMutualfundRequest {
    #[validate(length(min = 1))]
    pub items: Vec<CreateMutualfundRequest>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_mutualfund_request_validation() {
        let request = CreateMutualfundRequest {
            trade_date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            settlement_date: NaiveDate::from_ymd_opt(2024, 1, 19).unwrap(),
            fund_name: "テストファンド".to_string(),
            dividends: Some("再投資型".to_string()),
            account: "特定".to_string(),
            shares: 10000.0,
            exchange_rate: 1.0,
            cancellation_unit_price_yen: 15000.0,
            cancellation_amount_yen: 150000.0,
            average_acquisition_price_yen: 14000.0,
            realized_profit_and_loss: 10000.0,
            taxes: 2000.0,
            realized_profit_and_loss_after_tax: 8000.0,
        };

        assert!(request.validate().is_ok());
    }
}
