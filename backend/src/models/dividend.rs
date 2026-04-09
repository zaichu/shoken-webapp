use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
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
    #[schema(value_type = f64)]
    pub unit_price: Decimal,
    #[schema(value_type = f64)]
    pub shares: Decimal,
    #[schema(value_type = f64)]
    pub dividends_before_tax: Decimal,
    #[schema(value_type = f64)]
    pub taxes: Decimal,
    #[schema(value_type = f64)]
    pub net_amount_received: Decimal,
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
    #[validate(length(max = 10))]
    pub security_code: String,
    #[validate(length(min = 1, max = 200))]
    pub security_name: String,
    #[schema(value_type = f64)]
    pub unit_price: Decimal,
    #[schema(value_type = f64)]
    pub shares: Decimal,
    #[schema(value_type = f64)]
    pub dividends_before_tax: Decimal,
    #[schema(value_type = f64)]
    pub taxes: Decimal,
    #[schema(value_type = f64)]
    pub net_amount_received: Decimal,
}
#[cfg(test)] #[rustfmt::skip] mod tests {
    use {super::*, rust_decimal_macros::dec};
    #[test] fn test_create_dividend_request_validation() { assert!(CreateDividendRequest { settlement_date: NaiveDate::from_ymd_opt(2024, 1, 15).expect("有効な日付 2024-01-15"), product: "国内株式".to_string(), account: "特定".to_string(), security_code: "1234".to_string(), security_name: "テスト株式会社".to_string(), unit_price: dec!(100), shares: dec!(100), dividends_before_tax: dec!(1000), taxes: dec!(200), net_amount_received: dec!(800) }.validate().is_ok()); }
}
