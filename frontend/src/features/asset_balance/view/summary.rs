use crate::api::dto::AssetBalanceSummary;
use crate::features::asset_balance::csv::AssetBalanceRowData;
use crate::features::asset_balance::format::{
    format_currency, format_percentage_value, format_signed_valuation_amount,
    format_signed_valuation_rate, format_valuation_amount, is_negative_valuation,
    is_positive_valuation, price_date_range,
};
use crate::features::asset_balance::holdings::holding_view;
use crate::features::asset_balance::model::{
    KpiHolding, ValuationInput, calculate_portfolio_kpi, portfolio_valuation, summary_valuation,
};
use crate::features::asset_balance::store::FilteredPortfolio;
use crate::features::asset_balance::store::csv::ResolvedAssetBalance;
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

/// PC ヘッダー横の件数・基準日注記。取得上限超過で行が欠けるときは API 集計側の件数・
/// 価格なし判定を使い、集計欄と食い違わないようにする
pub(crate) fn list_subtitle(view: &ResolvedAssetBalance) -> String {
    let (count, has_unpriced) = match &view.fallback_totals {
        Some(summary) => (view.total, summary_valuation(summary).has_unpriced),
        None => {
            let inputs: Vec<ValuationInput> = view
                .rows
                .iter()
                .map(|row| ValuationInput::from(&holding_view(row)))
                .collect();
            (view.rows.len(), portfolio_valuation(&inputs).has_unpriced)
        }
    };
    let dates = view
        .rows
        .iter()
        .filter(|row| row.current_price() > Decimal::ZERO)
        .filter_map(|row| row.price_as_of());
    let date_text = match price_date_range(dates) {
        Some((min, max)) if min == max => format!("株価 {min} 時点(銘柄ごとの最新終値)"),
        Some((min, max)) => format!("株価 {min}〜{max} 時点(銘柄ごとの最新終値)"),
        None => "株価の基準日は不明".to_string(),
    };
    format!("{count}銘柄 · {date_text}{}", unpriced_suffix(has_unpriced))
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
            .map_or("—".to_string(), format_currency)
    });
    let yield_text = Signal::derive(move || {
        kpi.get()
            .dividend_yield
            .map_or("—".to_string(), format_percentage_value)
    });
    let asof_text = {
        let dates = all_views
            .iter()
            .filter(|view| view.current_price > 0.0)
            .filter_map(|view| view.price_as_of.as_deref());
        let base = match price_date_range(dates) {
            Some((min, max)) if min == max => format!("株価 {min} 時点(銘柄ごと)"),
            Some((min, max)) => format!("株価 {min}〜{max} 時点(銘柄ごと)"),
            None => "株価の基準日は不明".to_string(),
        };
        format!("{base}{}", unpriced_suffix(valuation.has_unpriced))
    };

    view! {
        <section
            id="assetbalance-summary"
            class="asset-summary"
            role="region"
            aria-label="集計"
            data-testid="asset-portfolio-summary"
        >
            <p class="asset-summary-asof lg:hidden">{asof_text}</p>
            <dl class="asset-summary-grid" data-testid="portfolio-kpi-grid">
                <div class="asset-summary-item asset-summary-item-primary">
                    <dt>"評価額合計"</dt>
                    <dd>
                        <Amount text=market_text class="asset-summary-primary-amount".to_string() />
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
                <div class="asset-summary-item asset-summary-item-sub1">
                    <dt>"年間配当(予想)"</dt>
                    <dd>
                        <Amount text=dividend_text class="asset-summary-sub-amount".to_string() />
                    </dd>
                </div>
                <div class="asset-summary-item asset-summary-item-sub2">
                    <dt>"取得額利回り"</dt>
                    <dd>
                        <Amount text=yield_text class="asset-summary-sub-amount".to_string() />
                    </dd>
                </div>
            </dl>
            <a href="/assetbalance/composition" class="asset-summary-comp lg:hidden">
                "構成比 ›"
            </a>
            <details class="asset-summary-basis lg:hidden">
                <summary>"計算基準"</summary>
                <div class="asset-summary-basis-body">
                    "評価額 = 現在値 × 株数(価格なし銘柄は除く。株価の日付は銘柄ごとに違う)"
                    <br />
                    "年間配当 = 1株配当 × 株数(会社予想。なければ直近の実績)"
                    <br />
                    "取得額利回り = 年間配当 ÷ 取得総額"
                </div>
            </details>
            <details class="asset-basis-pc hidden lg:block">
                <summary>"計算基準を見る"</summary>
                <div class="asset-basis-pc-body">
                    "評価額 = 現在値 × 株数(価格なし銘柄は除く。現在値は銘柄ごとの最新終値で、日付は銘柄ごとに違い、表の「現在値」の下に表示)"
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
