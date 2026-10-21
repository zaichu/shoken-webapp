use crate::api::dto::AssetBalanceSummary;
use crate::features::asset_balance::csv::{AssetBalanceRow, AssetBalanceRowData};
use crate::features::asset_balance::format::{
    dec_to_f64, format_amount, format_nav_updated_at, format_percentage_value, format_price_date,
    format_signed_row_valuation_amount, format_signed_valuation_amount,
    format_signed_valuation_rate, format_valuation_amount, is_negative_valuation,
    is_positive_valuation, price_date_representative,
};
use crate::features::asset_balance::fund::{fund_totals, fund_view};
use crate::features::asset_balance::holdings::{
    HoldingView, holding_view, stock_daily_change_total,
};
use crate::features::asset_balance::model::{
    KpiHolding, ValuationInput, calculate_portfolio_kpi, portfolio_valuation, summary_valuation,
};
use crate::features::asset_balance::store::csv::{ResolvedAssetBalance, ResolvedFundBalance};
use crate::features::dividend_per_share::DividendMaps;
use crate::ui::amount::Amount;
use leptos::prelude::*;
use rust_decimal::Decimal;
use shared::domain::FundNavStatus;

fn unpriced_suffix(has_unpriced: bool) -> &'static str {
    if has_unpriced {
        "・価格なし銘柄を除く"
    } else {
        ""
    }
}

/// 前日終値が取れない行を合計から除いたときの注記
fn missing_day_change_suffix(has_missing: bool) -> &'static str {
    if has_missing {
        "・前日比なし銘柄を除く"
    } else {
        ""
    }
}

/// PC ヘッダー横の件数。取得上限超過で行が欠けるときは API 集計側の件数を使い、
/// 集計欄と食い違わないようにする。基準日と除外した行の説明は帯の直下の注記に出す
pub(crate) fn list_subtitle(view: &ResolvedAssetBalance) -> String {
    format!("{}件", view.total)
}

pub(crate) fn fund_subtitle(view: &ResolvedFundBalance) -> String {
    format!("{}件", view.total)
}

pub(crate) fn merged_subtitle(
    stock: Option<&ResolvedAssetBalance>,
    fund: Option<&ResolvedFundBalance>,
) -> Option<String> {
    if stock.is_none() && fund.is_none() {
        return None;
    }
    let count =
        stock.map(|view| view.total).unwrap_or(0) + fund.map(|view| view.total).unwrap_or(0);
    Some(format!("{count}件"))
}

/// 「前日比が最新」の件数(前日比を持ち合計に入る行)。
/// 行が取り切れていないときは API 集計を優先する
pub(crate) fn fund_latest_nav_count(view: &ResolvedFundBalance) -> usize {
    match &view.fallback_totals {
        Some(summary) => summary.latest_nav_count as usize,
        None => {
            let views: Vec<_> = view.rows.iter().map(fund_view).collect();
            fund_totals(&views).latest_nav_count
        }
    }
}

/// 基準価額の自動更新の状況。取得できた/失敗した件数と、まだ取得していないときの文言で
/// 3状態を区別する。失敗の理由は API が丸めた短い文をそのまま使う
pub(crate) fn fund_nav_note(status: &FundNavStatus) -> String {
    let prefix = match status.updated_at.as_deref() {
        Some(updated_at) => format!("基準価額の自動更新: {}", format_nav_updated_at(updated_at)),
        None => "基準価額の自動更新".to_string(),
    };
    if status.failed_count > 0 {
        let reason = status.failure_reason.as_deref().unwrap_or("理由不明");
        let counts = if status.fetched_count > 0 {
            format!(
                "取得 {}件・失敗 {}件",
                status.fetched_count, status.failed_count
            )
        } else {
            format!("失敗 {}件", status.failed_count)
        };
        return format!("{prefix}({counts}: {reason})");
    }
    if status.fetched_count > 0 {
        return format!("{prefix}(取得 {}件)", status.fetched_count);
    }
    "基準価額の自動更新: まだ取得されていません".to_string()
}

/// 見出しの基準日と表の日付出し分けで共通に使う、価格あり行の基準日一覧
/// (価格が出ていない行の日付は「その時点の株価」と結びつかないので除く)
pub(crate) fn stock_price_dates(rows: &[AssetBalanceRow]) -> Vec<&str> {
    rows.iter()
        .filter(|row| row.current_price() > Decimal::ZERO)
        .filter_map(|row| row.price_as_of())
        .collect()
}

/// 帯の直下に出す、株価の基準日と合計から除いた行の付記。
/// 基準日は数字の根拠なので、付記が無くても常に出す
pub(crate) fn stock_note(view: &ResolvedAssetBalance) -> String {
    if view.rows.is_empty() {
        return String::new();
    }
    let (has_unpriced, has_missing_daily) = match &view.fallback_totals {
        Some(summary) => (
            summary_valuation(summary).has_unpriced,
            summary.has_missing_daily_change,
        ),
        None => {
            let views: Vec<_> = view.rows.iter().map(holding_view).collect();
            let inputs: Vec<ValuationInput> = views.iter().map(ValuationInput::from).collect();
            (
                portfolio_valuation(&inputs).has_unpriced,
                views.iter().any(|view| view.day_change.is_none()),
            )
        }
    };
    let dates = stock_price_dates(&view.rows);
    stock_note_text(has_unpriced, has_missing_daily, &dates)
}

fn stock_note_text(has_unpriced: bool, has_missing_daily: bool, dates: &[&str]) -> String {
    let base = match price_date_representative(dates.iter().copied()) {
        Some(date) => format!("株価 {} 時点(銘柄ごとの最新終値)", format_price_date(date)),
        None => "株価の基準日は不明".to_string(),
    };
    format!(
        "{base}{}{}",
        unpriced_suffix(has_unpriced),
        missing_day_change_suffix(has_missing_daily)
    )
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
            label: "年間配当(予想)",
            value: kpi
                .total_annual_dividends
                .map_or("—".to_string(), format_amount),
        },
        SummaryTag {
            label: "取得額利回り",
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
        label: "取得総額",
        value: format_valuation_amount(Some(purchase)),
    }
}

/// 「計算基準を見る」の中身(国内株式)
pub(crate) fn stock_basis_lines() -> Vec<String> {
    vec![
        "評価額 = 現在値 × 株数(価格なし銘柄は除く。現在値は銘柄ごとの最新終値で、日付は銘柄ごとに違い、他と違う行は表の「前日比」の横に表示)".to_string(),
        "前日比 = (現在値 − 前日終値) × 株数の合計(前日終値なし銘柄は除く。1株あたりの前日比は表の「前日比」に表示)".to_string(),
        "評価損益 = 評価額 − 取得総額、率 = 評価損益 ÷ 取得総額(価格あり銘柄の合計どうし)".to_string(),
        "年間配当 = 1株配当 × 株数(1株配当は会社予想。なければ直近の実績)".to_string(),
        "取得額利回り = 年間配当 ÷ 取得総額。合計は全銘柄の年間配当の合計 ÷ 取得総額の合計".to_string(),
    ]
}

/// 「計算基準を見る」の中身(投資信託)
pub(crate) fn fund_basis_lines() -> Vec<String> {
    vec![
        "評価額 = 基準価額 × 保有口数 ÷ 10000".to_string(),
        "前日比 = 基準価額(前日比) × 保有口数 ÷ 10000 の合計(最新の基準価額の日が分かるファンドだけ。更新待ちのファンドは含まない)".to_string(),
        "評価損益 = 評価額 − 取得総額、率 = 評価損益 ÷ 取得総額".to_string(),
    ]
}

/// 「計算基準を見る」の中身(すべて)
pub(crate) fn all_basis_lines() -> Vec<String> {
    vec![
        "評価額 = 国内株式は 現在値 × 株数、投資信託は 基準価額 × 保有口数 ÷ 10000 の合計".to_string(),
        "前日比 = 国内株式は (現在値 − 前日終値) × 株数、投資信託は 基準価額(前日比) × 保有口数 ÷ 10000 の合計(更新待ちのファンドは含まない)".to_string(),
        "評価損益 = 評価額 − 取得総額、率 = 評価損益 ÷ 取得総額".to_string(),
    ]
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
    /// 構成比ページへの入口(国内株式のみ)
    composition_link: bool,
) -> impl IntoView {
    let market_text = format_valuation_amount(market);
    let profit_text = format_signed_valuation_amount(profit);
    let rate_text = profit_rate.map(|rate| format_signed_valuation_rate(Some(rate), 1));
    // 前日比は表・カードと同じく表示単位を適用せず必ず円マーク付きで出す
    let daily_text = format_signed_row_valuation_amount(daily_change);
    let profit_tone = if is_negative_valuation(profit) {
        "asset-summary-loss"
    } else if is_positive_valuation(profit) {
        "asset-summary-gain"
    } else {
        ""
    };
    let daily_tone = if is_negative_valuation(daily_change) {
        "asset-summary-loss"
    } else if is_positive_valuation(daily_change) {
        "asset-summary-gain"
    } else {
        ""
    };
    view! {
        <section
            id="assetbalance-summary"
            class="asset-summary asset-summary-band"
            role="region"
            aria-label="集計"
            data-testid="asset-portfolio-summary"
        >
            <div class="asset-summary-grid" data-testid="portfolio-kpi-grid">
                <dl class="asset-summary-main">
                    <div class="asset-summary-item asset-summary-item-primary">
                        <dt>"評価額合計"</dt>
                        <dd>
                            <Amount
                                text=market_text
                                class="asset-summary-primary-amount".to_string()
                            />
                        </dd>
                    </div>
                    <div class="asset-summary-item asset-summary-item-pnl">
                        <dt class="sr-only lg:not-sr-only">"評価損益"</dt>
                        <dd>
                            <Amount
                                text=profit_text
                                class=format!("asset-summary-pnl-amount {profit_tone}")
                            />
                            {rate_text.map(|rate| {
                                view! {
                                    <span class=format!("asset-summary-pnl-note {profit_tone}")>
                                        {rate}
                                    </span>
                                }
                            })}
                        </dd>
                    </div>
                    <div class="asset-summary-item asset-summary-item-sub3 asset-summary-item-daily">
                        <dt>"前日比"</dt>
                        <dd>
                            <Amount
                                text=daily_text
                                class=format!("asset-summary-sub-amount {daily_tone}")
                            />
                        </dd>
                    </div>
                </dl>
                <dl class="asset-summary-tags">
                    {tags
                        .into_iter()
                        .take(2)
                        .enumerate()
                        .map(|(index, tag)| {
                            let class = format!(
                                "asset-summary-item asset-summary-item-sub{} asset-summary-item-tag",
                                index + 1
                            );
                            view! {
                                <div class=class>
                                    <dt>{tag.label}</dt>
                                    <dd>
                                        <Amount
                                            text=tag.value
                                            class="asset-summary-sub-amount".to_string()
                                        />
                                    </dd>
                                </div>
                            }
                        })
                        .collect_view()}
                </dl>
            </div>
            {composition_link.then(|| {
                view! {
                    <a href="/assetbalance/composition" class="asset-summary-comp lg:hidden">
                        "構成比 ›"
                    </a>
                }
            })}
        </section>
    }
}

/// 帯の直下の注記行。注記の右端に「計算基準を見る」を置く
#[component]
pub(crate) fn AssetSummaryNote(
    /// 注記の文。空のときは注記の欄を出さない
    note: Vec<String>,
    /// 「計算基準を見る」で出す説明
    basis: Vec<String>,
) -> impl IntoView {
    view! {
        <div class="asset-band-note-row">
            <div class="asset-band-note">
                {note
                    .into_iter()
                    .filter(|line| !line.is_empty())
                    .map(|line| view! { <p>{line}</p> })
                    .collect_view()}
            </div>
            <details class="asset-basis">
                <summary>"計算基準を見る"</summary>
                <div class="asset-basis-body">
                    {basis
                        .into_iter()
                        .map(|line| view! { <p>{line}</p> })
                        .collect_view()}
                </div>
            </details>
        </div>
    }
}
