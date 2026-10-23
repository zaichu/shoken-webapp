//! 投資信託の保有残高。国内株式(資産残高)と同じ画面に乗せるため、
//! 行の読み出し・絞り込み・並べ替え・合計はここにまとめる

use super::portfolio::SortOrder;
use crate::api::dto::{FundBalance, FundPriceSource};
use crate::features::asset_balance::format::dec_to_f64;
use crate::support::csv_flow::CsvChunking;
use crate::support::row::Row;
use rust_decimal::Decimal;
use serde::Deserialize;

pub const FUND_LIST_PATH: &str = "/api/v1/fund-balances";
pub const FUND_PREVIEW_PATH: &str = "/api/v1/fund-balance-import-validations";
pub const FUND_IMPORT_PATH: &str = "/api/v1/fund-balance-imports";

// 証券会社の前置き行数は口座種別で変わるため、見出し行で判定する。
// Replace 型なので import は単一リクエストのまま
pub const FUND_CSV_CHUNKING: CsvChunking = CsvChunking {
    header_lines: 6,
    import_chunked: false,
    preview_chunked: true,
    header_marker: Some("投資信託種別"),
};

/// 資産管理で扱う保有の種別。一覧の切り替えと CSV の判別に使う
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AssetKind {
    #[default]
    All,
    Stock,
    Fund,
}

impl AssetKind {
    pub(crate) const ALL: [AssetKind; 3] = [AssetKind::All, AssetKind::Stock, AssetKind::Fund];

    pub(crate) fn label(self) -> &'static str {
        match self {
            AssetKind::All => "すべて",
            AssetKind::Stock => "国内株式",
            AssetKind::Fund => "投資信託",
        }
    }

    /// タブの testid など機械向けの値
    pub(crate) fn value(self) -> &'static str {
        match self {
            AssetKind::All => "all",
            AssetKind::Stock => "stock",
            AssetKind::Fund => "fund",
        }
    }
}

/// CSV プレビュー行。backend の CreateFundBalanceRequest そのままの形を
/// lenient に受け取る(検証に通らない値も画面に出すため)
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct FundBalanceCsvRow {
    #[serde(default)]
    pub fund_name: String,
    #[serde(default)]
    pub account: String,
    #[serde(default)]
    pub dividends_course: Option<String>,
    #[serde(default)]
    pub shares: Decimal,
    #[serde(default)]
    pub average_purchase_price: Decimal,
    #[serde(default)]
    pub total_purchase_amount: Decimal,
    #[serde(default)]
    pub base_price: Decimal,
    #[serde(default)]
    pub base_price_day_change: Option<Decimal>,
    #[serde(default)]
    pub base_price_month_change: Option<Decimal>,
    #[serde(default)]
    pub market_value: Decimal,
    #[serde(default)]
    pub profit_loss: Decimal,
    #[serde(default)]
    pub profit_loss_rate: Option<Decimal>,
    #[serde(default)]
    pub total_return: Option<Decimal>,
}

/// 一覧行と CSV プレビュー行を束ねる
pub(crate) type FundBalanceRow = Row<FundBalance, FundBalanceCsvRow>;

/// 保存済み行とプレビュー行の共通読み出し面
pub(crate) trait FundRowData {
    fn fund_name(&self) -> &str;
    fn account(&self) -> &str;
    fn dividends_course(&self) -> Option<&str>;
    fn shares(&self) -> Decimal;
    fn average_purchase_price(&self) -> Decimal;
    fn total_purchase_amount(&self) -> Decimal;
    fn base_price(&self) -> Decimal;
    fn base_price_day_change(&self) -> Option<Decimal>;
    fn base_price_month_change(&self) -> Option<Decimal>;
    fn market_value(&self) -> Decimal;
    fn profit_loss(&self) -> Decimal;
    fn profit_loss_rate(&self) -> Option<Decimal>;
    fn total_return(&self) -> Option<Decimal>;
    fn price_as_of(&self) -> Option<String>;
    /// 基準価額まわりの値の出どころ(CSV か公式APIの共有値)
    fn price_source(&self) -> FundPriceSource;
    /// 前日比が「最新の基準価額の日の変動」として出せるか。false は「更新待ち」
    fn is_nav_latest(&self) -> bool;
    /// 「公表済みのはずの最新の基準価額の日」(更新待ちの説明文に使う)
    fn expected_nav_date(&self) -> Option<String>;
    /// 保有残高の取込時刻(CSV 取り込みの時刻)。プレビュー行には無い
    fn created_at(&self) -> Option<&str>;
}

impl FundRowData for FundBalance {
    fn fund_name(&self) -> &str {
        &self.fund_name
    }
    fn account(&self) -> &str {
        self.account.as_str()
    }
    fn dividends_course(&self) -> Option<&str> {
        self.dividends_course.as_deref()
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
    fn base_price(&self) -> Decimal {
        self.base_price
    }
    fn base_price_day_change(&self) -> Option<Decimal> {
        self.base_price_day_change
    }
    fn base_price_month_change(&self) -> Option<Decimal> {
        self.base_price_month_change
    }
    fn market_value(&self) -> Decimal {
        self.market_value
    }
    fn profit_loss(&self) -> Decimal {
        self.profit_loss
    }
    fn profit_loss_rate(&self) -> Option<Decimal> {
        self.profit_loss_rate
    }
    fn total_return(&self) -> Option<Decimal> {
        self.total_return
    }
    fn price_as_of(&self) -> Option<String> {
        Some(self.price_as_of.to_string())
    }
    fn price_source(&self) -> FundPriceSource {
        self.price_source
    }
    fn is_nav_latest(&self) -> bool {
        self.nav_is_latest
    }
    fn expected_nav_date(&self) -> Option<String> {
        self.expected_nav_date.clone()
    }
    fn created_at(&self) -> Option<&str> {
        Some(self.created_at.as_str())
    }
}

impl FundRowData for FundBalanceCsvRow {
    fn fund_name(&self) -> &str {
        &self.fund_name
    }
    fn account(&self) -> &str {
        &self.account
    }
    fn dividends_course(&self) -> Option<&str> {
        self.dividends_course.as_deref()
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
    fn base_price(&self) -> Decimal {
        self.base_price
    }
    fn base_price_day_change(&self) -> Option<Decimal> {
        self.base_price_day_change
    }
    fn base_price_month_change(&self) -> Option<Decimal> {
        self.base_price_month_change
    }
    fn market_value(&self) -> Decimal {
        self.market_value
    }
    fn profit_loss(&self) -> Decimal {
        self.profit_loss
    }
    fn profit_loss_rate(&self) -> Option<Decimal> {
        self.profit_loss_rate
    }
    fn total_return(&self) -> Option<Decimal> {
        self.total_return
    }
    fn price_as_of(&self) -> Option<String> {
        None
    }
    fn price_source(&self) -> FundPriceSource {
        FundPriceSource::Csv
    }
    fn is_nav_latest(&self) -> bool {
        // プレビューは取り込むファイルの値そのものなので「最新」として扱う
        true
    }
    fn expected_nav_date(&self) -> Option<String> {
        None
    }
    fn created_at(&self) -> Option<&str> {
        None
    }
}

impl FundBalanceRow {
    fn inner(&self) -> &dyn FundRowData {
        match self {
            Self::Saved(row) => row,
            Self::Preview(row) => row,
        }
    }

    /// 詳細ページのパスに使う保存済みレコード ID。CSV プレビュー行には無い
    pub(crate) fn saved_id(&self) -> Option<String> {
        self.saved().map(|row| row.id.to_string())
    }
}

impl FundRowData for FundBalanceRow {
    fn fund_name(&self) -> &str {
        self.inner().fund_name()
    }
    fn account(&self) -> &str {
        self.inner().account()
    }
    fn dividends_course(&self) -> Option<&str> {
        self.inner().dividends_course()
    }
    fn shares(&self) -> Decimal {
        self.inner().shares()
    }
    fn average_purchase_price(&self) -> Decimal {
        self.inner().average_purchase_price()
    }
    fn total_purchase_amount(&self) -> Decimal {
        self.inner().total_purchase_amount()
    }
    fn base_price(&self) -> Decimal {
        self.inner().base_price()
    }
    fn base_price_day_change(&self) -> Option<Decimal> {
        self.inner().base_price_day_change()
    }
    fn base_price_month_change(&self) -> Option<Decimal> {
        self.inner().base_price_month_change()
    }
    fn market_value(&self) -> Decimal {
        self.inner().market_value()
    }
    fn profit_loss(&self) -> Decimal {
        self.inner().profit_loss()
    }
    fn profit_loss_rate(&self) -> Option<Decimal> {
        self.inner().profit_loss_rate()
    }
    fn total_return(&self) -> Option<Decimal> {
        self.inner().total_return()
    }
    fn price_as_of(&self) -> Option<String> {
        self.inner().price_as_of()
    }
    fn price_source(&self) -> FundPriceSource {
        self.inner().price_source()
    }
    fn is_nav_latest(&self) -> bool {
        self.inner().is_nav_latest()
    }
    fn expected_nav_date(&self) -> Option<String> {
        self.inner().expected_nav_date()
    }
    fn created_at(&self) -> Option<&str> {
        self.inner().created_at()
    }
}

/// 一覧・詳細に渡す1行分の表示用データ
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FundView {
    pub id: Option<String>,
    pub name: String,
    pub account: String,
    pub course: Option<String>,
    pub shares: f64,
    pub average_purchase_price: f64,
    pub purchase: f64,
    pub base_price: f64,
    pub base_price_day_change: Option<f64>,
    pub base_price_month_change: Option<f64>,
    pub market_value: f64,
    pub profit_loss: f64,
    pub profit_loss_rate: Option<f64>,
    pub total_return: Option<f64>,
    pub price_as_of: Option<String>,
    pub price_source: FundPriceSource,
    pub is_nav_latest: bool,
    pub expected_nav_date: Option<String>,
}

impl FundView {
    /// 「更新待ち」の行。前日比は最新の基準価額の日の変動でないため出さない
    pub(crate) fn nav_pending(&self) -> bool {
        !self.is_nav_latest
    }

    /// 表の「前日比」セルに出す値。更新待ちの行は古い日の前日比を見せない
    pub(crate) fn view_day_change(&self) -> Option<f64> {
        if self.is_nav_latest {
            self.base_price_day_change
        } else {
            None
        }
    }

    /// 前日比セルの title。出どころと日付を示し、「更新待ち」は理由を添える
    pub(crate) fn day_change_title(&self) -> Option<String> {
        let mmdd = |d: &str| {
            let mut it = d.split('-');
            match (it.next(), it.next(), it.next()) {
                (Some(_y), Some(m), Some(d)) => {
                    format!(
                        "{}/{}",
                        m.trim_start_matches('0'),
                        d.trim_start_matches('0')
                    )
                }
                _ => d.to_string(),
            }
        };
        let as_of = self.price_as_of.as_deref().map(mmdd);
        let source = match self.price_source {
            FundPriceSource::Api => "自動更新",
            FundPriceSource::Csv => "CSV 取り込み",
        };
        if self.is_nav_latest {
            as_of.map(|d| format!("{source} {d} の前日比"))
        } else {
            let expected = self.expected_nav_date.as_deref().map(mmdd);
            match (expected, as_of) {
                (Some(e), Some(a)) => Some(format!(
                    "更新待ち: 最新の基準価額の日は {e}、この値は{source} {a}"
                )),
                _ => Some("更新待ち".to_string()),
            }
        }
    }
}

pub(crate) fn fund_view(row: &FundBalanceRow) -> FundView {
    FundView {
        id: row.saved_id(),
        name: row.fund_name().to_string(),
        account: row.account().to_string(),
        course: row.dividends_course().map(str::to_string),
        shares: dec_to_f64(&row.shares()),
        average_purchase_price: dec_to_f64(&row.average_purchase_price()),
        purchase: dec_to_f64(&row.total_purchase_amount()),
        base_price: dec_to_f64(&row.base_price()),
        base_price_day_change: row.base_price_day_change().as_ref().map(dec_to_f64),
        base_price_month_change: row.base_price_month_change().as_ref().map(dec_to_f64),
        market_value: dec_to_f64(&row.market_value()),
        profit_loss: dec_to_f64(&row.profit_loss()),
        profit_loss_rate: row.profit_loss_rate().as_ref().map(dec_to_f64),
        total_return: row.total_return().as_ref().map(dec_to_f64),
        price_as_of: row.price_as_of(),
        price_source: row.price_source(),
        is_nav_latest: row.is_nav_latest(),
        expected_nav_date: row.expected_nav_date(),
    }
}

/// ファンドの絞り込み条件。口座・分配金コースは複数選択
#[derive(Clone, Debug, Default)]
pub(crate) struct AssetSearch {
    pub text: String,
    pub accounts: Vec<String>,
    pub courses: Vec<String>,
}

impl AssetSearch {
    pub(crate) fn is_empty(&self) -> bool {
        self.text.trim().is_empty() && self.accounts.is_empty() && self.courses.is_empty()
    }
}

/// テキストだけの検索条件。口座・コースの選択肢はタブごとに語彙が違うため、
/// 「すべて」の絞り込みやタブ切替の直後はテキストのみ引き継ぐ
pub(crate) fn text_only_search(search: &AssetSearch) -> AssetSearch {
    AssetSearch {
        text: search.text.clone(),
        ..Default::default()
    }
}

/// ファンド名・口座の部分一致と、口座・分配金コースの選択条件で絞る
pub(crate) fn filter_fund_rows<'a>(
    rows: &'a [FundBalanceRow],
    search: &AssetSearch,
) -> Vec<&'a FundBalanceRow> {
    let query = search.text.trim().to_lowercase();
    rows.iter()
        .filter(|row| {
            if !query.is_empty() {
                let name = row.fund_name().to_lowercase();
                let account = row.account().to_lowercase();
                if !(name.contains(&query) || account.contains(&query)) {
                    return false;
                }
            }
            if !search.accounts.is_empty() && !search.accounts.iter().any(|a| a == row.account()) {
                return false;
            }
            if !search.courses.is_empty()
                && !search
                    .courses
                    .iter()
                    .any(|c| Some(c.as_str()) == row.dividends_course())
            {
                return false;
            }
            true
        })
        .collect()
}

/// 行に出ている口座・分配金コースの選択肢(重複なし・出現順)
pub(crate) fn fund_accounts(rows: &[FundBalanceRow]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for row in rows {
        let account = row.account();
        if !account.is_empty() && !out.iter().any(|a| a == account) {
            out.push(account.to_string());
        }
    }
    out
}

pub(crate) fn fund_courses(rows: &[FundBalanceRow]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for row in rows {
        if let Some(course) = row.dividends_course()
            && !course.is_empty()
            && !out.iter().any(|c| c == course)
        {
            out.push(course.to_string());
        }
    }
    out
}

/// ファンド一覧の並べ替えキー
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FundSortKey {
    MarketValue,
    ProfitLoss,
    Shares,
    BasePrice,
}

impl FundSortKey {
    /// 投資信託タブの選択肢
    pub(crate) const ALL: [FundSortKey; 4] = [
        FundSortKey::MarketValue,
        FundSortKey::ProfitLoss,
        FundSortKey::Shares,
        FundSortKey::BasePrice,
    ];

    /// 「すべて」タブでは両種別に共通する指標だけを出す
    pub(crate) const MERGED: [FundSortKey; 2] = [FundSortKey::MarketValue, FundSortKey::ProfitLoss];

    pub(crate) fn label(self) -> &'static str {
        match self {
            FundSortKey::MarketValue => "評価額",
            FundSortKey::ProfitLoss => "評価損益",
            FundSortKey::Shares => "保有口数",
            FundSortKey::BasePrice => "基準価額",
        }
    }

    pub(crate) fn value(self) -> &'static str {
        match self {
            FundSortKey::MarketValue => "market_value",
            FundSortKey::ProfitLoss => "profit_loss",
            FundSortKey::Shares => "shares",
            FundSortKey::BasePrice => "base_price",
        }
    }
}

/// 行から取り出す並べ替え値
fn sort_key_value(view: &FundView, key: FundSortKey) -> f64 {
    match key {
        FundSortKey::MarketValue => view.market_value,
        FundSortKey::ProfitLoss => view.profit_loss,
        FundSortKey::Shares => view.shares,
        FundSortKey::BasePrice => view.base_price,
    }
}

/// インデックスの並べ替え。同値はファンド名順
pub(crate) fn sort_fund_views(
    views: &[FundView],
    key: FundSortKey,
    order: SortOrder,
) -> Vec<usize> {
    let mut indexed: Vec<usize> = (0..views.len()).collect();
    indexed.sort_by(|&a, &b| {
        let ordering = sort_key_value(&views[b], key).total_cmp(&sort_key_value(&views[a], key));
        let ordering = match order {
            SortOrder::Desc => ordering,
            SortOrder::Asc => ordering.reverse(),
        };
        if ordering == std::cmp::Ordering::Equal {
            views[a].name.cmp(&views[b].name)
        } else {
            ordering
        }
    });
    indexed
}

/// 一覧の合計。daily_change は保有数量で重み付けした基準価額(前日比)の円換算。
/// 「最新」の前日比を持つ行だけを足す(更新待ちの行は古い日の変動なので入れない)
#[derive(Clone, Debug, Default)]
pub(crate) struct FundTotals {
    pub market: f64,
    pub purchase: f64,
    pub profit: f64,
    pub daily_change: f64,
    pub has_daily_change: bool,
    /// 前日比の合計に入った(最新で前日比を持つ)ファンド数
    pub latest_nav_count: usize,
    /// 「更新待ち」のファンド数
    pub pending_nav_count: usize,
}

pub(crate) fn fund_totals(views: &[FundView]) -> FundTotals {
    let mut totals = FundTotals::default();
    for view in views {
        totals.market += view.market_value;
        totals.purchase += view.purchase;
        totals.profit += view.profit_loss;
        if view.nav_pending() {
            totals.pending_nav_count += 1;
            continue;
        }
        if let Some(change) = view.base_price_day_change {
            totals.daily_change += change * view.shares / 10_000.0;
            totals.has_daily_change = true;
            totals.latest_nav_count += 1;
        }
    }
    totals
}

#[cfg(test)]
mod tests;
