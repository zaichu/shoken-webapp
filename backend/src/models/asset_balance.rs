use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

/// 保有銘柄モデル（DB + APIレスポンス兼用）
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct AssetBalance {
    pub id: Uuid,
    #[serde(skip_serializing)]
    #[allow(dead_code)]
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
#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateAssetBalanceRequest {
    #[validate(length(min = 1, max = 10))]
    pub security_code: String,
    #[validate(length(min = 1, max = 200))]
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

/// 保有銘柄一括作成リクエスト
#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct BulkCreateAssetBalanceRequest {
    #[validate(length(min = 1))]
    pub items: Vec<CreateAssetBalanceRequest>,
}
#[cfg(test)]
mod tests {
    use {super::*, rust_decimal_macros::dec};
    #[test]
    fn test_create_asset_balance_request_validation() {
        assert!(CreateAssetBalanceRequest {
            security_code: "1234".to_string(),
            security_name: "テスト株式会社".to_string(),
            shares: dec!(100),
            executing_shares: dec!(0),
            average_purchase_price: dec!(1500),
            total_purchase_amount: dec!(150000),
            current_price: dec!(1600),
            daily_change: dec!(10),
            market_value: dec!(160000),
            profit_loss_rate: dec!(6.67)
        }
        .validate()
        .is_ok());
    }
}
