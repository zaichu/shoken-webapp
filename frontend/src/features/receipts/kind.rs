mod config;

use config::{RECEIPT_KINDS, ReceiptKind};
use std::collections::HashMap;

use rust_decimal::Decimal;
use shared::normalize::normalize_display_name;
use shared::summary::{DomesticDailyRow, domestic_daily, domestic_total};
use shared::tax::SPECIFIC_ACCOUNT_KEYWORD;

use crate::api::ApiError;
use crate::api::dto::{
    Dividend, DividendSummary, DomesticStock, DomesticStockSummary, Mutualfund, MutualfundSummary,
};
use crate::features::receipts::csv::{
    CsvPreviewRow, DividendCsvRow, DomesticStockCsvRow, MutualfundCsvRow,
};
use crate::features::receipts::model::{create_year_month_key, format_date};
use crate::features::receipts::{
    ReceiptCell, ReceiptItem, ReceiptSummary, ReceiptTabData, ReceiptsTab,
};
use crate::support::list_search::FilterConfig;
use crate::support::list_search::group_key::{GroupKeyRule, create_group_key_fn};
use crate::support::list_search::support::{ColumnReorderRule, group_and_summarize};
use crate::support::pagination::{ListEndpoint, fetch_all_pages};
use crate::support::row::Row;
use shared::format::{format_currency, format_number};

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
/// 各型は `view` で商品ごとの値に展開し、表示規則は `RowView` 側に集約する。
pub(crate) trait ReceiptRowData {
    fn view(&self) -> RowView<'_>;

    fn date(&self) -> &str {
        self.view().date()
    }
    fn code(&self) -> &str {
        self.view().code()
    }
    fn name(&self) -> &str {
        self.view().name()
    }
    fn account(&self) -> &str {
        self.view().account()
    }
    fn product(&self) -> &str {
        self.view().product()
    }
    fn is_specific(&self) -> bool {
        self.view().is_specific()
    }
    fn cells(&self) -> Vec<ReceiptCell> {
        self.view().cells()
    }
    // カードの開閉状態を引き継ぐ照合は表示丸め前の値で行う。
    // 数量 1.001 と 1.002 はともに「1.00」と出るが別行として区別する。
    fn raw_key(&self) -> String {
        self.view().raw_key()
    }
    /// 検索用の金額フィールド。`ReceiptKind::search_amounts` の列番号に対応する。
    fn search_amount(&self, index: usize) -> Decimal {
        self.view().search_amount(index)
    }
    /// グループ・ヘッダー集計用の金額 3 つ組。(対象金額, 税額, 税引後)
    fn summary_amounts(&self) -> (Decimal, Decimal, Decimal) {
        self.view().summary_amounts()
    }
    /// 国内株式の日次集計で特定口座・NISA 等に分けて使う実現損益。
    fn realized_pnl(&self) -> Decimal {
        self.view().realized_pnl()
    }
}

/// 保存済み・CSV 行どちらからも同じ形で取り出せる商品ごとの値。
/// 文字列は行からの借用で、金額はフィールドのコピー。
pub(crate) enum RowView<'a> {
    Dividend(DividendView<'a>),
    DomesticStock(DomesticStockView<'a>),
    MutualFund(MutualfundView<'a>),
}

impl<'a> RowView<'a> {
    fn date(&self) -> &'a str {
        match self {
            Self::Dividend(row) => row.date,
            Self::DomesticStock(row) => row.date,
            Self::MutualFund(row) => row.date,
        }
    }

    fn code(&self) -> &'a str {
        match self {
            Self::Dividend(row) => row.code,
            Self::DomesticStock(row) => row.code,
            Self::MutualFund(_) => "",
        }
    }

    fn name(&self) -> &'a str {
        match self {
            Self::Dividend(row) => row.name,
            Self::DomesticStock(row) => row.name,
            Self::MutualFund(row) => row.name,
        }
    }

    fn account(&self) -> &'a str {
        match self {
            Self::Dividend(row) => row.account,
            Self::DomesticStock(row) => row.account,
            Self::MutualFund(row) => row.account,
        }
    }

    fn product(&self) -> &'a str {
        match self {
            Self::Dividend(row) => row.product,
            _ => "",
        }
    }

    fn is_specific(&self) -> bool {
        self.account().contains(SPECIFIC_ACCOUNT_KEYWORD)
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
            Self::MutualFund(_) => Decimal::ZERO,
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
            Self::DomesticStock(row) => row.realized_profit_and_loss,
            _ => Decimal::ZERO,
        }
    }
}

pub(crate) struct DividendView<'a> {
    date: &'a str,
    product: &'a str,
    account: &'a str,
    code: &'a str,
    name: &'a str,
    unit_price: Decimal,
    shares: Decimal,
    dividends_before_tax: Decimal,
    taxes: Decimal,
    net_amount_received: Decimal,
}

impl DividendView<'_> {
    fn cells(&self) -> Vec<ReceiptCell> {
        vec![
            ReceiptCell::Text(format_date(self.date)),
            ReceiptCell::Text(self.product.to_string()),
            ReceiptCell::Text(self.account.to_string()),
            ReceiptCell::SecurityCode(self.code.to_string()),
            ReceiptCell::InstrumentName {
                name: normalize_display_name(self.name),
                code: Some(self.code.to_string()),
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
            self.date.to_string(),
            self.product.to_string(),
            self.account.to_string(),
            self.code.to_string(),
            self.name.to_string(),
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

impl ReceiptRowData for Dividend {
    fn view(&self) -> RowView<'_> {
        RowView::Dividend(DividendView {
            date: &self.settlement_date,
            product: &self.product,
            account: self.account.as_str(),
            code: &self.security_code,
            name: &self.security_name,
            unit_price: self.unit_price,
            shares: self.shares,
            dividends_before_tax: self.dividends_before_tax,
            taxes: self.taxes,
            net_amount_received: self.net_amount_received,
        })
    }
}

// プレビュー行は backend の検証を通った Create*Request 相当のため、
// 口座・銘柄コードは文字列のまま保持する(保存済み行と表示・検索の規則は同じ)。
impl ReceiptRowData for DividendCsvRow {
    fn view(&self) -> RowView<'_> {
        RowView::Dividend(DividendView {
            date: &self.settlement_date,
            product: &self.product,
            account: &self.account,
            code: &self.security_code,
            name: &self.security_name,
            unit_price: self.unit_price,
            shares: self.shares,
            dividends_before_tax: self.dividends_before_tax,
            taxes: self.taxes,
            net_amount_received: self.net_amount_received,
        })
    }
}

pub(crate) struct DomesticStockView<'a> {
    date: &'a str,
    code: &'a str,
    name: &'a str,
    account: &'a str,
    shares: Decimal,
    asked_price: Decimal,
    proceeds: Decimal,
    purchase_price: Decimal,
    realized_profit_and_loss: Decimal,
    taxes: Decimal,
    realized_profit_and_loss_after_tax: Decimal,
}

impl DomesticStockView<'_> {
    fn cells(&self) -> Vec<ReceiptCell> {
        vec![
            ReceiptCell::Text(format_date(self.date)),
            ReceiptCell::SecurityCode(self.code.to_string()),
            ReceiptCell::InstrumentName {
                name: normalize_display_name(self.name),
                code: Some(self.code.to_string()),
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
            self.date.to_string(),
            self.code.to_string(),
            self.name.to_string(),
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
}

impl ReceiptRowData for DomesticStock {
    fn view(&self) -> RowView<'_> {
        RowView::DomesticStock(DomesticStockView {
            date: &self.trade_date,
            code: self.security_code.as_str(),
            name: &self.security_name,
            account: self.account.as_str(),
            shares: self.shares,
            asked_price: self.asked_price,
            proceeds: self.proceeds,
            purchase_price: self.purchase_price,
            realized_profit_and_loss: self.realized_profit_and_loss,
            taxes: self.taxes,
            realized_profit_and_loss_after_tax: self.realized_profit_and_loss_after_tax,
        })
    }
}

impl ReceiptRowData for DomesticStockCsvRow {
    fn view(&self) -> RowView<'_> {
        RowView::DomesticStock(DomesticStockView {
            date: &self.trade_date,
            code: &self.security_code,
            name: &self.security_name,
            account: &self.account,
            shares: self.shares,
            asked_price: self.asked_price,
            proceeds: self.proceeds,
            purchase_price: self.purchase_price,
            realized_profit_and_loss: self.realized_profit_and_loss,
            taxes: self.taxes,
            realized_profit_and_loss_after_tax: self.realized_profit_and_loss_after_tax,
        })
    }
}

pub(crate) struct MutualfundView<'a> {
    date: &'a str,
    name: &'a str,
    account: &'a str,
    shares: Decimal,
    exchange_rate: Decimal,
    cancellation_unit_price_yen: Decimal,
    cancellation_amount_yen: Decimal,
    average_acquisition_price_yen: Decimal,
    realized_profit_and_loss: Decimal,
    taxes: Decimal,
    realized_profit_and_loss_after_tax: Decimal,
}

impl MutualfundView<'_> {
    fn cells(&self) -> Vec<ReceiptCell> {
        vec![
            ReceiptCell::Text(format_date(self.date)),
            ReceiptCell::InstrumentName {
                name: normalize_display_name(self.name),
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
            self.date.to_string(),
            self.name.to_string(),
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

    fn summary_amounts(&self) -> (Decimal, Decimal, Decimal) {
        (
            self.realized_profit_and_loss,
            self.taxes,
            self.realized_profit_and_loss_after_tax,
        )
    }
}

impl ReceiptRowData for Mutualfund {
    fn view(&self) -> RowView<'_> {
        RowView::MutualFund(MutualfundView {
            date: &self.trade_date,
            name: &self.fund_name,
            account: self.account.as_str(),
            shares: self.shares,
            exchange_rate: self.exchange_rate,
            cancellation_unit_price_yen: self.cancellation_unit_price_yen,
            cancellation_amount_yen: self.cancellation_amount_yen,
            average_acquisition_price_yen: self.average_acquisition_price_yen,
            realized_profit_and_loss: self.realized_profit_and_loss,
            taxes: self.taxes,
            realized_profit_and_loss_after_tax: self.realized_profit_and_loss_after_tax,
        })
    }
}

impl ReceiptRowData for MutualfundCsvRow {
    fn view(&self) -> RowView<'_> {
        RowView::MutualFund(MutualfundView {
            date: &self.trade_date,
            name: &self.fund_name,
            account: &self.account,
            shares: self.shares,
            exchange_rate: self.exchange_rate,
            cancellation_unit_price_yen: self.cancellation_unit_price_yen,
            cancellation_amount_yen: self.cancellation_amount_yen,
            average_acquisition_price_yen: self.average_acquisition_price_yen,
            realized_profit_and_loss: self.realized_profit_and_loss,
            taxes: self.taxes,
            realized_profit_and_loss_after_tax: self.realized_profit_and_loss_after_tax,
        })
    }
}

impl ReceiptRowData for CsvPreviewRow {
    fn view(&self) -> RowView<'_> {
        match self {
            Self::Dividend(row) => row.view(),
            Self::DomesticStock(row) => row.view(),
            Self::MutualFund(row) => row.view(),
        }
    }
}

impl ReceiptRowData for ReceiptItem {
    fn view(&self) -> RowView<'_> {
        match self {
            Self::Dividend(row) => row.view(),
            Self::DomesticStock(row) => row.view(),
            Self::MutualFund(row) => row.view(),
        }
    }
}

impl<S: ReceiptRowData, P: ReceiptRowData> ReceiptRowData for Row<S, P> {
    fn view(&self) -> RowView<'_> {
        match self {
            Self::Saved(row) => row.view(),
            Self::Preview(row) => row.view(),
        }
    }
}

/// 非 Core の列は画面帯で段階的に出す(input.css の .receipt-tier-* と対になる)。
/// 左パネルは常時 18rem を占有するため、開閉では段階を変えない
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub(crate) enum ColumnTier {
    Core,
    /// lg 帯だけ隠す
    Md,
    /// xl から出す
    Wide,
    /// 1650px から出す
    Wider,
}

impl ColumnTier {
    pub(crate) fn class(self) -> &'static str {
        match self {
            ColumnTier::Core => "",
            ColumnTier::Md => " receipt-tier-md",
            ColumnTier::Wide => " receipt-tier-wide",
            ColumnTier::Wider => " receipt-tier-wider",
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum ColumnAlign {
    Left,
    Center,
    Right,
}

impl ColumnAlign {
    pub(crate) fn class(self) -> &'static str {
        match self {
            ColumnAlign::Left => "text-left",
            ColumnAlign::Center => "text-center",
            ColumnAlign::Right => "text-right tabular-nums",
        }
    }
}

/// モバイルカードの見出し(銘柄名・日付・口座)に使う列の役割
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum CardRole {
    None,
    Name,
    Date,
    Account,
}

/// 1列分の表示属性。列の追加・並び替えはこの組で編集する
pub(crate) struct ColumnSpec {
    pub(crate) header: &'static str,
    pub(crate) width: &'static str,
    pub(crate) tier: ColumnTier,
    pub(crate) align: ColumnAlign,
    pub(crate) card_role: CardRole,
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

impl ListEndpoint for Dividend {
    type Row = Dividend;
    type Summary = DividendSummary;
    const PATH: &'static str = RECEIPT_KINDS[ReceiptsTab::Dividend as usize].list_path;
}

pub(crate) fn dividend_totals(rows: &[ReceiptRow]) -> DividendSummary {
    totals_from_amounts(rows, |(before_tax, taxes, net)| DividendSummary {
        total_dividends_before_tax: before_tax,
        total_taxes: taxes,
        total_net_amount_received: net,
    })
}

fn dividend_table_groups(
    rows: &[ReceiptRow],
    all_rows: &[ReceiptRow],
    query: &str,
) -> Vec<TableGroup> {
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

impl ListEndpoint for DomesticStock {
    type Row = DomesticStock;
    type Summary = DomesticStockSummary;
    const PATH: &'static str = RECEIPT_KINDS[ReceiptsTab::DomesticStock as usize].list_path;
}

fn domestic_table_groups(rows: &[ReceiptRow]) -> Vec<TableGroup> {
    let sorted = sorted_rows(rows);
    domestic_daily(&sorted)
        .into_iter()
        .map(|day| {
            let date = day.filter;
            let rows: Vec<_> = sorted
                .iter()
                .filter(|row| row.date() == date)
                .map(row_tuple)
                .collect();
            let summary = if rows.len() > 1 {
                vec![
                    format_currency(day.total_realized_profit_and_loss),
                    format_currency(day.total_taxes),
                    format_currency(day.total_realized_profit_and_loss_after_tax),
                ]
            } else {
                vec![]
            };
            TableGroup {
                key: date.clone(),
                label: group_label(&date),
                summary,
                rows,
            }
        })
        .collect()
}

impl ListEndpoint for Mutualfund {
    type Row = Mutualfund;
    type Summary = MutualfundSummary;
    const PATH: &'static str = RECEIPT_KINDS[ReceiptsTab::MutualFund as usize].list_path;
}

fn mutual_fund_table_groups(rows: &[ReceiptRow], query: &str) -> Vec<TableGroup> {
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

impl ReceiptsTab {
    fn kind(self) -> &'static ReceiptKind {
        &RECEIPT_KINDS[self as usize]
    }

    pub fn label(self) -> &'static str {
        self.kind().label
    }

    pub(crate) fn list_path(self) -> &'static str {
        self.kind().list_path
    }

    pub(crate) fn preview_path(self) -> &'static str {
        self.kind().preview_path
    }

    pub(crate) fn import_path(self) -> &'static str {
        self.kind().import_path
    }

    pub(crate) fn columns(self) -> &'static [ColumnSpec] {
        self.kind().columns
    }

    pub(crate) fn card_fields(self) -> CardFields {
        let find = |role| {
            self.columns()
                .iter()
                .position(|column| column.card_role == role)
                .expect("カード見出しの列が設定に存在する")
        };
        CardFields {
            name: find(CardRole::Name),
            date: find(CardRole::Date),
            account: find(CardRole::Account),
        }
    }

    pub(crate) fn summary_labels(self) -> [&'static str; 3] {
        self.kind().summary_labels
    }

    pub(crate) fn header_items(self) -> [(&'static str, bool); 3] {
        self.kind().header_items
    }

    pub(crate) fn empty_hint(self) -> &'static str {
        self.kind().empty_hint
    }

    pub(crate) fn csv_input_id(self) -> &'static str {
        self.kind().csv_input_id
    }

    pub(crate) fn product_category(self) -> bool {
        self.kind().product_category
    }

    pub(crate) fn account_category(self) -> bool {
        self.kind().account_category
    }

    pub(crate) fn reorder_rules(self) -> &'static [ColumnReorderRule<ReceiptRow>] {
        self.kind().reorder_rules
    }

    pub(crate) fn reorder_fixed(self) -> usize {
        self.kind().reorder_fixed
    }

    pub(crate) fn filter_config(self) -> FilterConfig<ReceiptRow> {
        let kind = self.kind();
        FilterConfig {
            string_fields: Some(kind.string_fields.to_vec()),
            partial_string_fields: None,
            date_field: Some(ReceiptRow::date),
            year_search: true,
            year_month_search: true,
            date_search: true,
            date_range_search: true,
            amount_fields: (!kind.search_amounts.is_empty()).then(|| kind.search_amounts.to_vec()),
        }
    }

    pub(crate) fn table_groups(
        self,
        rows: &[ReceiptRow],
        all_rows: &[ReceiptRow],
        query: &str,
    ) -> Vec<TableGroup> {
        match self {
            Self::Dividend => dividend_table_groups(rows, all_rows, query),
            Self::DomesticStock => domestic_table_groups(rows),
            Self::MutualFund => mutual_fund_table_groups(rows, query),
        }
    }

    /// API の集計行をヘッダー表示用の 3 値に変換する。タブの組が違う集計は None。
    pub(crate) fn api_summary_triple(self, summary: &ReceiptSummary) -> Option<[Decimal; 3]> {
        match (self, summary) {
            (Self::Dividend, ReceiptSummary::Dividend(summary)) => Some([
                summary.total_dividends_before_tax,
                summary.total_taxes,
                summary.total_net_amount_received,
            ]),
            (Self::DomesticStock, ReceiptSummary::DomesticStock(summary)) => Some([
                summary.total_realized_profit_and_loss,
                summary.total_taxes,
                summary.total_realized_profit_and_loss_after_tax,
            ]),
            (Self::MutualFund, ReceiptSummary::MutualFund(summary)) => Some([
                summary.total_realized_profit_and_loss,
                summary.total_taxes,
                summary.total_realized_profit_and_loss_after_tax,
            ]),
            _ => None,
        }
    }

    /// 検索・プレビュー中に使う画面側合計をヘッダー表示用の 3 値にする。
    pub(crate) fn client_summary_triple(self, rows: &[ReceiptRow]) -> [Decimal; 3] {
        match self {
            Self::DomesticStock => {
                let summary = domestic_total(rows);
                [
                    summary.total_realized_profit_and_loss,
                    summary.total_taxes,
                    summary.total_realized_profit_and_loss_after_tax,
                ]
            }
            Self::Dividend | Self::MutualFund => totals_from_amounts(rows, |(a, b, c)| [a, b, c]),
        }
    }

    pub(crate) fn parse_csv_row(self, value: serde_json::Value) -> CsvPreviewRow {
        match self {
            Self::Dividend => {
                CsvPreviewRow::Dividend(serde_json::from_value(value).unwrap_or_default())
            }
            Self::DomesticStock => {
                CsvPreviewRow::DomesticStock(serde_json::from_value(value).unwrap_or_default())
            }
            Self::MutualFund => {
                CsvPreviewRow::MutualFund(serde_json::from_value(value).unwrap_or_default())
            }
        }
    }

    pub(crate) async fn fetch_list(self) -> Result<ReceiptTabData, ApiError> {
        match self {
            // MutualfundSummary は DomesticStockSummary の別名なので
            // Into ではバリアントを選べず、集計側だけコンストラクタを渡す
            Self::Dividend => load_tab::<Dividend>(ReceiptSummary::Dividend).await,
            Self::DomesticStock => load_tab::<DomesticStock>(ReceiptSummary::DomesticStock).await,
            Self::MutualFund => load_tab::<Mutualfund>(ReceiptSummary::MutualFund).await,
        }
    }
}

async fn load_tab<E: ListEndpoint>(
    wrap_summary: fn(E::Summary) -> ReceiptSummary,
) -> Result<ReceiptTabData, ApiError>
where
    E::Row: Into<ReceiptItem>,
{
    let page = fetch_all_pages::<E>().await?;
    Ok(ReceiptTabData {
        rows: page
            .rows
            .into_iter()
            .map(Into::into)
            .map(Row::Saved)
            .collect(),
        summary: page.summary.map(wrap_summary),
        truncated: page.truncated,
    })
}

#[cfg(test)]
mod tests;
