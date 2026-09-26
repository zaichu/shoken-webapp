mod store;

use crate::receipts_domain::{format_currency, format_date, format_number};
use rust_decimal::Decimal;

pub use store::{use_receipts_data, ReceiptsStore};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ReceiptsTab {
    Dividend,
    DomesticStock,
    MutualFund,
}

impl ReceiptsTab {
    pub const ALL: [ReceiptsTab; 3] = [
        ReceiptsTab::Dividend,
        ReceiptsTab::DomesticStock,
        ReceiptsTab::MutualFund,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            ReceiptsTab::Dividend => "配当金",
            ReceiptsTab::DomesticStock => "国内株式",
            ReceiptsTab::MutualFund => "投資信託",
        }
    }

    pub(crate) fn list_path(&self) -> &'static str {
        match self {
            ReceiptsTab::Dividend => "/api/v1/dividends",
            ReceiptsTab::DomesticStock => "/api/v1/domestic-stock-transactions",
            ReceiptsTab::MutualFund => "/api/v1/mutual-fund-transactions",
        }
    }

    pub(crate) fn preview_path(&self) -> &'static str {
        match self {
            ReceiptsTab::Dividend => "/api/v1/dividend-import-validations",
            ReceiptsTab::DomesticStock => "/api/v1/domestic-stock-import-validations",
            ReceiptsTab::MutualFund => "/api/v1/mutual-fund-import-validations",
        }
    }

    pub(crate) fn import_path(&self) -> &'static str {
        match self {
            ReceiptsTab::Dividend => "/api/v1/dividend-imports",
            ReceiptsTab::DomesticStock => "/api/v1/domestic-stock-imports",
            ReceiptsTab::MutualFund => "/api/v1/mutual-fund-imports",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ReceiptItem {
    Dividend(crate::dto::Dividend),
    DomesticStock(crate::dto::DomesticStock),
    MutualFund(crate::dto::Mutualfund),
}

/// テーブル1セルの内容。銘柄コードは `/search` へのリンク、
/// 銘柄名・ファンド名はコピーボタンを出すため、表示文字列とは別に種別を持つ。
#[derive(Clone, Debug, PartialEq)]
pub enum ReceiptCell {
    Text(String),
    SecurityCode(String),
    InstrumentName { name: String, code: Option<String> },
}

#[cfg(test)]
impl ReceiptCell {
    pub fn text(&self) -> &str {
        match self {
            Self::Text(value) | Self::SecurityCode(value) => value,
            Self::InstrumentName { name, .. } => name,
        }
    }
}

impl ReceiptItem {
    pub fn id(&self) -> &str {
        match self {
            ReceiptItem::Dividend(row) => &row.id,
            ReceiptItem::DomesticStock(row) => &row.id,
            ReceiptItem::MutualFund(row) => &row.id,
        }
    }

    pub fn cells(&self) -> Vec<ReceiptCell> {
        match self {
            ReceiptItem::Dividend(row) => vec![
                ReceiptCell::Text(format_date(&row.settlement_date)),
                ReceiptCell::Text(row.product.clone()),
                ReceiptCell::Text(row.account.clone()),
                ReceiptCell::SecurityCode(row.security_code.clone()),
                ReceiptCell::InstrumentName {
                    name: row.security_name.clone(),
                    code: Some(row.security_code.clone()),
                },
                ReceiptCell::Text(format_currency(row.unit_price)),
                ReceiptCell::Text(format_number(row.shares, 2)),
                ReceiptCell::Text(format_currency(row.dividends_before_tax)),
                ReceiptCell::Text(format_currency(row.taxes)),
                ReceiptCell::Text(format_currency(row.net_amount_received)),
            ],
            ReceiptItem::DomesticStock(row) => vec![
                ReceiptCell::Text(format_date(&row.trade_date)),
                ReceiptCell::SecurityCode(row.security_code.clone()),
                ReceiptCell::InstrumentName {
                    name: row.security_name.clone(),
                    code: Some(row.security_code.clone()),
                },
                ReceiptCell::Text(row.account.clone()),
                ReceiptCell::Text(format_number(row.shares, 2)),
                ReceiptCell::Text(format_currency(row.asked_price)),
                ReceiptCell::Text(format_currency(row.proceeds)),
                ReceiptCell::Text(format_currency(row.purchase_price)),
                ReceiptCell::Text(format_currency(row.realized_profit_and_loss)),
                ReceiptCell::Text(format_currency(row.taxes)),
                ReceiptCell::Text(format_currency(row.realized_profit_and_loss_after_tax)),
            ],
            ReceiptItem::MutualFund(row) => vec![
                ReceiptCell::Text(format_date(&row.trade_date)),
                ReceiptCell::InstrumentName {
                    name: row.fund_name.clone(),
                    code: None,
                },
                ReceiptCell::Text(row.account.clone()),
                ReceiptCell::Text(format_number(row.shares, 2)),
                ReceiptCell::Text(format_currency(row.cancellation_unit_price_yen)),
                ReceiptCell::Text(format_currency(row.cancellation_amount_yen)),
                ReceiptCell::Text(format_currency(row.average_acquisition_price_yen)),
                ReceiptCell::Text(format_currency(row.realized_profit_and_loss)),
                ReceiptCell::Text(format_currency(row.taxes)),
                ReceiptCell::Text(format_currency(row.realized_profit_and_loss_after_tax)),
            ],
        }
    }

    // カードの開閉状態を引き継ぐ照合は表示丸め前の値で行う。
    // 数量 1.001 と 1.002 はともに「1.00」と出るが別行として区別する。
    pub fn raw_key(&self) -> String {
        let fields: Vec<String> = match self {
            ReceiptItem::Dividend(row) => vec![
                row.settlement_date.clone(),
                row.product.clone(),
                row.account.clone(),
                row.security_code.clone(),
                row.security_name.clone(),
                row.unit_price.to_string(),
                row.shares.to_string(),
                row.dividends_before_tax.to_string(),
                row.taxes.to_string(),
                row.net_amount_received.to_string(),
            ],
            ReceiptItem::DomesticStock(row) => vec![
                row.trade_date.clone(),
                row.security_code.clone(),
                row.security_name.clone(),
                row.account.clone(),
                row.shares.to_string(),
                row.asked_price.to_string(),
                row.proceeds.to_string(),
                row.purchase_price.to_string(),
                row.realized_profit_and_loss.to_string(),
                row.taxes.to_string(),
                row.realized_profit_and_loss_after_tax.to_string(),
            ],
            ReceiptItem::MutualFund(row) => vec![
                row.trade_date.clone(),
                row.fund_name.clone(),
                row.account.clone(),
                row.shares.to_string(),
                row.exchange_rate.to_string(),
                row.cancellation_unit_price_yen.to_string(),
                row.cancellation_amount_yen.to_string(),
                row.average_acquisition_price_yen.to_string(),
                row.realized_profit_and_loss.to_string(),
                row.taxes.to_string(),
                row.realized_profit_and_loss_after_tax.to_string(),
            ],
        };
        fields.join("\u{1f}")
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ReceiptSummary {
    Dividend(crate::dto::DividendSummary),
    DomesticStock(crate::dto::DomesticStockSummary),
    MutualFund(crate::dto::MutualfundSummary),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ReceiptTabData {
    pub rows: Vec<ReceiptItem>,
    pub summary: Option<ReceiptSummary>,
    pub truncated: bool,
}

pub fn select_header_summary<T: Clone>(
    api_summary: Option<&T>,
    has_preview: bool,
    search_query: &str,
    client_summary: T,
) -> T {
    if let Some(summary) = api_summary.filter(|_| !has_preview && search_query.is_empty()) {
        summary.clone()
    } else {
        client_summary
    }
}

pub fn truncated_list_warning() -> String {
    format!(
        "一覧は最大{}件まで表示しています。検索条件を絞り込んでください。",
        format_number(
            Decimal::from(store::RECEIPT_LIST_PER_PAGE * store::RECEIPT_LIST_MAX_PAGES),
            0
        )
    )
}

#[derive(Clone, Debug, PartialEq)]
pub enum TabState {
    Loading,
    Ready(ReceiptTabData),
    Failed(String),
}

#[cfg(test)]
mod csv_tests;
#[cfg(test)]
mod fetch_tests;
#[cfg(test)]
mod search_tests;
#[cfg(test)]
mod tests;
