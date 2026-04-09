use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
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
    #[schema(value_type = f64)]
    pub shares: Decimal,
    #[schema(value_type = f64)]
    pub asked_price: Decimal,
    #[schema(value_type = f64)]
    pub proceeds: Decimal,
    #[schema(value_type = f64)]
    pub purchase_price: Decimal,
    #[schema(value_type = f64)]
    pub realized_profit_and_loss: Decimal,
    #[schema(value_type = f64)]
    pub taxes: Decimal,
    #[schema(value_type = f64)]
    pub realized_profit_and_loss_after_tax: Decimal,
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
    #[schema(value_type = f64)]
    pub shares: Decimal,
    #[schema(value_type = f64)]
    pub asked_price: Decimal,
    #[schema(value_type = f64)]
    pub proceeds: Decimal,
    #[schema(value_type = f64)]
    pub purchase_price: Decimal,
    #[schema(value_type = f64)]
    pub realized_profit_and_loss: Decimal,
    #[schema(value_type = f64)]
    pub taxes: Decimal,
    #[schema(value_type = f64)]
    pub realized_profit_and_loss_after_tax: Decimal,
}

#[cfg(test)]
#[rustfmt::skip]
mod tests {
    use {super::*, rust_decimal_macros::dec};
    #[test]
    fn test_create_domestic_stock_request_validation() { assert!(CreateDomesticStockRequest { trade_date: NaiveDate::from_ymd_opt(2024, 1, 15).expect("有効な日付 2024-01-15"), settlement_date: NaiveDate::from_ymd_opt(2024, 1, 17).expect("有効な日付 2024-01-17"), security_code: "1234".to_string(), security_name: "テスト株式会社".to_string(), account: "特定".to_string(), shares: dec!(100), asked_price: dec!(1500), proceeds: dec!(150000), purchase_price: dec!(1400), realized_profit_and_loss: dec!(10000), taxes: dec!(2000), realized_profit_and_loss_after_tax: dec!(8000) }.validate().is_ok()); }
}
