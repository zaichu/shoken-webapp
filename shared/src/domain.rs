// `typed` feature 有効時は正規型、無効時は wire 上の文字列型になる。serde の wire 形は同一で、
// utoipa/sqlx の derive が型名をそのまま見るため、フィールド宣言は共通のまま型の実体だけを切り替える。
// user_id は backend 専用（wire に出ない）なので typed 時のみ存在させる。
#[cfg(feature = "typed")]
pub use chrono::{DateTime, NaiveDate, Utc};
#[cfg(feature = "typed")]
pub use uuid::Uuid;
#[cfg(not(feature = "typed"))]
pub type Uuid = String;
#[cfg(not(feature = "typed"))]
pub type NaiveDate = String;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// 国内株式取引モデル（DB + APIレスポンス兼用）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct DomesticStock {
    pub id: Uuid,
    #[cfg(feature = "typed")]
    #[serde(default, skip_serializing)]
    pub user_id: Uuid,
    pub trade_date: NaiveDate,
    pub settlement_date: NaiveDate,
    pub security_code: String,
    pub security_name: String,
    pub account: String,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub shares: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub asked_price: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub proceeds: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub purchase_price: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub realized_profit_and_loss: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub taxes: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub realized_profit_and_loss_after_tax: Decimal,
    #[cfg(feature = "typed")]
    pub created_at: DateTime<Utc>,
    #[cfg(not(feature = "typed"))]
    pub created_at: String,
    #[cfg(feature = "typed")]
    pub updated_at: DateTime<Utc>,
    #[cfg(not(feature = "typed"))]
    pub updated_at: String,
}

/// 国内株式取引 検索条件全体の集計
///
/// trade_date ごとに特定口座（account に「特定」を含む）と NISA 等口座を分離し、
/// 特定口座の実現損益合計がプラスの時だけ `floor(合計 * 0.20315)` を日次税額として
/// 日次集計した結果を合計する。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct DomesticStockSummary {
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub total_realized_profit_and_loss: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub total_taxes: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub total_realized_profit_and_loss_after_tax: Decimal,
}

/// 配当金モデル（DB + APIレスポンス兼用）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct Dividend {
    pub id: Uuid,
    #[cfg(feature = "typed")]
    #[serde(default, skip_serializing)]
    pub user_id: Uuid,
    pub settlement_date: NaiveDate,
    pub product: String,
    pub account: String,
    pub security_code: String,
    pub security_name: String,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub unit_price: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub shares: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub dividends_before_tax: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub taxes: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub net_amount_received: Decimal,
    #[cfg(feature = "typed")]
    pub created_at: DateTime<Utc>,
    #[cfg(not(feature = "typed"))]
    pub created_at: String,
    #[cfg(feature = "typed")]
    pub updated_at: DateTime<Utc>,
    #[cfg(not(feature = "typed"))]
    pub updated_at: String,
}

/// 配当金 検索条件全体の集計
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct DividendSummary {
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub total_dividends_before_tax: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub total_taxes: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub total_net_amount_received: Decimal,
}

/// 投資信託モデル（DB + APIレスポンス兼用）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct Mutualfund {
    pub id: Uuid,
    #[cfg(feature = "typed")]
    #[serde(default, skip_serializing)]
    pub user_id: Uuid,
    pub trade_date: NaiveDate,
    pub settlement_date: NaiveDate,
    pub fund_name: String,
    pub dividends: Option<String>,
    pub account: String,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub shares: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub exchange_rate: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub cancellation_unit_price_yen: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub cancellation_amount_yen: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub average_acquisition_price_yen: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub realized_profit_and_loss: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub taxes: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub realized_profit_and_loss_after_tax: Decimal,
    #[cfg(feature = "typed")]
    pub created_at: DateTime<Utc>,
    #[cfg(not(feature = "typed"))]
    pub created_at: String,
    #[cfg(feature = "typed")]
    pub updated_at: DateTime<Utc>,
    #[cfg(not(feature = "typed"))]
    pub updated_at: String,
}

/// 投資信託 検索条件全体の集計
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct MutualfundSummary {
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub total_realized_profit_and_loss: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub total_taxes: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub total_realized_profit_and_loss_after_tax: Decimal,
}

/// 保有銘柄モデル（DB + APIレスポンス兼用）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct AssetBalance {
    pub id: Uuid,
    #[cfg(feature = "typed")]
    #[serde(default, skip_serializing)]
    pub user_id: Uuid,
    pub security_code: String,
    pub security_name: String,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub shares: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub executing_shares: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub average_purchase_price: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub total_purchase_amount: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub current_price: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub daily_change: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub market_value: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub profit_loss_rate: Decimal,
    #[cfg(feature = "typed")]
    pub created_at: DateTime<Utc>,
    #[cfg(not(feature = "typed"))]
    pub created_at: String,
    #[cfg(feature = "typed")]
    pub updated_at: DateTime<Utc>,
    #[cfg(not(feature = "typed"))]
    pub updated_at: String,
}

/// 保有銘柄 検索条件全体の集計
///
/// profit_loss_rate は銘柄ごとの比率のため単純合算せず、summary には含めない。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct AssetBalanceSummary {
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub total_market_value: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub total_purchase_amount: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub total_daily_change: Decimal,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn domestic_stock_omits_user_id_and_accepts_its_absence() {
        let stock: DomesticStock = serde_json::from_str(
            r#"{
                "id": "550e8400-e29b-41d4-a716-446655440000",
                "trade_date": "2024-01-15",
                "settlement_date": "2024-01-17",
                "security_code": "1301",
                "security_name": "極洋",
                "account": "特定",
                "shares": 100,
                "asked_price": 1500.5,
                "proceeds": 150050,
                "purchase_price": 1400.25,
                "realized_profit_and_loss": 10000.1,
                "taxes": 2031.5,
                "realized_profit_and_loss_after_tax": 7968.6,
                "created_at": "2024-01-15T01:23:45Z",
                "updated_at": "2024-01-16T01:23:45Z"
            }"#,
        )
        .expect("deserialize");

        let json = serde_json::to_value(&stock).expect("serialize");
        assert!(json.get("user_id").is_none());
        assert_eq!(json["trade_date"], "2024-01-15");
        assert_eq!(json["created_at"], "2024-01-15T01:23:45Z");
        assert_eq!(json["id"], "550e8400-e29b-41d4-a716-446655440000");
    }
}
