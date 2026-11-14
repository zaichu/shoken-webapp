//! 投資信託の保有残高。国内株式(資産残高)と同じ画面に乗せるため、
//! 行の読み出し・絞り込み・並べ替え・合計はここにまとめる

use super::portfolio::SortOrder;
use crate::api::dto::{FundBalance, FundPriceSource};
use crate::features::asset_balance::format::dec_to_f64;
use crate::support::csv_flow::CsvChunking;
use crate::ui::labels;
pub const FUND_LIST_PATH: &str = "/api/v1/fund-balances";
pub const FUND_PREVIEW_PATH: &str = "/api/v1/fund-balance-import-validations";
pub const FUND_IMPORT_PATH: &str = "/api/v1/fund-balance-imports";

// 証券会社の前置き行数は口座種別で変わるため、見出し行で判定する。
// Replace 型なので import は単一リクエストのまま
pub const FUND_CSV_CHUNKING: CsvChunking = CsvChunking {
    header_lines: 6,
    import_chunked: false,
    preview_chunked: true,
    header_marker: Some(labels::FUND_KIND),
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
    pub(crate) const ALL: [Self; 3] = [Self::All, Self::Stock, Self::Fund];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::All => labels::ALL,
            Self::Stock => labels::DOMESTIC_STOCK,
            Self::Fund => labels::MUTUAL_FUND,
        }
    }

    /// タブの testid など機械向けの値
    pub(crate) fn value(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Stock => "stock",
            Self::Fund => "fund",
        }
    }
}

/// 一覧行
pub(crate) type FundBalanceRow = FundBalance;

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
            FundPriceSource::Api => labels::FUND_PRICE_SOURCE_AUTO,
            FundPriceSource::Csv => labels::FUND_PRICE_SOURCE_CSV,
        };
        if self.is_nav_latest {
            as_of.map(|d| labels::fund_day_change_title(source, &d))
        } else {
            let expected = self.expected_nav_date.as_deref().map(mmdd);
            match (expected, as_of) {
                (Some(e), Some(a)) => Some(labels::fund_pending_detail(&e, source, &a)),
                _ => Some(labels::PENDING_UPDATE.to_string()),
            }
        }
    }
}

pub(crate) fn fund_view(row: &FundBalanceRow) -> FundView {
    FundView {
        id: Some(row.id.to_string()),
        name: row.fund_name.clone(),
        account: row.account.as_str().to_string(),
        course: row.dividends_course.clone(),
        shares: dec_to_f64(&row.shares),
        average_purchase_price: dec_to_f64(&row.average_purchase_price),
        purchase: dec_to_f64(&row.total_purchase_amount),
        base_price: dec_to_f64(&row.base_price),
        base_price_day_change: row.base_price_day_change.as_ref().map(dec_to_f64),
        base_price_month_change: row.base_price_month_change.as_ref().map(dec_to_f64),
        market_value: dec_to_f64(&row.market_value),
        profit_loss: dec_to_f64(&row.profit_loss),
        profit_loss_rate: row.profit_loss_rate.as_ref().map(dec_to_f64),
        total_return: row.total_return.as_ref().map(dec_to_f64),
        price_as_of: Some(row.price_as_of.clone()),
        price_source: row.price_source,
        is_nav_latest: row.nav_is_latest,
        expected_nav_date: row.expected_nav_date.clone(),
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
                let name = row.fund_name.to_lowercase();
                let account = row.account.as_str().to_lowercase();
                if !(name.contains(&query) || account.contains(&query)) {
                    return false;
                }
            }
            if !search.accounts.is_empty()
                && !search.accounts.iter().any(|a| a == row.account.as_str())
            {
                return false;
            }
            if !search.courses.is_empty()
                && !search
                    .courses
                    .iter()
                    .any(|c| Some(c.as_str()) == row.dividends_course.as_deref())
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
        let account = row.account.as_str();
        if !account.is_empty() && !out.iter().any(|a| a == account) {
            out.push(account.to_string());
        }
    }
    out
}

pub(crate) fn fund_courses(rows: &[FundBalanceRow]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for row in rows {
        if let Some(course) = row.dividends_course.as_deref()
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
    pub(crate) const ALL: [Self; 4] = [
        Self::MarketValue,
        Self::ProfitLoss,
        Self::Shares,
        Self::BasePrice,
    ];

    /// 「すべて」タブでは両種別に共通する指標だけを出す
    pub(crate) const MERGED: [Self; 2] = [Self::MarketValue, Self::ProfitLoss];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::MarketValue => labels::MARKET_VALUE,
            Self::ProfitLoss => labels::VALUATION_PROFIT_LOSS,
            Self::Shares => labels::FUND_SHARES,
            Self::BasePrice => labels::BASE_PRICE,
        }
    }

    pub(crate) fn value(self) -> &'static str {
        match self {
            Self::MarketValue => "market_value",
            Self::ProfitLoss => "profit_loss",
            Self::Shares => "shares",
            Self::BasePrice => "base_price",
        }
    }

    pub(crate) fn from_value(value: &str) -> Option<Self> {
        match value {
            "market_value" => Some(Self::MarketValue),
            "profit_loss" => Some(Self::ProfitLoss),
            "shares" => Some(Self::Shares),
            "base_price" => Some(Self::BasePrice),
            _ => None,
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
