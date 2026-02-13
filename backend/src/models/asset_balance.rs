use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// 保有銘柄モデル（DB + APIレスポンス兼用）
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AssetBalance {
    pub id: Uuid,
    #[serde(skip_serializing)]
    #[allow(dead_code)]
    pub user_id: Uuid,
    pub security_code: String,
    pub security_name: String,
    pub shares: f64,
    pub executing_shares: f64,
    pub average_purchase_price: f64,
    pub total_purchase_amount: f64,
    pub current_price: f64,
    pub daily_change: f64,
    pub market_value: f64,
    pub profit_loss_rate: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 保有銘柄作成リクエスト
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateAssetBalanceRequest {
    #[validate(length(min = 1, max = 10))]
    pub security_code: String,
    #[validate(length(min = 1, max = 200))]
    pub security_name: String,
    pub shares: f64,
    pub executing_shares: f64,
    pub average_purchase_price: f64,
    pub total_purchase_amount: f64,
    pub current_price: f64,
    pub daily_change: f64,
    pub market_value: f64,
    pub profit_loss_rate: f64,
}

/// 保有銘柄一括作成リクエスト
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct BulkCreateAssetBalanceRequest {
    #[validate(length(min = 1))]
    pub items: Vec<CreateAssetBalanceRequest>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_asset_balance_request_validation() {
        let request = CreateAssetBalanceRequest {
            security_code: "1234".to_string(),
            security_name: "テスト株式会社".to_string(),
            shares: 100.0,
            executing_shares: 0.0,
            average_purchase_price: 1500.0,
            total_purchase_amount: 150000.0,
            current_price: 1600.0,
            daily_change: 10.0,
            market_value: 160000.0,
            profit_loss_rate: 6.67,
        };

        assert!(request.validate().is_ok());
    }
}
