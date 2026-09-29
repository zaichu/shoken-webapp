use crate::api::dto::{AssetBalance, CsvPreviewResponse};
use crate::support::csv_flow::CsvPreview;
use crate::support::row::Row;
use rust_decimal::Decimal;
use serde::Deserialize;

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
}

/// 一覧行と CSV プレビュー行を束ねる。プレビュー行は id・タイムスタンプを持たないため
/// `AssetBalance` に寄せず `Row::Preview` のまま扱う。
pub(crate) type AssetBalanceRow = Row<AssetBalance, AssetBalanceCsvRow>;

/// `AssetBalance`(保存済み) と `AssetBalanceCsvRow`(CSV プレビュー) の共通読み出し面。
/// 検証に通らない値も差し替えず画面に出すため、文字列は生値を返す。
pub(crate) trait AssetBalanceRowData {
    fn security_code(&self) -> &str;
    fn security_name(&self) -> &str;
    fn shares(&self) -> Decimal;
    fn average_purchase_price(&self) -> Decimal;
    fn total_purchase_amount(&self) -> Decimal;
    fn current_price(&self) -> Decimal;
}

impl AssetBalanceRowData for AssetBalance {
    fn security_code(&self) -> &str {
        self.security_code.as_str()
    }
    fn security_name(&self) -> &str {
        &self.security_name
    }
    fn shares(&self) -> Decimal {
        self.shares
    }
    fn average_purchase_price(&self) -> Decimal {
        self.average_purchase_price
    }
    fn total_purchase_amount(&self) -> Decimal {
        self.total_purchase_amount
    }
    fn current_price(&self) -> Decimal {
        self.current_price
    }
}

impl AssetBalanceRowData for AssetBalanceCsvRow {
    fn security_code(&self) -> &str {
        &self.security_code
    }
    fn security_name(&self) -> &str {
        &self.security_name
    }
    fn shares(&self) -> Decimal {
        self.shares
    }
    fn average_purchase_price(&self) -> Decimal {
        self.average_purchase_price
    }
    fn total_purchase_amount(&self) -> Decimal {
        self.total_purchase_amount
    }
    fn current_price(&self) -> Decimal {
        self.current_price
    }
}

impl AssetBalanceRowData for AssetBalanceRow {
    fn security_code(&self) -> &str {
        match self {
            Self::Saved(row) => row.security_code(),
            Self::Preview(row) => row.security_code(),
        }
    }
    fn security_name(&self) -> &str {
        match self {
            Self::Saved(row) => row.security_name(),
            Self::Preview(row) => row.security_name(),
        }
    }
    fn shares(&self) -> Decimal {
        match self {
            Self::Saved(row) => row.shares(),
            Self::Preview(row) => row.shares(),
        }
    }
    fn average_purchase_price(&self) -> Decimal {
        match self {
            Self::Saved(row) => row.average_purchase_price(),
            Self::Preview(row) => row.average_purchase_price(),
        }
    }
    fn total_purchase_amount(&self) -> Decimal {
        match self {
            Self::Saved(row) => row.total_purchase_amount(),
            Self::Preview(row) => row.total_purchase_amount(),
        }
    }
    fn current_price(&self) -> Decimal {
        match self {
            Self::Saved(row) => row.current_price(),
            Self::Preview(row) => row.current_price(),
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
