use crate::api::dto::{AssetBalance, CsvPreviewResponse};
use crate::support::csv_flow::CsvPreview;
use rust_decimal::Decimal;
use serde::Deserialize;
use shared::value::{RecordId, SecurityCode};

pub const LIST_PATH: &str = "/api/v1/asset-balances";
pub const PREVIEW_PATH: &str = "/api/v1/asset-balance-import-validations";
pub const IMPORT_PATH: &str = "/api/v1/asset-balance-imports";

// プレビュー行は backend が CreateAssetBalanceRequest をシリアライズしたもので id・タイムスタンプを持たないため、全フィールドを lenient に受け取る
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct AssetBalanceCsvRow {
    #[serde(default)]
    pub security_code: String,
    #[serde(default)]
    pub security_name: String,
    #[serde(default)]
    pub shares: Decimal,
    #[serde(default)]
    pub executing_shares: Decimal,
    #[serde(default)]
    pub average_purchase_price: Decimal,
    #[serde(default)]
    pub total_purchase_amount: Decimal,
    #[serde(default)]
    pub current_price: Decimal,
    #[serde(default)]
    pub daily_change: Decimal,
    #[serde(default)]
    pub market_value: Decimal,
    #[serde(default)]
    pub profit_loss_rate: Decimal,
}

// 一覧表示に載せるため AssetBalance に揃える。id・タイムスタンプは未確定なので空
impl From<AssetBalanceCsvRow> for AssetBalance {
    fn from(row: AssetBalanceCsvRow) -> AssetBalance {
        AssetBalance {
            id: RecordId::default(),
            // 検証に通らない値も差し替えず、そのまま表示する
            security_code: SecurityCode::from_raw(row.security_code),
            security_name: row.security_name,
            shares: row.shares,
            executing_shares: row.executing_shares,
            average_purchase_price: row.average_purchase_price,
            total_purchase_amount: row.total_purchase_amount,
            current_price: row.current_price,
            daily_change: row.daily_change,
            market_value: row.market_value,
            profit_loss_rate: row.profit_loss_rate,
            created_at: String::new(),
            updated_at: String::new(),
        }
    }
}

pub fn to_preview(response: CsvPreviewResponse) -> CsvPreview<AssetBalanceCsvRow> {
    CsvPreview::from_response(response, |row| {
        serde_json::from_value(row).unwrap_or_default()
    })
}

#[cfg(test)]
mod tests;
