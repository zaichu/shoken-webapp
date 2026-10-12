use crate::api::dto::AssetBalanceSummary;
use crate::features::asset_balance::format::{
    format_currency, format_percentage_value, format_signed_valuation_amount,
    format_signed_valuation_rate, format_valuation_amount, is_negative_valuation,
};
use crate::features::asset_balance::model::{
    KpiHolding, ValuationInput, calculate_portfolio_kpi, portfolio_valuation, summary_valuation,
};
use crate::features::asset_balance::store::FilteredPortfolio;
use crate::features::dividend_per_share::DividendMaps;
use crate::ui::button::{Button, ButtonVariant};
use crate::ui::empty_state::EmptyState;
use crate::ui::summary_strip::{SummaryItem, SummaryStrip};
use leptos::prelude::*;

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
            <div class="rounded-lg border border-border bg-surface p-6">
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
    let mut market_note = "現在値の基準日は銘柄ごとに異なります".to_string();
    if valuation.has_unpriced {
        market_note.push_str("・価格なし銘柄を除く");
    }
    let pnl_text = format_signed_valuation_amount(valuation.profit_loss.amount);
    let pnl_note = format_signed_valuation_rate(valuation.profit_loss.rate, 1);
    let pnl_negative = is_negative_valuation(valuation.profit_loss.amount);

    let items = Signal::derive(move || {
        let kpi = kpi.get();
        vec![
            SummaryItem::new("評価額合計", Some(market_text.clone())),
            SummaryItem {
                label: "含み損益",
                value: Some(pnl_text.clone()),
                note: Some(pnl_note.clone()),
                negative: pnl_negative,
            },
            SummaryItem::new(
                "年間配当参考額",
                Some(
                    kpi.total_annual_dividends
                        .map_or("—".to_string(), format_currency),
                ),
            ),
            SummaryItem::new(
                "取得額基準利回り",
                Some(
                    kpi.dividend_yield
                        .map_or("—".to_string(), format_percentage_value),
                ),
            ),
        ]
    });

    view! {
        <section id="assetbalance-summary" class="mb-3" data-testid="asset-portfolio-summary">
            <div data-testid="portfolio-kpi-strip">
                <div data-testid="portfolio-kpi-grid">
                    <SummaryStrip items=items />
                </div>
                <p class="mt-1 text-xs font-medium text-text-muted">{market_note}</p>
            </div>
            {views_empty.then(move || {
                view! {
                    <div class="rounded-lg border border-border bg-surface p-6">
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
