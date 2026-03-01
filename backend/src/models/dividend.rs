use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

/// 配当金モデル（DB + APIレスポンス兼用）
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Dividend {
    pub id: Uuid,
    #[serde(skip_serializing)]
    #[allow(dead_code)]
    pub user_id: Uuid,
    pub settlement_date: NaiveDate,
    pub product: String,
    pub account: String,
    pub security_code: String,
    pub security_name: String,
    pub unit_price: f64,
    pub shares: f64,
    pub dividends_before_tax: f64,
    pub taxes: f64,
    pub net_amount_received: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 配当金作成リクエスト
#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateDividendRequest {
    pub settlement_date: NaiveDate,
    #[validate(length(min = 1, max = 100))]
    pub product: String,
    #[validate(length(min = 1, max = 100))]
    pub account: String,
    #[validate(length(min = 1, max = 10))]
    pub security_code: String,
    #[validate(length(min = 1, max = 200))]
    pub security_name: String,
    pub unit_price: f64,
    pub shares: f64,
    pub dividends_before_tax: f64,
    pub taxes: f64,
    pub net_amount_received: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_dividend_request_validation() {
        let request = CreateDividendRequest {
            settlement_date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            product: "国内株式".to_string(),
            account: "特定".to_string(),
            security_code: "1234".to_string(),
            security_name: "テスト株式会社".to_string(),
            unit_price: 100.0,
            shares: 100.0,
            dividends_before_tax: 1000.0,
            taxes: 200.0,
            net_amount_received: 800.0,
        };

        assert!(request.validate().is_ok());
    }
}
