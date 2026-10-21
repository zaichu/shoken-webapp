use crate::api::dto::AssetBalanceSummary;
use crate::features::asset_balance::csv::{AssetBalanceRow, AssetBalanceRowData};
use crate::features::asset_balance::format::{
    dec_to_f64, format_amount, format_nav_updated_at, format_percentage_value, format_price_date,
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

    view! {
        <section
            id="assetbalance-summary"
            class="asset-summary asset-summary-5 asset-summary-band"
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
