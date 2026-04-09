use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

/// 投資信託モデル（DB + APIレスポンス兼用）
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
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
    #[schema(value_type = f64)]
    pub shares: Decimal,
    #[schema(value_type = f64)]
    pub exchange_rate: Decimal,
    #[schema(value_type = f64)]
    pub cancellation_unit_price_yen: Decimal,
    #[schema(value_type = f64)]
    pub cancellation_amount_yen: Decimal,
    #[schema(value_type = f64)]
    pub average_acquisition_price_yen: Decimal,
    #[schema(value_type = f64)]
    pub realized_profit_and_loss: Decimal,
    #[schema(value_type = f64)]
    pub taxes: Decimal,
    #[schema(value_type = f64)]
    pub realized_profit_and_loss_after_tax: Decimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 投資信託作成リクエスト
#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateMutualfundRequest {
    pub trade_date: NaiveDate,
    pub settlement_date: NaiveDate,
    #[validate(length(min = 1, max = 300))]
    pub fund_name: String,
    #[validate(length(max = 100))]
    pub dividends: Option<String>,
    #[validate(length(min = 1, max = 100))]
    pub account: String,
    #[schema(value_type = f64)]
    pub shares: Decimal,
    #[schema(value_type = f64)]
    pub exchange_rate: Decimal,
    #[schema(value_type = f64)]
    pub cancellation_unit_price_yen: Decimal,
    #[schema(value_type = f64)]
    pub cancellation_amount_yen: Decimal,
    #[schema(value_type = f64)]
    pub average_acquisition_price_yen: Decimal,
    #[schema(value_type = f64)]
    pub realized_profit_and_loss: Decimal,
    #[schema(value_type = f64)]
    pub taxes: Decimal,
    #[schema(value_type = f64)]
    pub realized_profit_and_loss_after_tax: Decimal,
}

#[cfg(test)] #[rustfmt::skip] mod tests {
    use {super::*, rust_decimal_macros::dec};
    #[test]
    fn test_create_mutualfund_request_validation() { assert!(CreateMutualfundRequest { trade_date: NaiveDate::from_ymd_opt(2024, 1, 15).expect("有効な日付 2024-01-15"), settlement_date: NaiveDate::from_ymd_opt(2024, 1, 19).expect("有効な日付 2024-01-19"), fund_name: "テストファンド".to_string(), dividends: Some("再投資型".to_string()), account: "特定".to_string(), shares: dec!(10000), exchange_rate: dec!(1), cancellation_unit_price_yen: dec!(15000), cancellation_amount_yen: dec!(150000), average_acquisition_price_yen: dec!(14000), realized_profit_and_loss: dec!(10000), taxes: dec!(2000), realized_profit_and_loss_after_tax: dec!(8000) }.validate().is_ok()); }
}
