use std::collections::HashMap;

use rust_decimal::Decimal;
use serde::de::DeserializeOwned;
use shared::normalize::normalize_display_name;
use shared::summary::{domestic_daily, domestic_total, DomesticDailyRow};
use shared::tax::SPECIFIC_ACCOUNT_KEYWORD;

use crate::api::dto::{
    Dividend, DividendSummary, DomesticStock, DomesticStockSummary, Mutualfund, MutualfundSummary,
};
use crate::api::ApiError;
use crate::features::receipts::csv::{
    CsvPreviewRow, DividendCsvRow, DomesticStockCsvRow, MutualfundCsvRow,
};
use crate::features::receipts::model::{
    create_year_month_key, format_currency, format_date, format_number,
};
use crate::features::receipts::{
    ReceiptCell, ReceiptItem, ReceiptSummary, ReceiptTabData, ReceiptsTab,
};
use crate::support::list_search::group_key::{create_group_key_fn, GroupKeyRule};
use crate::support::list_search::support::{group_and_summarize, ColumnReorderRule};
use crate::support::list_search::FilterConfig;
use crate::support::pagination::{fetch_all_pages, ListEndpoint};
use crate::support::row::Row;

/// 取引明細の一覧行。保存済み(DB の行)と CSV プレビュー行を分けて持ち、
/// プレビュー行に永続化済みの偽 id は割り当てない。
pub type ReceiptRow = Row<ReceiptItem, CsvPreviewRow>;

impl Row<ReceiptItem, CsvPreviewRow> {
    /// 保存済み行だけが持つ DB id。プレビュー行は None。
    pub(crate) fn saved_id(&self) -> Option<&str> {
        self.saved().map(ReceiptItem::id)
    }

    pub(crate) fn date(&self) -> &str {
        ReceiptRowData::date(self)
    }
    pub(crate) fn code(&self) -> &str {
        ReceiptRowData::code(self)
    }
    pub(crate) fn name(&self) -> &str {
        ReceiptRowData::name(self)
    }
    pub(crate) fn account(&self) -> &str {
        ReceiptRowData::account(self)
    }
    pub(crate) fn product(&self) -> &str {
        ReceiptRowData::product(self)
    }
    pub(crate) fn is_specific(&self) -> bool {
        ReceiptRowData::is_specific(self)
    }
    pub(crate) fn cells(&self) -> Vec<ReceiptCell> {
        ReceiptRowData::cells(self)
    }
    pub(crate) fn raw_key(&self) -> String {
        ReceiptRowData::raw_key(self)
    }
    pub(crate) fn search_amount(&self, index: usize) -> Decimal {
        ReceiptRowData::search_amount(self, index)
    }
    pub(crate) fn summary_amounts(&self) -> (Decimal, Decimal, Decimal) {
        ReceiptRowData::summary_amounts(self)
    }
    pub(crate) fn realized_pnl(&self) -> Decimal {
        ReceiptRowData::realized_pnl(self)
    }
}

/// 明細行(保存済み・CSV プレビュー)に共通の表示・検索・集計アクセサ。
/// タブ種別ごとの差は `ReceiptKind` 側で扱うため、ここでは全タブ共通の形だけを提供する。
pub(crate) trait ReceiptRowData {
    fn date(&self) -> &str;
    fn code(&self) -> &str;
    fn name(&self) -> &str;
    fn account(&self) -> &str;
    fn product(&self) -> &str {
        ""
    }
    fn is_specific(&self) -> bool {
        self.account().contains(SPECIFIC_ACCOUNT_KEYWORD)
    }
    fn cells(&self) -> Vec<ReceiptCell>;
    // カードの開閉状態を引き継ぐ照合は表示丸め前の値で行う。
    // 数量 1.001 と 1.002 はともに「1.00」と出るが別行として区別する。
    fn raw_key(&self) -> String;
    /// 検索用の金額フィールド。`ReceiptKind::SEARCH_AMOUNTS` の列番号に対応する。
    fn search_amount(&self, index: usize) -> Decimal;
    /// グループ・ヘッダー集計用の金額 3 つ組。(対象金額, 税額, 税引後)
    fn summary_amounts(&self) -> (Decimal, Decimal, Decimal);
    /// 国内株式の日次集計で特定口座・NISA 等に分けて使う実現損益。
    fn realized_pnl(&self) -> Decimal {
        Decimal::ZERO
    }
}

impl ReceiptRowData for Dividend {
    fn date(&self) -> &str {
        &self.settlement_date
    }
    fn code(&self) -> &str {
        &self.security_code
    }
    fn name(&self) -> &str {
        &self.security_name
    }
    fn account(&self) -> &str {
        self.account.as_str()
    }
    fn product(&self) -> &str {
        &self.product
    }
    fn is_specific(&self) -> bool {
        self.account.is_specific()
    }
    fn cells(&self) -> Vec<ReceiptCell> {
        vec![
            ReceiptCell::Text(format_date(&self.settlement_date)),
            ReceiptCell::Text(self.product.clone()),
            ReceiptCell::Text(self.account.to_string()),
            ReceiptCell::SecurityCode(self.security_code.clone()),
            ReceiptCell::InstrumentName {
                name: normalize_display_name(&self.security_name),
                code: Some(self.security_code.clone()),
            },
            ReceiptCell::Text(format_currency(self.unit_price)),
            ReceiptCell::Text(format_number(self.shares, 2)),
            ReceiptCell::Text(format_currency(self.dividends_before_tax)),
            ReceiptCell::Text(format_currency(self.taxes)),
            ReceiptCell::Text(format_currency(self.net_amount_received)),
        ]
    }
    fn raw_key(&self) -> String {
        [
            self.settlement_date.clone(),
            self.product.clone(),
            self.account.to_string(),
            self.security_code.clone(),
            self.security_name.clone(),
            self.unit_price.to_string(),
            self.shares.to_string(),
            self.dividends_before_tax.to_string(),
            self.taxes.to_string(),
            self.net_amount_received.to_string(),
        ]
        .join("\u{1f}")
    }
    fn search_amount(&self, index: usize) -> Decimal {
        match index {
            0 => self.unit_price,
            1 => self.shares,
            2 => self.dividends_before_tax,
            3 => self.taxes,
            _ => self.net_amount_received,
        }
    }
    fn summary_amounts(&self) -> (Decimal, Decimal, Decimal) {
        (
            self.dividends_before_tax,
            self.taxes,
            self.net_amount_received,
        )
    }
}

impl ReceiptRowData for DomesticStock {
    fn date(&self) -> &str {
        &self.trade_date
    }
    fn code(&self) -> &str {
        self.security_code.as_str()
    }
    fn name(&self) -> &str {
        &self.security_name
    }
    fn account(&self) -> &str {
        self.account.as_str()
    }
    fn is_specific(&self) -> bool {
        self.account.is_specific()
    }
    fn cells(&self) -> Vec<ReceiptCell> {
        vec![
            ReceiptCell::Text(format_date(&self.trade_date)),
            ReceiptCell::SecurityCode(self.security_code.to_string()),
            ReceiptCell::InstrumentName {
                name: normalize_display_name(&self.security_name),
                code: Some(self.security_code.to_string()),
            },
            ReceiptCell::Text(self.account.to_string()),
            ReceiptCell::Text(format_number(self.shares, 2)),
            ReceiptCell::Text(format_currency(self.asked_price)),
            ReceiptCell::Text(format_currency(self.proceeds)),
            ReceiptCell::Text(format_currency(self.purchase_price)),
            ReceiptCell::Text(format_currency(self.realized_profit_and_loss)),
            ReceiptCell::Text(format_currency(self.taxes)),
            ReceiptCell::Text(format_currency(self.realized_profit_and_loss_after_tax)),
        ]
    }
    fn raw_key(&self) -> String {
        [
            self.trade_date.clone(),
            self.security_code.to_string(),
            self.security_name.clone(),
            self.account.to_string(),
            self.shares.to_string(),
            self.asked_price.to_string(),
            self.proceeds.to_string(),
            self.purchase_price.to_string(),
            self.realized_profit_and_loss.to_string(),
            self.taxes.to_string(),
            self.realized_profit_and_loss_after_tax.to_string(),
        ]
        .join("\u{1f}")
    }
    fn search_amount(&self, index: usize) -> Decimal {
        match index {
            0 => self.shares,
            1 => self.asked_price,
            2 => self.proceeds,
            3 => self.purchase_price,
            _ => self.realized_profit_and_loss,
        }
    }
    fn summary_amounts(&self) -> (Decimal, Decimal, Decimal) {
        (
            self.realized_profit_and_loss,
            self.taxes,
            self.realized_profit_and_loss_after_tax,
        )
    }
    fn realized_pnl(&self) -> Decimal {
        self.realized_profit_and_loss
    }
}

impl ReceiptRowData for Mutualfund {
    fn date(&self) -> &str {
        &self.trade_date
    }
    fn code(&self) -> &str {
        ""
    }
    fn name(&self) -> &str {
        &self.fund_name
    }
    fn account(&self) -> &str {
        self.account.as_str()
    }
    fn is_specific(&self) -> bool {
        self.account.is_specific()
    }
    fn cells(&self) -> Vec<ReceiptCell> {
        vec![
            ReceiptCell::Text(format_date(&self.trade_date)),
            ReceiptCell::InstrumentName {
                name: normalize_display_name(&self.fund_name),
                code: None,
            },
            ReceiptCell::Text(self.account.to_string()),
            ReceiptCell::Text(format_number(self.shares, 2)),
            ReceiptCell::Text(format_currency(self.cancellation_unit_price_yen)),
            ReceiptCell::Text(format_currency(self.cancellation_amount_yen)),
            ReceiptCell::Text(format_currency(self.average_acquisition_price_yen)),
            ReceiptCell::Text(format_currency(self.realized_profit_and_loss)),
            ReceiptCell::Text(format_currency(self.taxes)),
            ReceiptCell::Text(format_currency(self.realized_profit_and_loss_after_tax)),
        ]
    }
    fn raw_key(&self) -> String {
        [
            self.trade_date.clone(),
            self.fund_name.clone(),
            self.account.to_string(),
            self.shares.to_string(),
            self.exchange_rate.to_string(),
            self.cancellation_unit_price_yen.to_string(),
            self.cancellation_amount_yen.to_string(),
            self.average_acquisition_price_yen.to_string(),
            self.realized_profit_and_loss.to_string(),
            self.taxes.to_string(),
            self.realized_profit_and_loss_after_tax.to_string(),
        ]
        .join("\u{1f}")
    }
    fn search_amount(&self, _index: usize) -> Decimal {
        Decimal::ZERO
    }
    fn summary_amounts(&self) -> (Decimal, Decimal, Decimal) {
        (
            self.realized_profit_and_loss,
            self.taxes,
            self.realized_profit_and_loss_after_tax,
        )
    }
}

// プレビュー行は backend の検証を通った Create*Request 相当のため、
// 口座・銘柄コードは文字列のまま保持する(保存済み行と表示・検索の規則は同じ)。
impl ReceiptRowData for DividendCsvRow {
    fn date(&self) -> &str {
        &self.settlement_date
    }
    fn code(&self) -> &str {
        &self.security_code
    }
    fn name(&self) -> &str {
        &self.security_name
    }
    fn account(&self) -> &str {
        &self.account
    }
    fn product(&self) -> &str {
        &self.product
    }
    fn cells(&self) -> Vec<ReceiptCell> {
        vec![
            ReceiptCell::Text(format_date(&self.settlement_date)),
            ReceiptCell::Text(self.product.clone()),
            ReceiptCell::Text(self.account.clone()),
            ReceiptCell::SecurityCode(self.security_code.clone()),
            ReceiptCell::InstrumentName {
                name: normalize_display_name(&self.security_name),
                code: Some(self.security_code.clone()),
            },
            ReceiptCell::Text(format_currency(self.unit_price)),
            ReceiptCell::Text(format_number(self.shares, 2)),
            ReceiptCell::Text(format_currency(self.dividends_before_tax)),
            ReceiptCell::Text(format_currency(self.taxes)),
            ReceiptCell::Text(format_currency(self.net_amount_received)),
        ]
    }
    fn raw_key(&self) -> String {
        [
            self.settlement_date.clone(),
            self.product.clone(),
            self.account.clone(),
            self.security_code.clone(),
            self.security_name.clone(),
            self.unit_price.to_string(),
            self.shares.to_string(),
            self.dividends_before_tax.to_string(),
            self.taxes.to_string(),
            self.net_amount_received.to_string(),
        ]
        .join("\u{1f}")
    }
    fn search_amount(&self, index: usize) -> Decimal {
        match index {
            0 => self.unit_price,
            1 => self.shares,
            2 => self.dividends_before_tax,
            3 => self.taxes,
            _ => self.net_amount_received,
        }
    }
    fn summary_amounts(&self) -> (Decimal, Decimal, Decimal) {
        (
            self.dividends_before_tax,
            self.taxes,
            self.net_amount_received,
        )
    }
}

impl ReceiptRowData for DomesticStockCsvRow {
    fn date(&self) -> &str {
        &self.trade_date
    }
    fn code(&self) -> &str {
        &self.security_code
    }
    fn name(&self) -> &str {
        &self.security_name
    }
    fn account(&self) -> &str {
        &self.account
    }
    fn cells(&self) -> Vec<ReceiptCell> {
        vec![
            ReceiptCell::Text(format_date(&self.trade_date)),
            ReceiptCell::SecurityCode(self.security_code.clone()),
            ReceiptCell::InstrumentName {
                name: normalize_display_name(&self.security_name),
                code: Some(self.security_code.clone()),
            },
            ReceiptCell::Text(self.account.clone()),
            ReceiptCell::Text(format_number(self.shares, 2)),
            ReceiptCell::Text(format_currency(self.asked_price)),
            ReceiptCell::Text(format_currency(self.proceeds)),
            ReceiptCell::Text(format_currency(self.purchase_price)),
            ReceiptCell::Text(format_currency(self.realized_profit_and_loss)),
            ReceiptCell::Text(format_currency(self.taxes)),
            ReceiptCell::Text(format_currency(self.realized_profit_and_loss_after_tax)),
        ]
    }
    fn raw_key(&self) -> String {
        [
            self.trade_date.clone(),
            self.security_code.clone(),
            self.security_name.clone(),
            self.account.clone(),
            self.shares.to_string(),
            self.asked_price.to_string(),
            self.proceeds.to_string(),
            self.purchase_price.to_string(),
            self.realized_profit_and_loss.to_string(),
            self.taxes.to_string(),
            self.realized_profit_and_loss_after_tax.to_string(),
        ]
        .join("\u{1f}")
    }
    fn search_amount(&self, index: usize) -> Decimal {
        match index {
            0 => self.shares,
            1 => self.asked_price,
            2 => self.proceeds,
            3 => self.purchase_price,
            _ => self.realized_profit_and_loss,
        }
    }
    fn summary_amounts(&self) -> (Decimal, Decimal, Decimal) {
        (
            self.realized_profit_and_loss,
            self.taxes,
            self.realized_profit_and_loss_after_tax,
        )
    }
    fn realized_pnl(&self) -> Decimal {
        self.realized_profit_and_loss
    }
}

impl ReceiptRowData for MutualfundCsvRow {
    fn date(&self) -> &str {
        &self.trade_date
    }
    fn code(&self) -> &str {
        ""
    }
    fn name(&self) -> &str {
        &self.fund_name
    }
    fn account(&self) -> &str {
        &self.account
    }
    fn cells(&self) -> Vec<ReceiptCell> {
        vec![
            ReceiptCell::Text(format_date(&self.trade_date)),
            ReceiptCell::InstrumentName {
                name: normalize_display_name(&self.fund_name),
                code: None,
            },
            ReceiptCell::Text(self.account.clone()),
            ReceiptCell::Text(format_number(self.shares, 2)),
            ReceiptCell::Text(format_currency(self.cancellation_unit_price_yen)),
            ReceiptCell::Text(format_currency(self.cancellation_amount_yen)),
            ReceiptCell::Text(format_currency(self.average_acquisition_price_yen)),
            ReceiptCell::Text(format_currency(self.realized_profit_and_loss)),
            ReceiptCell::Text(format_currency(self.taxes)),
            ReceiptCell::Text(format_currency(self.realized_profit_and_loss_after_tax)),
        ]
    }
    fn raw_key(&self) -> String {
        [
            self.trade_date.clone(),
            self.fund_name.clone(),
            self.account.clone(),
            self.shares.to_string(),
            self.exchange_rate.to_string(),
            self.cancellation_unit_price_yen.to_string(),
            self.cancellation_amount_yen.to_string(),
            self.average_acquisition_price_yen.to_string(),
            self.realized_profit_and_loss.to_string(),
            self.taxes.to_string(),
            self.realized_profit_and_loss_after_tax.to_string(),
        ]
        .join("\u{1f}")
    }
    fn search_amount(&self, _index: usize) -> Decimal {
        Decimal::ZERO
    }
    fn summary_amounts(&self) -> (Decimal, Decimal, Decimal) {
        (
            self.realized_profit_and_loss,
            self.taxes,
            self.realized_profit_and_loss_after_tax,
        )
    }
}

impl ReceiptRowData for CsvPreviewRow {
    fn date(&self) -> &str {
        match self {
            Self::Dividend(row) => row.date(),
            Self::DomesticStock(row) => row.date(),
            Self::MutualFund(row) => row.date(),
        }
    }
    fn code(&self) -> &str {
        match self {
            Self::Dividend(row) => row.code(),
            Self::DomesticStock(row) => row.code(),
            Self::MutualFund(row) => row.code(),
        }
    }
    fn name(&self) -> &str {
        match self {
            Self::Dividend(row) => row.name(),
            Self::DomesticStock(row) => row.name(),
            Self::MutualFund(row) => row.name(),
        }
    }
    fn account(&self) -> &str {
        match self {
            Self::Dividend(row) => row.account(),
            Self::DomesticStock(row) => row.account(),
            Self::MutualFund(row) => row.account(),
        }
    }
    fn product(&self) -> &str {
        match self {
            Self::Dividend(row) => row.product(),
            Self::DomesticStock(row) => row.product(),
            Self::MutualFund(row) => row.product(),
        }
    }
    fn cells(&self) -> Vec<ReceiptCell> {
        match self {
            Self::Dividend(row) => row.cells(),
            Self::DomesticStock(row) => row.cells(),
            Self::MutualFund(row) => row.cells(),
        }
    }
    fn raw_key(&self) -> String {
        match self {
            Self::Dividend(row) => row.raw_key(),
            Self::DomesticStock(row) => row.raw_key(),
            Self::MutualFund(row) => row.raw_key(),
        }
    }
    fn search_amount(&self, index: usize) -> Decimal {
        match self {
            Self::Dividend(row) => row.search_amount(index),
            Self::DomesticStock(row) => row.search_amount(index),
            Self::MutualFund(row) => row.search_amount(index),
        }
    }
    fn summary_amounts(&self) -> (Decimal, Decimal, Decimal) {
        match self {
            Self::Dividend(row) => row.summary_amounts(),
            Self::DomesticStock(row) => row.summary_amounts(),
            Self::MutualFund(row) => row.summary_amounts(),
        }
    }
    fn realized_pnl(&self) -> Decimal {
        match self {
            Self::Dividend(row) => row.realized_pnl(),
            Self::DomesticStock(row) => row.realized_pnl(),
            Self::MutualFund(row) => row.realized_pnl(),
        }
    }
}

impl ReceiptRowData for ReceiptItem {
    fn date(&self) -> &str {
        match self {
            Self::Dividend(row) => row.date(),
            Self::DomesticStock(row) => row.date(),
            Self::MutualFund(row) => row.date(),
        }
    }
    fn code(&self) -> &str {
        match self {
            Self::Dividend(row) => row.code(),
            Self::DomesticStock(row) => row.code(),
            Self::MutualFund(row) => row.code(),
        }
    }
    fn name(&self) -> &str {
        match self {
            Self::Dividend(row) => row.name(),
            Self::DomesticStock(row) => row.name(),
            Self::MutualFund(row) => row.name(),
        }
    }
    fn account(&self) -> &str {
        match self {
            Self::Dividend(row) => row.account(),
            Self::DomesticStock(row) => row.account(),
            Self::MutualFund(row) => row.account(),
        }
    }
    fn product(&self) -> &str {
        match self {
            Self::Dividend(row) => row.product(),
            Self::DomesticStock(row) => row.product(),
            Self::MutualFund(row) => row.product(),
        }
    }
    fn is_specific(&self) -> bool {
        match self {
            Self::Dividend(row) => row.is_specific(),
            Self::DomesticStock(row) => row.is_specific(),
            Self::MutualFund(row) => row.is_specific(),
        }
    }
    fn cells(&self) -> Vec<ReceiptCell> {
        match self {
            Self::Dividend(row) => row.cells(),
            Self::DomesticStock(row) => row.cells(),
            Self::MutualFund(row) => row.cells(),
        }
    }
    fn raw_key(&self) -> String {
        match self {
            Self::Dividend(row) => row.raw_key(),
            Self::DomesticStock(row) => row.raw_key(),
            Self::MutualFund(row) => row.raw_key(),
        }
    }
    fn search_amount(&self, index: usize) -> Decimal {
        match self {
            Self::Dividend(row) => row.search_amount(index),
            Self::DomesticStock(row) => row.search_amount(index),
            Self::MutualFund(row) => row.search_amount(index),
        }
    }
    fn summary_amounts(&self) -> (Decimal, Decimal, Decimal) {
        match self {
            Self::Dividend(row) => row.summary_amounts(),
            Self::DomesticStock(row) => row.summary_amounts(),
            Self::MutualFund(row) => row.summary_amounts(),
        }
    }
    fn realized_pnl(&self) -> Decimal {
        match self {
            Self::Dividend(row) => row.realized_pnl(),
            Self::DomesticStock(row) => row.realized_pnl(),
            Self::MutualFund(row) => row.realized_pnl(),
        }
    }
}

impl<S: ReceiptRowData, P: ReceiptRowData> ReceiptRowData for Row<S, P> {
    fn date(&self) -> &str {
        match self {
            Self::Saved(row) => row.date(),
            Self::Preview(row) => row.date(),
        }
    }
    fn code(&self) -> &str {
        match self {
            Self::Saved(row) => row.code(),
            Self::Preview(row) => row.code(),
        }
    }
    fn name(&self) -> &str {
        match self {
            Self::Saved(row) => row.name(),
            Self::Preview(row) => row.name(),
        }
    }
    fn account(&self) -> &str {
        match self {
            Self::Saved(row) => row.account(),
            Self::Preview(row) => row.account(),
        }
    }
    fn product(&self) -> &str {
        match self {
            Self::Saved(row) => row.product(),
            Self::Preview(row) => row.product(),
        }
    }
    fn is_specific(&self) -> bool {
        match self {
            Self::Saved(row) => row.is_specific(),
            Self::Preview(row) => row.is_specific(),
        }
    }
    fn cells(&self) -> Vec<ReceiptCell> {
        match self {
            Self::Saved(row) => row.cells(),
            Self::Preview(row) => row.cells(),
        }
    }
    fn raw_key(&self) -> String {
        match self {
            Self::Saved(row) => row.raw_key(),
            Self::Preview(row) => row.raw_key(),
        }
    }
    fn search_amount(&self, index: usize) -> Decimal {
        match self {
            Self::Saved(row) => row.search_amount(index),
            Self::Preview(row) => row.search_amount(index),
        }
    }
    fn summary_amounts(&self) -> (Decimal, Decimal, Decimal) {
        match self {
            Self::Saved(row) => row.summary_amounts(),
            Self::Preview(row) => row.summary_amounts(),
        }
    }
    fn realized_pnl(&self) -> Decimal {
        match self {
            Self::Saved(row) => row.realized_pnl(),
            Self::Preview(row) => row.realized_pnl(),
        }
    }
}

/// lg 以上はレールが横に並んで表の幅が狭くなるため、隠す列の境目は xl と 2xl に置く
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub(crate) enum ColumnTier {
    Core,
    Wide,
    Wider,
}

impl ColumnTier {
    pub(crate) fn class(self) -> &'static str {
        match self {
            ColumnTier::Core => "",
            ColumnTier::Wide => " hidden xl:table-cell print:table-cell",
            ColumnTier::Wider => " hidden 2xl:table-cell print:table-cell",
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct CardFields {
    pub(crate) name: usize,
    pub(crate) date: usize,
    pub(crate) account: usize,
}

/// 表の1グループ。保存行は id、プレビュー行は None の行識別子を持つ。
pub(crate) struct TableGroup {
    pub(crate) key: String,
    pub(crate) label: String,
    pub(crate) summary: Vec<String>,
    pub(crate) rows: Vec<(Option<String>, String, Vec<ReceiptCell>)>,
}

// 検索語が銘柄名・口座などに一致したグループはキーが日付形にならない
pub(crate) fn is_date_group_key(key: &str) -> bool {
    (key.len() == 7 || key.len() == 10)
        && key.bytes().enumerate().all(|(i, b)| {
            if i == 4 || i == 7 {
                b == b'-'
            } else {
                b.is_ascii_digit()
            }
        })
}

pub(crate) fn group_label(key: &str) -> String {
    if !is_date_group_key(key) {
        return normalize_display_name(key);
    }
    let parts: Vec<_> = key.split('-').collect();
    match parts.as_slice() {
        [year, month, day] => format!(
            "{year}年{}月{}日",
            month.parse::<u32>().unwrap_or(0),
            day.parse::<u32>().unwrap_or(0)
        ),
        [year, month] => format!("{year}年{}月", month.parse::<u32>().unwrap_or(0)),
        _ => key.to_string(),
    }
}

fn product_matches(row: &ReceiptRow, query: &str) -> bool {
    normalize_display_name(row.product())
        .to_lowercase()
        .contains(query)
}

fn account_matches(row: &ReceiptRow, query: &str) -> bool {
    normalize_display_name(row.account())
        .to_lowercase()
        .contains(query)
}

// `search_amount` の列番号から fn ポインタの配列を作る
macro_rules! amount_getters {
    ($($index:literal),+ $(,)?) => {
        &[$(|row: &ReceiptRow| row.search_amount($index)),+]
    };
}

/// タブ種別ごとの違いを 1 か所に集約する。静的ディスパッチのため実体は持たない。
pub(crate) trait ReceiptKind: ListEndpoint {
    type CsvRow: Default + Clone + std::fmt::Debug + PartialEq + DeserializeOwned + ReceiptRowData;

    const LABEL: &'static str;
    const PREVIEW_PATH: &'static str;
    const IMPORT_PATH: &'static str;
    const HEADERS: &'static [&'static str];
    const COLUMN_WIDTHS: &'static [&'static str];
    const COLUMN_TIERS: &'static [ColumnTier];
    const COLUMN_ALIGNS: &'static [&'static str];
    const CARD_FIELDS: CardFields;
    const SUMMARY_LABELS: [&'static str; 3];
    // ヘッダー集計の項目。(ラベル, 損益系なら負数で赤文字にするか)
    const HEADER_ITEMS: [(&'static str, bool); 3];
    const EMPTY_HINT: &'static str;
    const CSV_INPUT_ID: &'static str;
    const STRING_FIELDS: &'static [fn(&ReceiptRow) -> &str];
    const SEARCH_AMOUNTS: &'static [fn(&ReceiptRow) -> Decimal] = &[];
    const PRODUCT_CATEGORY: bool = false;
    const ACCOUNT_CATEGORY: bool = true;
    const REORDER_RULES: &'static [ColumnReorderRule<ReceiptRow>] = &[];
    const REORDER_FIXED: usize = 0;

    fn wrap_row(row: Self::Row) -> ReceiptItem;
    fn wrap_summary(summary: Self::Summary) -> ReceiptSummary;
    fn wrap_csv_row(row: Self::CsvRow) -> CsvPreviewRow;
    fn api_summary(summary: &ReceiptSummary) -> Option<[Decimal; 3]>;
    /// 検索・プレビュー中に代わりに使う、画面側での行合計。
    fn totals(rows: &[ReceiptRow]) -> Self::Summary;
    fn summary_triple(summary: &Self::Summary) -> [Decimal; 3];
    fn table_groups(rows: &[ReceiptRow], all_rows: &[ReceiptRow], query: &str) -> Vec<TableGroup>;

    fn filter_config() -> FilterConfig<ReceiptRow> {
        FilterConfig {
            string_fields: Some(Self::STRING_FIELDS.to_vec()),
            partial_string_fields: None,
            date_field: Some(ReceiptRow::date),
            year_search: true,
            year_month_search: true,
            date_search: true,
            date_range_search: true,
            amount_fields: (!Self::SEARCH_AMOUNTS.is_empty())
                .then(|| Self::SEARCH_AMOUNTS.to_vec()),
        }
    }
}

fn sorted_rows(rows: &[ReceiptRow]) -> Vec<ReceiptRow> {
    let mut sorted = rows.to_vec();
    sorted.sort_by(|a, b| b.date().cmp(a.date()));
    sorted
}

fn row_tuple(row: &ReceiptRow) -> (Option<String>, String, Vec<ReceiptCell>) {
    (
        row.saved_id().map(str::to_string),
        row.raw_key(),
        row.cells(),
    )
}

fn summarize_groups(sorted: &[ReceiptRow], key: impl Fn(&ReceiptRow) -> String) -> Vec<TableGroup> {
    group_and_summarize(
        sorted,
        &key,
        &[
            |row: &ReceiptRow| row.summary_amounts().0,
            |row: &ReceiptRow| row.summary_amounts().1,
            |row: &ReceiptRow| row.summary_amounts().2,
        ],
        true,
    )
    .into_iter()
    .map(|summary| {
        let group_rows = sorted
            .iter()
            .filter(|row| key(row) == summary.filter)
            .map(row_tuple)
            .collect();
        TableGroup {
            key: summary.filter.clone(),
            label: group_label(&summary.filter),
            summary: summary
                .values
                .iter()
                .map(|value| format_currency(*value))
                .collect(),
            rows: group_rows,
        }
    })
    .collect()
}

fn totals_from_amounts<S>(
    rows: &[ReceiptRow],
    build: impl Fn((Decimal, Decimal, Decimal)) -> S,
) -> S {
    let mut sum = (Decimal::ZERO, Decimal::ZERO, Decimal::ZERO);
    for row in rows {
        let (a, b, c) = row.summary_amounts();
        sum = (sum.0 + a, sum.1 + b, sum.2 + c);
    }
    build(sum)
}

/// 保存済み行・CSV プレビュー行どちらも `shared::summary` の日次集計にそのまま渡せるようにする。
impl DomesticDailyRow for Row<ReceiptItem, CsvPreviewRow> {
    fn daily_key(&self) -> String {
        self.date().to_string()
    }

    fn is_specific_account(&self) -> bool {
        self.is_specific()
    }

    fn realized_profit_and_loss(&self) -> Decimal {
        self.realized_pnl()
    }
}

pub(crate) struct DividendKind;
pub(crate) struct DomesticStockKind;
pub(crate) struct MutualFundKind;

impl ListEndpoint for DividendKind {
    type Row = Dividend;
    type Summary = DividendSummary;
    const PATH: &'static str = "/api/v1/dividends";
}

impl ReceiptKind for DividendKind {
    type CsvRow = DividendCsvRow;

    const LABEL: &'static str = "配当金";
    const PREVIEW_PATH: &'static str = "/api/v1/dividend-import-validations";
    const IMPORT_PATH: &'static str = "/api/v1/dividend-imports";
    const HEADERS: &'static [&'static str] = &[
        "入金日",
        "商品",
        "口座",
        "銘柄コード",
        "銘柄名",
        "単価",
        "数量",
        "配当金",
        "税額",
        "税引後",
    ];
    const COLUMN_WIDTHS: &'static [&'static str] = &[
        "96px", "76px", "76px", "88px", "", "80px", "72px", "104px", "88px", "104px",
    ];
    const COLUMN_TIERS: &'static [ColumnTier] = &[
        ColumnTier::Core,
        ColumnTier::Wider,
        ColumnTier::Wide,
        ColumnTier::Core,
        ColumnTier::Core,
        ColumnTier::Wide,
        ColumnTier::Wide,
        ColumnTier::Core,
        ColumnTier::Core,
        ColumnTier::Core,
    ];
    const COLUMN_ALIGNS: &'static [&'static str] = &[
        "left", "left", "left", "center", "left", "right", "right", "right", "right", "right",
    ];
    const CARD_FIELDS: CardFields = CardFields {
        name: 4,
        date: 0,
        account: 2,
    };
    const SUMMARY_LABELS: [&'static str; 3] = ["配当金", "税額", "税引後"];
    const HEADER_ITEMS: [(&'static str, bool); 3] =
        [("配当金", false), ("税額", false), ("税引後", false)];
    const EMPTY_HINT: &'static str = "配当金明細をCSVで追加してください";
    const CSV_INPUT_ID: &'static str = "csv-file-input-dividend";
    const STRING_FIELDS: &'static [fn(&ReceiptRow) -> &str] = &[
        ReceiptRow::code,
        ReceiptRow::name,
        ReceiptRow::account,
        ReceiptRow::product,
    ];
    const SEARCH_AMOUNTS: &'static [fn(&ReceiptRow) -> Decimal] = amount_getters!(0, 1, 2, 3, 4);
    const PRODUCT_CATEGORY: bool = true;
    const REORDER_RULES: &'static [ColumnReorderRule<ReceiptRow>] = &[
        ColumnReorderRule {
            column_key: 1,
            matches: product_matches,
        },
        ColumnReorderRule {
            column_key: 2,
            matches: account_matches,
        },
    ];
    const REORDER_FIXED: usize = 1;

    fn wrap_row(row: Self::Row) -> ReceiptItem {
        ReceiptItem::Dividend(row)
    }

    fn wrap_summary(summary: Self::Summary) -> ReceiptSummary {
        ReceiptSummary::Dividend(summary)
    }

    fn wrap_csv_row(row: Self::CsvRow) -> CsvPreviewRow {
        CsvPreviewRow::Dividend(row)
    }

    fn api_summary(summary: &ReceiptSummary) -> Option<[Decimal; 3]> {
        match summary {
            ReceiptSummary::Dividend(summary) => Some(Self::summary_triple(summary)),
            _ => None,
        }
    }

    fn totals(rows: &[ReceiptRow]) -> Self::Summary {
        totals_from_amounts(rows, |(before_tax, taxes, net)| DividendSummary {
            total_dividends_before_tax: before_tax,
            total_taxes: taxes,
            total_net_amount_received: net,
        })
    }

    fn summary_triple(summary: &Self::Summary) -> [Decimal; 3] {
        [
            summary.total_dividends_before_tax,
            summary.total_taxes,
            summary.total_net_amount_received,
        ]
    }

    fn table_groups(rows: &[ReceiptRow], all_rows: &[ReceiptRow], query: &str) -> Vec<TableGroup> {
        let sorted = sorted_rows(rows);
        // 同じ銘柄コードでも新しい行の銘柄名を優先する。絞り込み前の全行から引く
        let mut latest: HashMap<&str, (&str, &str)> = HashMap::new();
        for row in all_rows {
            match latest.get_mut(row.code()) {
                Some(entry) if row.date() > entry.0 => {
                    *entry = (row.date(), row.name());
                }
                None => {
                    latest.insert(row.code(), (row.date(), row.name()));
                }
                _ => {}
            }
        }
        let security_key = |row: &ReceiptRow| {
            let name = latest
                .get(row.code())
                .map_or_else(|| row.name().to_string(), |(_, name)| (*name).to_string());
            // 見出しは半角化して出すのでキーも揃え、全角・半角だけ違う名が別コード間で分かれないようにする
            normalize_display_name(&name)
        };
        let rules = [
            GroupKeyRule {
                test: |row: &ReceiptRow, token| {
                    row.code().to_lowercase() == token
                        || normalize_display_name(row.name()).to_lowercase() == token
                },
                key_fn: &security_key,
            },
            GroupKeyRule {
                test: |row: &ReceiptRow, token| {
                    normalize_display_name(row.product()).to_lowercase() == token
                },
                key_fn: &|row: &ReceiptRow| normalize_display_name(row.product()),
            },
            GroupKeyRule {
                test: |row: &ReceiptRow, token| {
                    normalize_display_name(row.account()).to_lowercase() == token
                },
                key_fn: &|row: &ReceiptRow| normalize_display_name(row.account()),
            },
        ];
        let key = create_group_key_fn(query, |row| create_year_month_key(row.date()), &rules);
        summarize_groups(&sorted, key)
    }
}

impl ListEndpoint for DomesticStockKind {
    type Row = DomesticStock;
    type Summary = DomesticStockSummary;
    const PATH: &'static str = "/api/v1/domestic-stock-transactions";
}

impl ReceiptKind for DomesticStockKind {
    type CsvRow = DomesticStockCsvRow;

    const LABEL: &'static str = "国内株式";
    const PREVIEW_PATH: &'static str = "/api/v1/domestic-stock-import-validations";
    const IMPORT_PATH: &'static str = "/api/v1/domestic-stock-imports";
    const HEADERS: &'static [&'static str] = &[
        "約定日",
        "銘柄コード",
        "銘柄名",
        "口座",
        "数量",
        "売却単価",
        "売却額",
        "取得価額",
        "実現損益",
        "税額",
        "税引後",
    ];
    const COLUMN_WIDTHS: &'static [&'static str] = &[
        "96px", "88px", "", "76px", "72px", "80px", "104px", "104px", "104px", "88px", "104px",
    ];
    const COLUMN_TIERS: &'static [ColumnTier] = &[
        ColumnTier::Core,
        ColumnTier::Core,
        ColumnTier::Core,
        ColumnTier::Wider,
        ColumnTier::Wider,
        ColumnTier::Wide,
        ColumnTier::Wide,
        ColumnTier::Wider,
        ColumnTier::Core,
        ColumnTier::Core,
        ColumnTier::Core,
    ];
    const COLUMN_ALIGNS: &'static [&'static str] = &[
        "left", "center", "left", "left", "right", "right", "right", "right", "right", "right",
        "right",
    ];
    const CARD_FIELDS: CardFields = CardFields {
        name: 2,
        date: 0,
        account: 3,
    };
    const SUMMARY_LABELS: [&'static str; 3] = ["実現損益", "税額", "税引後"];
    const HEADER_ITEMS: [(&'static str, bool); 3] =
        [("実現損益", true), ("税額", false), ("税引後", true)];
    const EMPTY_HINT: &'static str = "国内株式明細をCSVで追加してください";
    const CSV_INPUT_ID: &'static str = "csv-file-input-domesticstock";
    const STRING_FIELDS: &'static [fn(&ReceiptRow) -> &str] =
        &[ReceiptRow::code, ReceiptRow::name, ReceiptRow::account];
    const SEARCH_AMOUNTS: &'static [fn(&ReceiptRow) -> Decimal] = amount_getters!(0, 1, 2, 3, 4);
    const REORDER_RULES: &'static [ColumnReorderRule<ReceiptRow>] = &[ColumnReorderRule {
        column_key: 3,
        matches: account_matches,
    }];
    const REORDER_FIXED: usize = 2;

    fn wrap_row(row: Self::Row) -> ReceiptItem {
        ReceiptItem::DomesticStock(row)
    }

    fn wrap_summary(summary: Self::Summary) -> ReceiptSummary {
        ReceiptSummary::DomesticStock(summary)
    }

    fn wrap_csv_row(row: Self::CsvRow) -> CsvPreviewRow {
        CsvPreviewRow::DomesticStock(row)
    }

    fn api_summary(summary: &ReceiptSummary) -> Option<[Decimal; 3]> {
        match summary {
            ReceiptSummary::DomesticStock(summary) => Some(Self::summary_triple(summary)),
            _ => None,
        }
    }

    fn totals(rows: &[ReceiptRow]) -> Self::Summary {
        domestic_total(rows)
    }

    fn summary_triple(summary: &Self::Summary) -> [Decimal; 3] {
        [
            summary.total_realized_profit_and_loss,
            summary.total_taxes,
            summary.total_realized_profit_and_loss_after_tax,
        ]
    }

    fn table_groups(
        rows: &[ReceiptRow],
        _all_rows: &[ReceiptRow],
        _query: &str,
    ) -> Vec<TableGroup> {
        let sorted = sorted_rows(rows);
        domestic_daily(&sorted)
            .into_iter()
            .map(|day| {
                let date = day.filter;
                TableGroup {
                    key: date.clone(),
                    label: group_label(&date),
                    summary: vec![
                        format_currency(day.total_realized_profit_and_loss),
                        format_currency(day.total_taxes),
                        format_currency(day.total_realized_profit_and_loss_after_tax),
                    ],
                    rows: sorted
                        .iter()
                        .filter(|row| row.date() == date)
                        .map(row_tuple)
                        .collect(),
                }
            })
            .collect()
    }
}

impl ListEndpoint for MutualFundKind {
    type Row = Mutualfund;
    type Summary = MutualfundSummary;
    const PATH: &'static str = "/api/v1/mutual-fund-transactions";
}

impl ReceiptKind for MutualFundKind {
    type CsvRow = MutualfundCsvRow;

    const LABEL: &'static str = "投資信託";
    const PREVIEW_PATH: &'static str = "/api/v1/mutual-fund-import-validations";
    const IMPORT_PATH: &'static str = "/api/v1/mutual-fund-imports";
    const HEADERS: &'static [&'static str] = &[
        "約定日",
        "ファンド名",
        "口座",
        "数量",
        "解約単価",
        "解約額",
        "取得価額",
        "実現損益",
        "税額",
        "税引後",
    ];
    const COLUMN_WIDTHS: &'static [&'static str] = &[
        "96px", "", "76px", "72px", "80px", "104px", "104px", "104px", "88px", "104px",
    ];
    const COLUMN_TIERS: &'static [ColumnTier] = &[
        ColumnTier::Core,
        ColumnTier::Core,
        ColumnTier::Wider,
        ColumnTier::Wider,
        ColumnTier::Wide,
        ColumnTier::Core,
        ColumnTier::Wide,
        ColumnTier::Core,
        ColumnTier::Core,
        ColumnTier::Core,
    ];
    const COLUMN_ALIGNS: &'static [&'static str] = &[
        "left", "left", "left", "right", "right", "right", "right", "right", "right", "right",
    ];
    const CARD_FIELDS: CardFields = CardFields {
        name: 1,
        date: 0,
        account: 2,
    };
    const SUMMARY_LABELS: [&'static str; 3] = ["実現損益", "税額", "税引後"];
    const HEADER_ITEMS: [(&'static str, bool); 3] =
        [("実現損益", true), ("税額", false), ("税引後", true)];
    const EMPTY_HINT: &'static str = "投資信託明細をCSVで追加してください";
    const CSV_INPUT_ID: &'static str = "csv-file-input-mutualfund";
    const STRING_FIELDS: &'static [fn(&ReceiptRow) -> &str] =
        &[ReceiptRow::name, ReceiptRow::account];
    const ACCOUNT_CATEGORY: bool = false;

    fn wrap_row(row: Self::Row) -> ReceiptItem {
        ReceiptItem::MutualFund(row)
    }

    fn wrap_summary(summary: Self::Summary) -> ReceiptSummary {
        ReceiptSummary::MutualFund(summary)
    }

    fn wrap_csv_row(row: Self::CsvRow) -> CsvPreviewRow {
        CsvPreviewRow::MutualFund(row)
    }

    fn api_summary(summary: &ReceiptSummary) -> Option<[Decimal; 3]> {
        match summary {
            ReceiptSummary::MutualFund(summary) => Some(Self::summary_triple(summary)),
            _ => None,
        }
    }

    fn totals(rows: &[ReceiptRow]) -> Self::Summary {
        totals_from_amounts(rows, |(pnl, taxes, after_tax)| MutualfundSummary {
            total_realized_profit_and_loss: pnl,
            total_taxes: taxes,
            total_realized_profit_and_loss_after_tax: after_tax,
        })
    }

    fn summary_triple(summary: &Self::Summary) -> [Decimal; 3] {
        [
            summary.total_realized_profit_and_loss,
            summary.total_taxes,
            summary.total_realized_profit_and_loss_after_tax,
        ]
    }

    fn table_groups(rows: &[ReceiptRow], _all_rows: &[ReceiptRow], query: &str) -> Vec<TableGroup> {
        let sorted = sorted_rows(rows);
        let rules = [GroupKeyRule {
            test: |row: &ReceiptRow, token| {
                normalize_display_name(row.name())
                    .to_lowercase()
                    .contains(token)
            },
            // 見出しは半角化して出すのでキーも揃え、全角・半角だけ違う名をまとめる
            key_fn: &|row: &ReceiptRow| normalize_display_name(row.name()),
        }];
        let key = create_group_key_fn(query, |row| create_year_month_key(row.date()), &rules);
        summarize_groups(&sorted, key)
    }
}

macro_rules! kind_const {
    ($self:expr, $name:ident) => {
        match $self {
            ReceiptsTab::Dividend => DividendKind::$name,
            ReceiptsTab::DomesticStock => DomesticStockKind::$name,
            ReceiptsTab::MutualFund => MutualFundKind::$name,
        }
    };
}

macro_rules! dispatch_kind {
    ($self:expr, $method:ident ( $($args:expr),* $(,)? )) => {
        match $self {
            ReceiptsTab::Dividend => DividendKind::$method($($args),*),
            ReceiptsTab::DomesticStock => DomesticStockKind::$method($($args),*),
            ReceiptsTab::MutualFund => MutualFundKind::$method($($args),*),
        }
    };
}

/// `ReceiptsTab` のタブ種別による分岐はすべてここを経由し、`ReceiptKind` の実装に委譲する。
impl ReceiptsTab {
    pub fn label(self) -> &'static str {
        kind_const!(self, LABEL)
    }

    pub(crate) fn list_path(self) -> &'static str {
        kind_const!(self, PATH)
    }

    pub(crate) fn preview_path(self) -> &'static str {
        kind_const!(self, PREVIEW_PATH)
    }

    pub(crate) fn import_path(self) -> &'static str {
        kind_const!(self, IMPORT_PATH)
    }

    pub(crate) fn headers(self) -> &'static [&'static str] {
        kind_const!(self, HEADERS)
    }

    pub(crate) fn column_widths(self) -> &'static [&'static str] {
        kind_const!(self, COLUMN_WIDTHS)
    }

    pub(crate) fn column_tiers(self) -> &'static [ColumnTier] {
        kind_const!(self, COLUMN_TIERS)
    }

    pub(crate) fn column_aligns(self) -> &'static [&'static str] {
        kind_const!(self, COLUMN_ALIGNS)
    }

    pub(crate) fn card_fields(self) -> CardFields {
        kind_const!(self, CARD_FIELDS)
    }

    pub(crate) fn summary_labels(self) -> [&'static str; 3] {
        kind_const!(self, SUMMARY_LABELS)
    }

    pub(crate) fn header_items(self) -> [(&'static str, bool); 3] {
        kind_const!(self, HEADER_ITEMS)
    }

    pub(crate) fn empty_hint(self) -> &'static str {
        kind_const!(self, EMPTY_HINT)
    }

    pub(crate) fn csv_input_id(self) -> &'static str {
        kind_const!(self, CSV_INPUT_ID)
    }

    pub(crate) fn product_category(self) -> bool {
        kind_const!(self, PRODUCT_CATEGORY)
    }

    pub(crate) fn account_category(self) -> bool {
        kind_const!(self, ACCOUNT_CATEGORY)
    }

    pub(crate) fn reorder_rules(self) -> &'static [ColumnReorderRule<ReceiptRow>] {
        kind_const!(self, REORDER_RULES)
    }

    pub(crate) fn reorder_fixed(self) -> usize {
        kind_const!(self, REORDER_FIXED)
    }

    pub(crate) fn filter_config(self) -> FilterConfig<ReceiptRow> {
        dispatch_kind!(self, filter_config())
    }

    pub(crate) fn table_groups(
        self,
        rows: &[ReceiptRow],
        all_rows: &[ReceiptRow],
        query: &str,
    ) -> Vec<TableGroup> {
        dispatch_kind!(self, table_groups(rows, all_rows, query))
    }

    /// API の集計行をヘッダー表示用の 3 値に変換する。タブの組が違う集計は None。
    pub(crate) fn api_summary_triple(self, summary: &ReceiptSummary) -> Option<[Decimal; 3]> {
        dispatch_kind!(self, api_summary(summary))
    }

    /// 検索・プレビュー中に使う画面側合計をヘッダー表示用の 3 値にする。
    pub(crate) fn client_summary_triple(self, rows: &[ReceiptRow]) -> [Decimal; 3] {
        match self {
            ReceiptsTab::Dividend => DividendKind::summary_triple(&DividendKind::totals(rows)),
            ReceiptsTab::DomesticStock => {
                DomesticStockKind::summary_triple(&DomesticStockKind::totals(rows))
            }
            ReceiptsTab::MutualFund => {
                MutualFundKind::summary_triple(&MutualFundKind::totals(rows))
            }
        }
    }

    pub(crate) fn parse_csv_row(self, value: serde_json::Value) -> CsvPreviewRow {
        dispatch_kind!(
            self,
            wrap_csv_row(serde_json::from_value(value).unwrap_or_default())
        )
    }

    pub(crate) async fn fetch_list(self) -> Result<ReceiptTabData, ApiError> {
        match self {
            ReceiptsTab::Dividend => load_tab::<DividendKind>().await,
            ReceiptsTab::DomesticStock => load_tab::<DomesticStockKind>().await,
            ReceiptsTab::MutualFund => load_tab::<MutualFundKind>().await,
        }
    }
}

async fn load_tab<K: ReceiptKind>() -> Result<ReceiptTabData, ApiError> {
    let page = fetch_all_pages::<K>().await?;
    Ok(ReceiptTabData {
        rows: page
            .rows
            .into_iter()
            .map(K::wrap_row)
            .map(Row::Saved)
            .collect(),
        summary: page.summary.map(K::wrap_summary),
        truncated: page.truncated,
    })
}

#[cfg(test)]
mod tests;
