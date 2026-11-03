use crate::api::dto::AssetBalanceSummary;
use crate::features::asset_balance::csv::AssetBalanceRow;
use crate::features::asset_balance::format::{
    dec_to_f64, format_amount, format_signed_row_valuation_amount, format_signed_valuation_amount,
    format_signed_valuation_rate, format_valuation_amount, is_negative_valuation,
    is_positive_valuation,
};
use crate::features::asset_balance::holdings::{HoldingView, stock_daily_change_total};
use crate::features::asset_balance::model::{
    KpiHolding, ValuationInput, calculate_portfolio_kpi, portfolio_valuation, summary_valuation,
};
use crate::features::asset_balance::store::csv::ResolvedFundBalance;
use crate::features::dividend_per_share::DividendMaps;
use crate::ui::format::format_percentage_value;
use crate::ui::labels;
use crate::ui::summary::{
    SummaryBand, SummaryBreakdownEntry, SummarySubItem, SummaryTone, daily_rate,
};
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
    match fallback_totals {
        Some(summary) => summary_valuation(summary).has_unpriced,
        None => rows.iter().any(|row| row.current_price <= Decimal::ZERO),
    }
}

/// 集計帯の補助項目(右側に並べる札)
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SummaryTag {
    pub label: &'static str,
    pub value: String,
}

/// 集計帯の主要3項目(評価額・評価損益・前日比)
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PortfolioSummaryValues {
    pub market: Option<f64>,
    pub profit: Option<f64>,
    pub profit_rate: Option<f64>,
    pub daily_change: Option<f64>,
}

/// 国内株式タブの集計値。合計は絞り込み前の全保有分から求める
pub(crate) fn stock_summary_values(
    all_views: &[HoldingView],
    fallback_totals: Option<&AssetBalanceSummary>,
    dividends: &DividendMaps,
) -> (PortfolioSummaryValues, Vec<SummaryTag>) {
    let kpi_holdings: Vec<KpiHolding> = all_views
        .iter()
        .map(|view| KpiHolding {
            security_code: view.code.clone(),
            shares: view.shares,
            total_purchase_amount: view.purchase,
        })
        .collect();
    let valuation_inputs: Vec<ValuationInput> =
        all_views.iter().map(ValuationInput::from).collect();
    let row_valuation = portfolio_valuation(&valuation_inputs);
    let valuation = fallback_totals
        .map(summary_valuation)
        .unwrap_or(row_valuation);
    // 前日比の合計は API 集計(全件)を優先し、無ければ全行から計算する。
    // 前日終値なしの行は除く(全部なしなら「—」)
    let daily_change = fallback_totals
        .map(|summary| dec_to_f64(&summary.total_daily_change))
        .or_else(|| stock_daily_change_total(all_views));
    let kpi = calculate_portfolio_kpi(&kpi_holdings, &dividends.per_share);
    let values = PortfolioSummaryValues {
        market: valuation.market_value,
        profit: valuation.profit_loss.amount,
        profit_rate: valuation.profit_loss.rate,
        daily_change,
    };
    let tags = vec![
        SummaryTag {
            label: labels::ANNUAL_DIVIDEND_FORECAST,
            value: kpi
                .total_annual_dividends
                .map_or("—".to_string(), format_amount),
        },
        SummaryTag {
            label: labels::ACQUISITION_YIELD,
            value: kpi
                .dividend_yield
                .map_or("—".to_string(), format_percentage_value),
        },
    ];
    (values, tags)
}

/// 投資信託・すべてのタブの補助の札(取得総額)
pub(crate) fn purchase_summary_tag(purchase: f64) -> SummaryTag {
    SummaryTag {
        label: labels::TOTAL_ACQUISITION,
        value: format_valuation_amount(Some(purchase)),
    }
}

/// 資産管理の集計帯。国内株式・投資信託・すべての3タブで同じ部品を使う
#[component]
pub(crate) fn AssetSummaryBand(
    market: Option<f64>,
    profit: Option<f64>,
    profit_rate: Option<f64>,
    /// 前日比の合計。行から求められないときは None
    daily_change: Option<f64>,
    tags: Vec<SummaryTag>,
    /// 取得総額(内訳用)。国内株式は呼び出し側で全保有分から求める
    purchase: Option<f64>,
    /// 帯のラベルの横に出す時点(株価・基準価額の最も古い日)。空なら出さない
    #[prop(optional, into)]
    as_of: String,
    /// 更新待ちの行があるときの小さな表示
    #[prop(optional)]
    show_pending: bool,
) -> impl IntoView {
    let market_text = format_valuation_amount(market);
    let profit_text = format_signed_valuation_amount(profit);
    let rate_text = profit_rate
        .map(|rate| format_signed_valuation_rate(Some(rate), 1))
        .unwrap_or_default();
    // 前日比は表・カードと同じく表示単位を適用せず必ず円マーク付きで出す
    let daily_text = format_signed_row_valuation_amount(daily_change);
    let daily_rate_text = daily_rate(market, daily_change)
        .map(|rate| format_signed_valuation_rate(Some(rate), 1))
        .unwrap_or_default();
    let profit_negative = is_negative_valuation(profit);
    let profit_positive = is_positive_valuation(profit);
    let daily_negative = is_negative_valuation(daily_change);
    let daily_positive = is_positive_valuation(daily_change);
    let purchase_text = format_valuation_amount(purchase);
    let pc_items: Vec<SummarySubItem> = {
        let mut items = vec![SummarySubItem {
            label: labels::DAY_CHANGE.to_string(),
            value: daily_text.clone(),
            note: String::new(),
            tone: if daily_negative {
                SummaryTone::Loss
            } else if daily_positive {
                SummaryTone::Gain
            } else {
                SummaryTone::Neutral
            },
            extra_class: String::new(),
        }];
        for (index, tag) in tags.iter().take(2).enumerate() {
            items.push(SummarySubItem {
                label: tag.label.to_string(),
                value: tag.value.clone(),
                note: String::new(),
                tone: SummaryTone::Neutral,
                extra_class: if index == 0 {
                    "summary-band-item-tag1".to_string()
                } else {
                    String::new()
                },
            });
        }
        items
    };
    view! {
        <section
            id="assetbalance-summary"
            class="summary-band"
            role="region"
            aria-label=labels::SUMMARY
            data-testid="asset-portfolio-summary"
        >
            <SummaryBand
                primary_label=labels::MARKET_VALUE
                primary_value=market_text
                primary_as_of=as_of
                show_pending=show_pending
                aux_label=labels::VALUATION_PROFIT_LOSS
                aux_value=profit_text
                aux_rate=rate_text
                aux_negative=profit_negative
                aux_positive=profit_positive
                pc_items=pc_items
                breakdown=vec![
                    SummaryBreakdownEntry {
                        label: labels::DAY_CHANGE.to_string(),
                        value: daily_text,
                        negative: daily_negative,
                        positive: daily_positive,
                        rate: daily_rate_text,
                    },
                    SummaryBreakdownEntry {
                        label: labels::TOTAL_ACQUISITION.to_string(),
                        value: purchase_text,
                        negative: false,
                        positive: false,
                        rate: String::new(),
                    },
                ]
                testid_band="portfolio-kpi-grid"
                testid_toggle="asset-summary-details-toggle"
            />
        </section>
    }
}
