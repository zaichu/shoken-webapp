use crate::models::common::validate_length_field;
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::{Validate, ValidationErrors};

/// 配当金モデル（DB + APIレスポンス兼用）
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Dividend {
    pub id: Uuid,
    #[serde(skip_serializing)]
    #[allow(dead_code)] // SELECT * で取得されるがRust側では参照しない行所有者ID
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
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateDividendRequest {
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
}

impl Validate for CreateDividendRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();
        validate_length_field(&mut errors, "product", &self.product, Some(1), Some(100));
        validate_length_field(&mut errors, "account", &self.account, Some(1), Some(100));
        validate_length_field(
            &mut errors,
            "security_code",
            &self.security_code,
            None,
            Some(10),
        );
        validate_length_field(
            &mut errors,
            "security_name",
            &self.security_name,
            Some(1),
            Some(200),
        );
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
    fn base() -> CreateDividendRequest {
        CreateDividendRequest {
            settlement_date: NaiveDate::from_ymd_opt(2024, 1, 15).expect("有効な日付 2024-01-15"),
            product: "国内株式".to_string(),
            account: "特定".to_string(),
            security_code: "1234".to_string(),
            security_name: "テスト株式会社".to_string(),
            unit_price: dec!(100),
            shares: dec!(100),
            dividends_before_tax: dec!(1000),
            taxes: dec!(200),
            net_amount_received: dec!(800),
        }
    }

    #[test]
    fn test_create_dividend_request_validation() {
        assert!(base().validate().is_ok());

        // security_code は max=10 のため 11 文字は NG
        assert!(CreateDividendRequest {
            security_code: "12345678901".to_string(),
            ..base()
        }
        .validate()
        .is_err());

        // product は min=1 のため空文字は NG
        assert!(CreateDividendRequest {
            product: String::new(),
            ..base()
        }
        .validate()
        .is_err());

        // account は min=1 のため空文字は NG
        assert!(CreateDividendRequest {
            account: String::new(),
            ..base()
        }
        .validate()
        .is_err());

        // security_name は min=1 のため空文字は NG
        assert!(CreateDividendRequest {
            security_name: String::new(),
            ..base()
        }
        .validate()
        .is_err());

        // security_code は空文字でも OK（max=10 のみ）
        assert!(CreateDividendRequest {
            security_code: String::new(),
            ..base()
        }
        .validate()
        .is_ok());
    }
}
