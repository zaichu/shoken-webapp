use crate::models::common::validate_length_field;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::{Validate, ValidationErrors};

/// 保有銘柄モデル（DB + APIレスポンス兼用）
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct AssetBalance {
    pub id: Uuid,
    #[serde(skip_serializing)]
    #[allow(dead_code)] // SELECT * で取得されるがRust側では参照しない行所有者ID
    pub user_id: Uuid,
    pub security_code: String,
    pub security_name: String,
    #[schema(value_type = f64)]
    pub shares: Decimal,
    #[schema(value_type = f64)]
    pub executing_shares: Decimal,
    #[schema(value_type = f64)]
    pub average_purchase_price: Decimal,
    #[schema(value_type = f64)]
    pub total_purchase_amount: Decimal,
    #[schema(value_type = f64)]
    pub current_price: Decimal,
    #[schema(value_type = f64)]
    pub daily_change: Decimal,
    #[schema(value_type = f64)]
    pub market_value: Decimal,
    #[schema(value_type = f64)]
    pub profit_loss_rate: Decimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 保有銘柄作成リクエスト
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateAssetBalanceRequest {
    pub security_code: String,
    pub security_name: String,
    #[schema(value_type = f64)]
    pub shares: Decimal,
    #[schema(value_type = f64)]
    pub executing_shares: Decimal,
    #[schema(value_type = f64)]
    pub average_purchase_price: Decimal,
    #[schema(value_type = f64)]
    pub total_purchase_amount: Decimal,
    #[schema(value_type = f64)]
    pub current_price: Decimal,
    #[schema(value_type = f64)]
    pub daily_change: Decimal,
    #[schema(value_type = f64)]
    pub market_value: Decimal,
    #[schema(value_type = f64)]
    pub profit_loss_rate: Decimal,
}

impl Validate for CreateAssetBalanceRequest {
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
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// 保有銘柄一括作成リクエスト
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BulkCreateAssetBalanceRequest {
    pub items: Vec<CreateAssetBalanceRequest>,
}

impl Validate for BulkCreateAssetBalanceRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();
        validate_length_field(&mut errors, "items", &self.items, Some(1), None);
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

    fn base() -> CreateAssetBalanceRequest {
        CreateAssetBalanceRequest {
            security_code: "1234".to_string(),
            security_name: "テスト株式会社".to_string(),
            shares: dec!(100),
            executing_shares: dec!(0),
            average_purchase_price: dec!(1500),
            total_purchase_amount: dec!(150000),
            current_price: dec!(1600),
            daily_change: dec!(10),
            market_value: dec!(160000),
            profit_loss_rate: dec!(6.67),
        }
    }

    #[test]
    fn test_create_asset_balance_request_validation() {
        assert!(base().validate().is_ok());

        // security_code は max=10 のため 11 文字は NG
        assert!(CreateAssetBalanceRequest {
            security_code: "12345678901".to_string(),
            ..base()
        }
        .validate()
        .is_err());

        // security_code は min=1 のため空文字は NG
        assert!(CreateAssetBalanceRequest {
            security_code: String::new(),
            ..base()
        }
        .validate()
        .is_err());

        // security_name は min=1 のため空文字は NG
        assert!(CreateAssetBalanceRequest {
            security_name: String::new(),
            ..base()
        }
        .validate()
        .is_err());
    }

    #[test]
    fn test_bulk_create_asset_balance_request_validation() {
        // items が空の場合は NG（min=1）
        assert!(BulkCreateAssetBalanceRequest { items: vec![] }
            .validate()
            .is_err());

        // items が 1 件以上なら OK
        assert!(BulkCreateAssetBalanceRequest {
            items: vec![base()]
        }
        .validate()
        .is_ok());
    }
}
