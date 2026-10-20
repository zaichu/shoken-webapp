use crate::api::dto::AssetBalanceSummary;
use crate::features::asset_balance::csv::{AssetBalanceRow, AssetBalanceRowData};
use crate::features::asset_balance::format::{
    dec_to_f64, format_amount, format_percentage_value, format_price_date,
    format_signed_row_valuation_amount, format_signed_valuation_amount,
    format_signed_valuation_rate, format_valuation_amount, is_negative_valuation,
    is_positive_valuation, price_date_representative,
};
use crate::features::asset_balance::fund::{fund_totals, fund_view};
use crate::features::asset_balance::holdings::{holding_view, stock_daily_change_total};
use crate::features::asset_balance::model::{
    KpiHolding, ValuationInput, calculate_portfolio_kpi, portfolio_valuation, summary_valuation,
};
use crate::features::asset_balance::store::FilteredPortfolio;
use crate::features::asset_balance::store::csv::{ResolvedAssetBalance, ResolvedFundBalance};
use crate::features::dividend_per_share::DividendMaps;
use crate::ui::amount::Amount;
use crate::ui::button::{Button, ButtonVariant};
use crate::ui::empty_state::EmptyState;
use leptos::prelude::*;
use rust_decimal::Decimal;

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

/// PC ヘッダー横の件数・基準日注記。取得上限超過で行が欠けるときは API 集計側の件数・
/// 価格なし判定を使い、集計欄と食い違わないようにする
pub(crate) fn list_subtitle(view: &ResolvedAssetBalance) -> String {
    let (count, has_unpriced, has_missing_daily, date_text) = stock_subtitle_parts(view);
    format!(
        "{count}銘柄 · {date_text}{}{}",
        unpriced_suffix(has_unpriced),
        missing_day_change_suffix(has_missing_daily)
    )
}

/// 投資信託タブの副題。前日比が「最新」で合計に入る件数と「更新待ち」の件数を出す
pub(crate) fn fund_subtitle(view: &ResolvedFundBalance) -> String {
    let (latest, pending) = fund_nav_counts(view);
    let freshness = fund_freshness_text(latest, pending);
    format!("{}件 ・ {freshness}", view.total)
}

/// 「すべて」タブの副題。株と投信の合計件数と、それぞれの基準日・前日比の状態を並べる
pub(crate) fn merged_subtitle(
    stock: Option<&ResolvedAssetBalance>,
    fund: Option<&ResolvedFundBalance>,
) -> Option<String> {
    if stock.is_none() && fund.is_none() {
        return None;
    }
    let count =
        stock.map(|view| view.total).unwrap_or(0) + fund.map(|view| view.total).unwrap_or(0);
    let mut parts = Vec::new();
    if let Some(view) = stock.filter(|view| !view.rows.is_empty()) {
        let (_, has_unpriced, has_missing_daily, date_text) = stock_subtitle_parts(view);
        parts.push(format!(
            "{date_text}{}{}",
            unpriced_suffix(has_unpriced),
            missing_day_change_suffix(has_missing_daily)
        ));
    }
    if let Some(view) = fund.filter(|view| !view.rows.is_empty()) {
        let (latest, pending) = fund_nav_counts(view);
        parts.push(fund_freshness_text(latest, pending));
    }
    if parts.is_empty() {
        Some(format!("{count}件"))
    } else {
        Some(format!("{count}件 · {}", parts.join(" · ")))
    }
}

/// 「前日比が最新」の件数(前日比を持ち合計に入る行)と「更新待ち」の件数。
/// 行が取り切れていないときは API 集計を優先する
pub(crate) fn fund_nav_counts(view: &ResolvedFundBalance) -> (usize, usize) {
    match &view.fallback_totals {
        Some(summary) => (
            summary.latest_nav_count as usize,
            summary.pending_nav_count as usize,
        ),
        None => {
            let views: Vec<_> = view.rows.iter().map(fund_view).collect();
            let totals = fund_totals(&views);
            (totals.latest_nav_count, totals.pending_nav_count)
        }
    }
}

fn fund_freshness_text(latest: usize, pending: usize) -> String {
    if pending == 0 {
        "前日比はすべて最新".to_string()
    } else {
        format!("前日比が最新 {latest}件 ・ 更新待ち {pending}件")
    }
}

/// 見出しの基準日と表の日付出し分けで共通に使う、価格あり行の基準日一覧
/// (価格が出ていない行の日付は「その時点の株価」と結びつかないので除く)
pub(crate) fn stock_price_dates(rows: &[AssetBalanceRow]) -> Vec<&str> {
    rows.iter()
        .filter(|row| row.current_price() > Decimal::ZERO)
        .filter_map(|row| row.price_as_of())
        .collect()
}

fn stock_subtitle_parts(view: &ResolvedAssetBalance) -> (usize, bool, bool, String) {
    let (count, has_unpriced, has_missing_daily) = match &view.fallback_totals {
        Some(summary) => (
            view.total,
            summary_valuation(summary).has_unpriced,
            summary.has_missing_daily_change,
        ),
        None => {
            let views: Vec<_> = view.rows.iter().map(holding_view).collect();
            let inputs: Vec<ValuationInput> = views.iter().map(ValuationInput::from).collect();
            let has_missing = views.iter().any(|view| view.day_change.is_none());
            (
                view.rows.len(),
                portfolio_valuation(&inputs).has_unpriced,
                has_missing,
            )
        }
    };
    let dates = stock_price_dates(&view.rows);
    let date_text = match price_date_representative(dates.iter().copied()) {
        Some(date) => format!("株価 {} 時点(銘柄ごとの最新終値)", format_price_date(date)),
        None => "株価の基準日は不明".to_string(),
    };
    (count, has_unpriced, has_missing_daily, date_text)
}

#[component]
pub(crate) fn PortfolioSummary(
    filtered: FilteredPortfolio,
    fallback_totals: Option<AssetBalanceSummary>,
    on_clear_filter: impl Fn() + Send + Sync + 'static,
    dividends: RwSignal<DividendMaps>,
) -> impl IntoView {
    let on_clear_filter = std::sync::Arc::new(on_clear_filter);
    let FilteredPortfolio { views, all_views } = filtered;
    let views_empty = views.is_empty();
    if all_views.is_empty() {
        return view! {
            <div class="asset-panel-padded mx-4 lg:mx-0">
                <div class="text-center">
                    <EmptyState
                        title="該当する銘柄がありません"
                        description="検索条件を変更するか、絞り込みを解除してください。"
                    />
                    <div class="mt-3 text-center">
                        <Button
                            variant=ButtonVariant::Ghost
                            on_click=move |_| on_clear_filter()
                        >
                            "絞り込みを解除"
                        </Button>
                    </div>
                </div>
            </div>
        }
        .into_any();
    }
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
        .as_ref()
        .map(summary_valuation)
        .unwrap_or(row_valuation);
    // 前日比の合計は API 集計(全件)を優先し、無ければ見えている行から計算する。
    // 前日終値なしの行は除く(全部なしなら「—」)
    let daily_change = fallback_totals
        .as_ref()
        .map(|summary| dec_to_f64(&summary.total_daily_change))
        .or_else(|| stock_daily_change_total(&all_views));
    let has_missing_daily = fallback_totals
        .as_ref()
        .map(|summary| summary.has_missing_daily_change)
        .unwrap_or_else(|| all_views.iter().any(|view| view.day_change.is_none()));
    let kpi =
        Memo::new(move |_| calculate_portfolio_kpi(&kpi_holdings, &dividends.get().per_share));

    let market_text = format_valuation_amount(valuation.market_value);
    let pnl_text = format_signed_valuation_amount(valuation.profit_loss.amount);
    let pnl_note = format_signed_valuation_rate(valuation.profit_loss.rate, 1);
    let has_pnl_rate = valuation.profit_loss.rate.is_some();
    let pnl_tone = if is_negative_valuation(valuation.profit_loss.amount) {
        "asset-summary-loss"
    } else if is_positive_valuation(valuation.profit_loss.amount) {
        "asset-summary-gain"
    } else {
        ""
    };
    let dividend_text = Signal::derive(move || {
        kpi.get()
            .total_annual_dividends
            .map_or("—".to_string(), format_amount)
    });
    let yield_text = Signal::derive(move || {
        kpi.get()
            .dividend_yield
            .map_or("—".to_string(), format_percentage_value)
    });
    // 前日比は表・カードと同じく表示単位を適用せず必ず円マーク付きで出す
    let daily_text = format_signed_row_valuation_amount(daily_change);
    let daily_tone = if is_negative_valuation(daily_change) {
        "asset-summary-loss"
    } else if is_positive_valuation(daily_change) {
        "asset-summary-gain"
    } else {
        ""
    };
    let asof_text = {
        let dates = all_views
            .iter()
            .filter(|view| view.current_price > 0.0)
            .filter_map(|view| view.price_as_of.as_deref());
        let base = match price_date_representative(dates) {
            Some(date) => format!("株価 {} 時点(銘柄ごと)", format_price_date(date)),
            None => "株価の基準日は不明".to_string(),
        };
        format!(
            "{base}{}{}",
            unpriced_suffix(valuation.has_unpriced),
            missing_day_change_suffix(has_missing_daily)
        )
    };

    view! {
        <section
            id="assetbalance-summary"
            class="asset-summary asset-summary-5 asset-summary-band"
            role="region"
            aria-label="集計"
            data-testid="asset-portfolio-summary"
        >
            <p class="asset-summary-asof lg:hidden">{asof_text}</p>
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
                        <dt class="sr-only lg:not-sr-only">"含み損益"</dt>
                        <dd>
                            <Amount
                                text=pnl_text
                                class=format!("asset-summary-pnl-amount {pnl_tone}")
                            />
                            {has_pnl_rate.then(move || {
                                view! {
                                    <span class=format!("asset-summary-pnl-note {pnl_tone}")>
                                        {pnl_note}
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
                    <div class="asset-summary-item asset-summary-item-sub1 asset-summary-item-tag">
                        <dt>"年間配当(予想)"</dt>
                        <dd>
                            <Amount text=dividend_text class="asset-summary-sub-amount".to_string() />
                        </dd>
                    </div>
                    <div class="asset-summary-item asset-summary-item-sub2 asset-summary-item-tag">
                        <dt>"取得額利回り"</dt>
                        <dd>
                            <Amount text=yield_text class="asset-summary-sub-amount".to_string() />
                        </dd>
                    </div>
                </dl>
            </div>
            <a href="/assetbalance/composition" class="asset-summary-comp lg:hidden">
                "構成比 ›"
            </a>
            <details class="asset-summary-basis lg:hidden">
                <summary>"計算基準"</summary>
                <div class="asset-summary-basis-body">
                    "評価額 = 現在値 × 株数(価格なし銘柄は除く。株価の日付は銘柄ごとに違う)"
                    <br />
                    "前日比 = (現在値 − 前日終値) × 株数の合計(前日終値なし銘柄は除く)"
                    <br />
                    "年間配当 = 1株配当 × 株数(会社予想。なければ直近の実績)"
                    <br />
                    "取得額利回り = 年間配当 ÷ 取得総額"
                </div>
            </details>
            <details class="asset-basis-pc hidden lg:block">
                <summary>"計算基準を見る"</summary>
                <div class="asset-basis-pc-body">
                    "評価額 = 現在値 × 株数(価格なし銘柄は除く。現在値は銘柄ごとの最新終値で、日付は銘柄ごとに違い、他と違う行は表の「前日比」の横に表示)"
                    <br />
                    "前日比 = (現在値 − 前日終値) × 株数の合計(前日終値なし銘柄は除く。1株あたりの前日比は表の「前日比」に表示)"
                    <br />
                    "含み損益 = 評価額 − 取得総額、率 = 含み損益 ÷ 取得総額(価格あり銘柄の合計どうし)"
                    <br />
                    "年間配当 = 1株配当 × 株数(1株配当は会社予想。なければ直近の実績)"
                    <br />
                    "取得額利回り = 年間配当 ÷ 取得総額。合計は全銘柄の年間配当の合計 ÷ 取得総額の合計"
                </div>
            </details>
            {views_empty.then(move || {
                view! {
                    <div class="asset-panel-padded col-span-full mx-4 mt-3 lg:mx-0">
                        <div class="text-center">
                            <EmptyState
                                title="該当する銘柄がありません"
                                description="検索条件を変更するか、絞り込みを解除してください。"
                            />
                            <div class="mt-3 text-center">
                                <Button
                                    variant=ButtonVariant::Ghost
                                    on_click=move |_| on_clear_filter()
                                >
                                    "絞り込みを解除"
                                </Button>
                            </div>
                        </div>
                    </div>
                }
            })}
        </section>
    }
    .into_any()
}
