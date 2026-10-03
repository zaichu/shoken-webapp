use crate::api::dto::CsvPreviewResponse;
use crate::features::receipts::ReceiptsTab;
use crate::support::csv_flow::CsvPreview;
use rust_decimal::Decimal;
use serde::Deserialize;

// プレビュー行は backend が Create*Request をシリアライズしたもので id・タイムスタンプを持たないため、全フィールドを lenient に受け取る(銘柄コード・口座は未検証の生値)
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct DividendCsvRow {
    #[serde(default)]
    pub settlement_date: String,
    #[serde(default)]
    pub product: String,
    #[serde(default)]
    pub account: String,
    #[serde(default)]
    pub security_code: String,
    #[serde(default)]
    pub security_name: String,
    #[serde(default)]
    pub unit_price: Decimal,
    #[serde(default)]
    pub shares: Decimal,
    #[serde(default)]
    pub dividends_before_tax: Decimal,
    #[serde(default)]
    pub taxes: Decimal,
    #[serde(default)]
    pub net_amount_received: Decimal,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct DomesticStockCsvRow {
    #[serde(default)]
    pub trade_date: String,
    #[serde(default)]
    pub settlement_date: String,
    #[serde(default)]
    pub security_code: String,
    #[serde(default)]
    pub security_name: String,
    #[serde(default)]
    pub account: String,
    #[serde(default)]
    pub shares: Decimal,
    #[serde(default)]
    pub asked_price: Decimal,
    #[serde(default)]
    pub proceeds: Decimal,
    #[serde(default)]
    pub purchase_price: Decimal,
    #[serde(default)]
    pub realized_profit_and_loss: Decimal,
    #[serde(default)]
    pub taxes: Decimal,
    #[serde(default)]
    pub realized_profit_and_loss_after_tax: Decimal,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct MutualfundCsvRow {
    #[serde(default)]
    pub trade_date: String,
    #[serde(default)]
    pub settlement_date: String,
    #[serde(default)]
    pub fund_name: String,
    #[serde(default)]
    pub dividends: Option<String>,
    #[serde(default)]
    pub account: String,
    #[serde(default)]
    pub shares: Decimal,
    #[serde(default)]
    pub exchange_rate: Decimal,
    #[serde(default)]
    pub cancellation_unit_price_yen: Decimal,
    #[serde(default)]
    pub cancellation_amount_yen: Decimal,
    #[serde(default)]
    pub average_acquisition_price_yen: Decimal,
    #[serde(default)]
    pub realized_profit_and_loss: Decimal,
    #[serde(default)]
    pub taxes: Decimal,
    #[serde(default)]
    pub realized_profit_and_loss_after_tax: Decimal,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CsvPreviewRow {
    Dividend(DividendCsvRow),
    DomesticStock(DomesticStockCsvRow),
    MutualFund(MutualfundCsvRow),
}

pub fn to_preview(tab: ReceiptsTab, response: CsvPreviewResponse) -> CsvPreview<CsvPreviewRow> {
    CsvPreview::from_response(response, |row| tab.parse_csv_row(row))
}

#[cfg(test)]
mod tests;
