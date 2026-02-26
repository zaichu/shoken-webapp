use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

/// 国内株式取引モデル（DB + APIレスポンス兼用）
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct DomesticStock {
    pub id: Uuid,
    #[serde(skip_serializing)]
    #[allow(dead_code)]
    pub user_id: Uuid,
    pub trade_date: NaiveDate,
    pub settlement_date: NaiveDate,
    pub security_code: String,
    pub security_name: String,
    pub account: String,
    pub shares: f64,
    pub asked_price: f64,
    pub proceeds: f64,
    pub purchase_price: f64,
    pub realized_profit_and_loss: f64,
    pub taxes: f64,
    pub realized_profit_and_loss_after_tax: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 国内株式取引作成リクエスト
#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateDomesticStockRequest {
    pub trade_date: NaiveDate,
    pub settlement_date: NaiveDate,
    #[validate(length(min = 1, max = 10))]
    pub security_code: String,
    #[validate(length(min = 1, max = 200))]
    pub security_name: String,
    #[validate(length(min = 1, max = 100))]
    pub account: String,
    pub shares: f64,
    pub asked_price: f64,
    pub proceeds: f64,
    pub purchase_price: f64,
    pub realized_profit_and_loss: f64,
    pub taxes: f64,
    pub realized_profit_and_loss_after_tax: f64,
}

/// 国内株式取引一括作成リクエスト
#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct BulkCreateDomesticStockRequest {
    #[validate(length(min = 1))]
    pub items: Vec<CreateDomesticStockRequest>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_domestic_stock_request_validation() {
        let request = CreateDomesticStockRequest {
            trade_date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            settlement_date: NaiveDate::from_ymd_opt(2024, 1, 17).unwrap(),
            security_code: "1234".to_string(),
            security_name: "テスト株式会社".to_string(),
            account: "特定".to_string(),
            shares: 100.0,
            asked_price: 1500.0,
            proceeds: 150000.0,
            purchase_price: 1400.0,
            realized_profit_and_loss: 10000.0,
            taxes: 2000.0,
            realized_profit_and_loss_after_tax: 8000.0,
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_bulk_create_request_allows_duplicate_rows() {
        // 同一内容の行が複数あっても BulkCreateDomesticStockRequest のバリデーションは通る
        let item = CreateDomesticStockRequest {
            trade_date: NaiveDate::from_ymd_opt(2026, 2, 12).unwrap(),
            settlement_date: NaiveDate::from_ymd_opt(2026, 2, 16).unwrap(),
            security_code: "9508".to_string(),
            security_name: "九州電力".to_string(),
            account: "特定".to_string(),
            shares: 100.0,
            asked_price: 1880.0,
            proceeds: 188000.0,
            purchase_price: 1770.0,
            realized_profit_and_loss: 11000.0,
            taxes: 2234.0,
            realized_profit_and_loss_after_tax: 8766.0,
        };
        let bulk = BulkCreateDomesticStockRequest {
            items: vec![item.clone(), item.clone(), item.clone(), item.clone(), item],
        };
        assert!(bulk.validate().is_ok());
        assert_eq!(bulk.items.len(), 5);
    }
}
