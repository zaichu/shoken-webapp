use crate::api::dto::AssetBalanceSummary;
use crate::features::asset_balance::csv::AssetBalanceRow;
use crate::features::asset_balance::format::{
    format_signed_valuation_amount, format_signed_valuation_rate, format_valuation_amount,
    is_negative_valuation, is_positive_valuation,
};
use crate::features::asset_balance::holdings::HoldingView;
use crate::features::asset_balance::model::{
    ValuationInput, portfolio_valuation, summary_valuation,
};
use crate::features::asset_balance::store::csv::ResolvedFundBalance;
use crate::ui::labels;
use crate::ui::summary::{SummaryBand, SummaryStat, SummaryTone, with_pending_note};
use leptos::prelude::*;
use rust_decimal::Decimal;

/// 投資信託タブの件数。行が取り切れていないときは API 集計の総件数を使い、
/// 集計欄と食い違わないようにする
pub(crate) fn fund_tab_count(rows_len: usize, total: usize) -> usize {
    total.max(rows_len)
}

/// 見出しの基準日と表の日付出し分けで共通に使う、価格あり行の基準日一覧
/// (価格が出ていない行の日付は「その時点の株価」と結びつかないので除く)
pub(crate) fn stock_price_dates(rows: &[AssetBalanceRow]) -> Vec<&str> {
    rows.iter()
        .filter(|row| row.current_price > Decimal::ZERO)
        .filter_map(|row| row.price_as_of.as_deref())
        .collect()
}

/// 帯のラベルの横に出す時点。混在時は最も古い日を出し、嘘の新しさを見せない
pub(crate) fn oldest_stock_as_of(rows: &[AssetBalanceRow]) -> Option<String> {
    let mut dates = stock_price_dates(rows);
    dates.sort_unstable();
    dates.first().map(|date| labels::price_as_of(date))
}

/// 投信の基準日のうち最も古い日を帯の時点に出す
pub(crate) fn oldest_fund_as_of(view: &ResolvedFundBalance) -> Option<String> {
    let mut dates: Vec<String> = view
        .rows
        .iter()
        .map(|row| row.price_as_of.clone())
        .collect();
    dates.sort();
    dates.first().map(|date| labels::price_as_of(date))
}

/// 株と投信を合わせた時点。混在時は最も古い日を出す
pub(crate) fn oldest_combined_as_of(
    stock_rows: &[AssetBalanceRow],
    fund_view: &ResolvedFundBalance,
) -> Option<String> {
    let mut dates: Vec<String> = stock_price_dates(stock_rows)
        .into_iter()
        .map(str::to_string)
        .collect();
    dates.extend(fund_view.rows.iter().map(|row| row.price_as_of.clone()));
    dates.sort();
    dates.first().map(|date| labels::price_as_of(date))
}

/// 投信に更新待ちの行があるか。前日比の合計に入っていない行があれば true
pub(crate) fn fund_has_pending(view: &ResolvedFundBalance) -> bool {
    view.rows.iter().any(|row| !row.nav_is_latest)
}

/// 行と API 集計から更新待ちを判定する(一覧の部品用)
pub(crate) fn stock_pending_from_parts(
    rows: &[AssetBalanceRow],
    fallback_totals: Option<&AssetBalanceSummary>,
) -> bool {
    if rows.is_empty() {
        return false;
    }
    fallback_totals.map_or_else(
        || rows.iter().any(|row| row.current_price <= Decimal::ZERO),
        |summary| summary_valuation(summary).has_unpriced,
    )
}

/// 集計帯の主要2項目(評価額・評価損益)
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PortfolioSummaryValues {
    pub market: Option<f64>,
    pub profit: Option<f64>,
    pub profit_rate: Option<f64>,
}

/// 国内株式タブの集計値。合計は絞り込み前の全保有分から求める
pub(crate) fn stock_summary_values(
    all_views: &[HoldingView],
    fallback_totals: Option<&AssetBalanceSummary>,
) -> PortfolioSummaryValues {
    let valuation_inputs: Vec<ValuationInput> =
        all_views.iter().map(ValuationInput::from).collect();
    let row_valuation = portfolio_valuation(&valuation_inputs);
    let valuation = fallback_totals.map_or(row_valuation, summary_valuation);
    PortfolioSummaryValues {
        market: valuation.market_value,
        profit: valuation.profit_loss.amount,
        profit_rate: valuation.profit_loss.rate,
    }
}

/// 資産管理の集計帯。国内株式・投資信託・すべての3タブで同じ部品を使う
#[component]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Leptos component props are owned by the view"
)]
pub(crate) fn AssetSummaryBand(
    market: Option<f64>,
    profit: Option<f64>,
    profit_rate: Option<f64>,
    /// 帯の基準日(株価・基準価額の最も古い日)。空なら出さない
    #[prop(optional, into)]
    as_of: String,
    /// 更新待ちの行があるときの小さな表示
    #[prop(optional)]
    show_pending: bool,
) -> impl IntoView {
    let market_text = format_valuation_amount(market);
    let profit_text = format_signed_valuation_amount(profit);
    let rate_text = profit_rate.map_or_default(|rate| format_signed_valuation_rate(Some(rate), 1));
    let profit_negative = is_negative_valuation(profit);
    let profit_positive = is_positive_valuation(profit);
    let dates_note = with_pending_note(&as_of, show_pending);
    let pc_dates: Vec<String> = vec![dates_note.clone()]
        .into_iter()
        .filter(|line| !line.is_empty())
        .collect();
    view! {
        <section
            id="assetbalance-summary"
            class="summary-band"
            role="region"
            aria-label=labels::SUMMARY
            data-testid="asset-portfolio-summary"
        >
            <SummaryBand
                primary=SummaryStat {
                    label: labels::MARKET_VALUE.to_string(),
                    value: market_text,
                    note: String::new(),
                    tone: SummaryTone::Neutral,
                }
                secondary=SummaryStat {
                    label: labels::VALUATION_PROFIT_LOSS.to_string(),
                    value: profit_text,
                    note: rate_text,
                    tone: if profit_negative {
                        SummaryTone::Loss
                    } else if profit_positive {
                        SummaryTone::Gain
                    } else {
                        SummaryTone::Neutral
                    },
                }
                mobile_dates=dates_note
                pc_dates=pc_dates
                testid_band="portfolio-kpi-grid"
            />
        </section>
    }
}
