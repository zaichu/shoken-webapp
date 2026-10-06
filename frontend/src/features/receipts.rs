mod csv;
mod dividend_info;
mod filter;
mod kind;
mod model;
mod store;
mod view;

pub use view::ReceiptsPage;

use rust_decimal::Decimal;
use shared::format::format_number;

pub use kind::ReceiptRow;
pub use store::{ReceiptsStore, use_receipts_data};

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
}

#[derive(Clone, Debug, PartialEq)]
pub enum ReceiptItem {
    Dividend(crate::api::dto::Dividend),
    DomesticStock(crate::api::dto::DomesticStock),
    MutualFund(crate::api::dto::Mutualfund),
}

/// テーブル1セルの内容。銘柄コードは `/search` へのリンク、
/// 銘柄名・ファンド名はコピーボタンを出すため、表示文字列とは別に種別を持つ。
#[derive(Clone, Debug, PartialEq)]
pub enum ReceiptCell {
    Text(String),
    SecurityCode(String),
    InstrumentName { name: String, code: Option<String> },
}

impl ReceiptCell {
    pub fn text(&self) -> &str {
        match self {
            Self::Text(value) | Self::SecurityCode(value) => value,
            Self::InstrumentName { name, .. } => name,
        }
    }
}

impl ReceiptItem {
    /// 保存済み行の DB id。行単位の DOM キーに使う。
    pub fn id(&self) -> &str {
        match self {
            ReceiptItem::Dividend(row) => row.id.as_str(),
            ReceiptItem::DomesticStock(row) => row.id.as_str(),
            ReceiptItem::MutualFund(row) => row.id.as_str(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ReceiptSummary {
    Dividend(crate::api::dto::DividendSummary),
    DomesticStock(crate::api::dto::DomesticStockSummary),
    MutualFund(crate::api::dto::MutualfundSummary),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ReceiptTabData {
    pub rows: Vec<ReceiptRow>,
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
            Decimal::from(
                crate::support::pagination::LIST_PER_PAGE
                    * crate::support::pagination::LIST_MAX_PAGES
            ),
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
#[path = "receipts/tests/suite.rs"]
mod tests;
