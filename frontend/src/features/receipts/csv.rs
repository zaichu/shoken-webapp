use crate::api::dto::CsvPreviewResponse;
use crate::features::receipts::{ReceiptItem, ReceiptsTab};
use crate::support::csv_flow::CsvPreview;
use rust_decimal::Decimal;
use serde::Deserialize;
use shared::value::{Account, RecordId, SecurityCode};

// プレビュー行は lenient に受け取るため、パース不能な値は表示用の代替値に畳む
fn preview_account(value: String) -> Account {
    value.parse().unwrap_or_else(|_| "-".parse().unwrap())
}

fn preview_security_code(value: String) -> SecurityCode {
    value.parse().unwrap_or_else(|_| "0".parse().unwrap())
}

// プレビュー行は backend が Create*Request をシリアライズしたもので id・タイムスタンプを持たないため、全フィールドを lenient に受け取る
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

impl From<CsvPreviewRow> for ReceiptItem {
    fn from(row: CsvPreviewRow) -> ReceiptItem {
        match row {
            CsvPreviewRow::Dividend(row) => ReceiptItem::Dividend(crate::api::dto::Dividend {
                id: RecordId::default(),
                settlement_date: row.settlement_date,
                product: row.product,
                account: preview_account(row.account),
                security_code: row.security_code,
                security_name: row.security_name,
                unit_price: row.unit_price,
                shares: row.shares,
                dividends_before_tax: row.dividends_before_tax,
                taxes: row.taxes,
                net_amount_received: row.net_amount_received,
                created_at: String::new(),
                updated_at: String::new(),
            }),
            CsvPreviewRow::DomesticStock(row) => {
                ReceiptItem::DomesticStock(crate::api::dto::DomesticStock {
                    id: RecordId::default(),
                    trade_date: row.trade_date,
                    settlement_date: row.settlement_date,
                    security_code: preview_security_code(row.security_code),
                    security_name: row.security_name,
                    account: preview_account(row.account),
                    shares: row.shares,
                    asked_price: row.asked_price,
                    proceeds: row.proceeds,
                    purchase_price: row.purchase_price,
                    realized_profit_and_loss: row.realized_profit_and_loss,
                    taxes: row.taxes,
                    realized_profit_and_loss_after_tax: row.realized_profit_and_loss_after_tax,
                    created_at: String::new(),
                    updated_at: String::new(),
                })
            }
            CsvPreviewRow::MutualFund(row) => {
                ReceiptItem::MutualFund(crate::api::dto::Mutualfund {
                    id: RecordId::default(),
                    trade_date: row.trade_date,
                    settlement_date: row.settlement_date,
                    fund_name: row.fund_name,
                    account: preview_account(row.account),
                    shares: row.shares,
                    exchange_rate: row.exchange_rate,
                    cancellation_unit_price_yen: row.cancellation_unit_price_yen,
                    cancellation_amount_yen: row.cancellation_amount_yen,
                    average_acquisition_price_yen: row.average_acquisition_price_yen,
                    realized_profit_and_loss: row.realized_profit_and_loss,
                    taxes: row.taxes,
                    realized_profit_and_loss_after_tax: row.realized_profit_and_loss_after_tax,
                    dividends: Some(row.dividends.unwrap_or_default()),
                    created_at: String::new(),
                    updated_at: String::new(),
                })
            }
        }
    }
}

pub fn to_preview(tab: ReceiptsTab, response: CsvPreviewResponse) -> CsvPreview<CsvPreviewRow> {
    CsvPreview::from_response(response, |row| match tab {
        ReceiptsTab::Dividend => {
            CsvPreviewRow::Dividend(serde_json::from_value(row).unwrap_or_default())
        }
        ReceiptsTab::DomesticStock => {
            CsvPreviewRow::DomesticStock(serde_json::from_value(row).unwrap_or_default())
        }
        ReceiptsTab::MutualFund => {
            CsvPreviewRow::MutualFund(serde_json::from_value(row).unwrap_or_default())
        }
    })
}

#[cfg(test)]
mod tests;
