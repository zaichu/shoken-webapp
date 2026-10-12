use crate::api::dto::AssetBalanceSummary;
use crate::features::asset_balance::format::{
    format_currency, format_percentage_value, format_signed_valuation_amount,
    format_signed_valuation_rate, format_valuation_amount, is_negative_valuation,
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

pub(crate) fn market_note(has_unpriced: bool) -> String {
    let mut note = "現在値の基準日は銘柄ごとに異なります".to_string();
    if has_unpriced {
        note.push_str("・価格なし銘柄を除く");
    }
    note
}

/// PC ヘッダー横の件数・注記。取得上限超過で行が欠けるときは API 集計側の件数・
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
    format!("{count}銘柄 · {}", market_note(has_unpriced))
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
    let market_note = market_note(valuation.has_unpriced);
    let pnl_text = format_signed_valuation_amount(valuation.profit_loss.amount);
    let pnl_note = format_signed_valuation_rate(valuation.profit_loss.rate, 1);
    let pnl_negative = is_negative_valuation(valuation.profit_loss.amount);
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

    view! {
        <section id="assetbalance-summary" class="lg:mb-3.5" data-testid="asset-portfolio-summary">
            <div class="asset-summary" data-testid="portfolio-kpi-strip">
                <dl class="asset-summary-grid" data-testid="portfolio-kpi-grid">
                    <div class="asset-summary-item">
                        <dt>"評価額合計"</dt>
                        <dd>
                            <Amount text=market_text class="asset-summary-amount".to_string() />
                        </dd>
                    </div>
                    <div class="asset-summary-item">
                        <dt>"含み損益"</dt>
                        <dd
                            class="flex flex-wrap items-baseline gap-x-1.5"
                            data-negative=pnl_negative.then_some("true")
                        >
                            <Amount
                                text=pnl_text
                                negative=pnl_negative
                                class="asset-summary-amount".to_string()
                            />
                            <span class="asset-summary-note">{format!("（{pnl_note}）")}</span>
                        </dd>
                    </div>
                    <div class="asset-summary-item">
                        <dt>"年間配当参考額"</dt>
                        <dd>
                            <Amount text=dividend_text class="asset-summary-amount-sm".to_string() />
                        </dd>
                    </div>
                    <div class="asset-summary-item">
                        <dt>"取得額基準利回り"</dt>
                        <dd class="flex items-end justify-between gap-2">
                            <Amount text=yield_text class="asset-summary-amount-sm".to_string() />
                            <a
                                href="/assetbalance/composition"
                                class="-my-2 -mr-2 inline-flex min-h-11 shrink-0 items-center px-2 text-xs text-info no-underline hover:underline lg:hidden"
                            >
                                "構成比 ›"
                            </a>
                        </dd>
                    </div>
                </dl>
                <p class="mt-1 text-xs font-medium text-text-muted lg:hidden">{market_note}</p>
            </div>
            {views_empty.then(move || {
                view! {
                    <div class="asset-panel-padded mx-4 mt-3 lg:mx-0">
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
