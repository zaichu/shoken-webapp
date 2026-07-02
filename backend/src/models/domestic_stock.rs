use crate::models::common::validate_length_field;
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::{Validate, ValidationErrors};

/// 国内株式取引モデル（DB + APIレスポンス兼用）
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct DomesticStock {
    pub id: Uuid,
    #[serde(skip_serializing)]
    #[allow(dead_code)] // SELECT * で取得されるがRust側では参照しない行所有者ID
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
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateDomesticStockRequest {
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
}

impl Validate for CreateDomesticStockRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();
        validate_length_field(
            &mut errors,
            "security_code",
            &self.security_code,
            Some(1),
            Some(10),
        );
        validate_length_field(
            &mut errors,
            "security_name",
            &self.security_name,
            Some(1),
            Some(200),
        );
        validate_length_field(&mut errors, "account", &self.account, Some(1), Some(100));
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
#[cfg(test)]
mod tests {
    use {super::*, rust_decimal_macros::dec};

    fn base() -> CreateDomesticStockRequest {
        CreateDomesticStockRequest {
            trade_date: NaiveDate::from_ymd_opt(2024, 1, 15).expect("有効な日付 2024-01-15"),
            settlement_date: NaiveDate::from_ymd_opt(2024, 1, 17).expect("有効な日付 2024-01-17"),
            security_code: "1234".to_string(),
            security_name: "テスト株式会社".to_string(),
            account: "特定".to_string(),
            shares: dec!(100),
            asked_price: dec!(1500),
            proceeds: dec!(150000),
            purchase_price: dec!(1400),
            realized_profit_and_loss: dec!(10000),
            taxes: dec!(2000),
            realized_profit_and_loss_after_tax: dec!(8000),
        }
    }

    #[test]
    fn test_create_domestic_stock_request_validation() {
        assert!(base().validate().is_ok());

        // security_code は max=10 のため 11 文字は NG
        assert!(CreateDomesticStockRequest {
            security_code: "12345678901".to_string(),
            ..base()
        }
        .validate()
        .is_err());

        // security_name は min=1 のため空文字は NG
        assert!(CreateDomesticStockRequest {
            security_name: String::new(),
            ..base()
        }
        .validate()
        .is_err());

        // account は min=1 のため空文字は NG
        assert!(CreateDomesticStockRequest {
            account: String::new(),
            ..base()
        }
        .validate()
        .is_err());
    }
}
