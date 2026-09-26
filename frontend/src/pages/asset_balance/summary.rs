use super::chart::ChartList;
use super::format::{
    dec_to_f64, format_currency, format_percentage_value, format_valuation_amount,
    format_valuation_rate, is_negative_valuation,
};
use super::holdings::HoldingView;
use crate::asset_balance::portfolio::chart_plan;
use crate::asset_balance_domain::{
    calculate_portfolio_kpi, summarize_valuation_with_summary, total_purchase_amount, KpiHolding,
    SummaryOverride, ValuationItem,
};
use crate::dividend_per_share::DividendMaps;
use crate::dto::AssetBalanceSummary;
use leptos::prelude::*;

#[derive(Clone, Debug)]
pub(crate) struct ChartItem {
    pub(crate) view: HoldingView,
    pub(crate) percentage: Option<f64>,
}

#[component]
pub(crate) fn PortfolioSummary(
    views: Vec<HoldingView>,
    total_count: usize,
    is_filtered: bool,
    on_clear_filter: impl Fn() + 'static,
    summary: Option<AssetBalanceSummary>,
    dividends: RwSignal<DividendMaps>,
    show_all: RwSignal<bool>,
) -> impl IntoView {
    if views.is_empty() {
        show_all.set(false);
        return view! {
            <div class="mb-3 overflow-hidden rounded-xl border border-slate-950/10 bg-white/90 shadow-[0_14px_38px_-32px_rgba(15,23,42,0.85)]">
                <div>
                    <div class="empty-state">
                        <h3 class="text-base font-black text-slate-950">
                            "該当する銘柄がありません"
                        </h3>
                        <p class="mt-1.5 max-w-md text-sm font-medium text-slate-600">
                            "検索条件を変更するか、絞り込みを解除してください。"
                        </p>
                    </div>
                    <div class="mt-3 text-center">
                        <button
                            type="button"
                            class="text-sm text-primary hover:underline"
                            on:click=move |_| on_clear_filter()
                        >
                            "絞り込みを解除"
                        </button>
                    </div>
                </div>
            </div>
        }
        .into_any();
    }
    let valuation_items: Vec<ValuationItem> = views
        .iter()
        .map(|view| ValuationItem {
            market_value: serde_json::json!(view.market),
            total_purchase_amount: serde_json::json!(view.purchase),
        })
        .collect();
    let summary_override = summary.as_ref().map(|summary| SummaryOverride {
        total_purchase_amount: summary.total_purchase_amount,
        total_market_value: summary.total_market_value,
    });
    let valuation = summarize_valuation_with_summary(&valuation_items, summary_override.as_ref());
    let kpi_holdings: Vec<KpiHolding> = views
        .iter()
        .map(|view| KpiHolding {
            security_code: view.code.clone(),
            shares: view.shares,
            total_purchase_amount: view.purchase,
        })
        .collect();
    let summary_total = summary
        .as_ref()
        .map(|summary| dec_to_f64(&summary.total_purchase_amount));
    let total_purchase_amount = total_purchase_amount(&kpi_holdings, summary_total);
    let kpi = Memo::new(move |_| {
        calculate_portfolio_kpi(&kpi_holdings, &dividends.get().per_share, summary_total)
    });

    let display_count = views.len();
    let chart_values: Vec<f64> = views.iter().map(|view| view.purchase).collect();
    let chart_markets: Vec<Option<f64>> = views.iter().map(|view| Some(view.market)).collect();
    let plan = chart_plan(&chart_values, &chart_markets);
    let chart_items: Vec<ChartItem> = plan
        .order
        .iter()
        .zip(&plan.percentages)
        .map(|(&index, &percentage)| ChartItem {
            view: views[index].clone(),
            percentage,
        })
        .collect();

    let market_value = valuation.market_value;
    if total_purchase_amount == 0.0 && matches!(market_value, None | Some(0.0)) {
        show_all.set(false);
        return ().into_any();
    }

    view! {
        <div class="mb-3 space-y-4" data-testid="asset-portfolio-summary">
            <section
                class="rounded-xl border border-slate-950/10 bg-white/95 px-5 py-5 shadow-[0_16px_44px_-38px_rgba(15,23,42,0.9)]"
                data-testid="portfolio-kpi-strip"
            >
                <div class="flex flex-col gap-3 border-b border-slate-950/10 pb-4 sm:flex-row sm:items-end sm:justify-between">
                    <div>
                        <h2 class="text-sm font-black text-slate-950">"資産サマリー"</h2>
                    </div>
                    {is_filtered.then(move || {
                        view! {
                            <div class="flex flex-wrap items-center gap-2">
                                <span class="inline-flex items-center rounded-md border border-blue-200 bg-blue-50 px-3 py-1 text-sm font-bold text-blue-700">
                                    {format!("絞り込み中: {display_count}/{total_count}件")}
                                </span>
                                <button
                                    type="button"
                                    class="filter-chip"
                                    on:click=move |_| on_clear_filter()
                                >
                                    "解除"
                                </button>
                            </div>
                        }
                    })}
                </div>
                <div class="mt-4" data-testid="portfolio-valuation-summary">
                    <p class="text-sm font-medium text-slate-600">"保有資産の評価額"</p>
                    <p class="mt-1 text-3xl font-black tabular-nums text-slate-950">
                        {market_value.map_or("—".to_string(), format_currency)}
                    </p>
                    <p
                        class=format!(
                            "mt-2 text-sm font-bold tabular-nums {}",
                            if is_negative_valuation(valuation.amount) {
                                "text-red-800"
                            } else {
                                "text-slate-800"
                            },
                        )
                        data-negative=is_negative_valuation(valuation.amount).then_some("true")
                    >
                        "評価損益 "
                        {match valuation.amount {
                            None => "—".to_string(),
                            Some(amount) => {
                                match valuation.rate {
                                    None => {
                                        format!(
                                            "{}（算出不可）",
                                            format_valuation_amount(Some(amount)),
                                        )
                                    }
                                    Some(rate) => {
                                        format!(
                                            "{}（{}）",
                                            format_valuation_amount(Some(amount)),
                                            format_valuation_rate(Some(rate), 1),
                                        )
                                    }
                                }
                            }
                        }}
                    </p>
                    <p class="mt-1 text-xs text-slate-500">"取込データ時点"</p>
                    {valuation
                        .incomplete
                        .then(|| {
                            view! {
                                <p class="mt-1 text-xs text-amber-700">
                                    "一部の銘柄の評価額が不足しているため、合計を算出できません"
                                </p>
                            }
                        })}
                </div>
                <div
                    class="mt-4 grid grid-cols-2 gap-2 max-sm:hidden sm:gap-3 lg:grid-cols-4"
                    data-testid="portfolio-kpi-grid"
                >
                    <div class="rounded-lg border border-slate-950/10 bg-white px-4 py-4 shadow-sm">
                        <p class="mb-1 text-xs font-medium text-slate-600">"合計取得総額"</p>
                        <p
                            class="text-base font-bold sm:text-3xl text-primary tabular-nums"
                            data-negative=move || {
                                if total_purchase_amount < 0.0 { Some("true") } else { None }
                            }
                        >
                            {format_currency(total_purchase_amount)}
                        </p>
                    </div>
                    <div class="rounded-lg border border-teal-200 bg-teal-50 px-4 py-4 shadow-sm">
                        <p class="mb-1 text-xs font-medium text-slate-600">"年間配当金額"</p>
                        <p
                            class="text-base font-bold sm:text-3xl text-teal-700 tabular-nums"
                            data-testid="portfolio-annual-dividends"
                        >
                            {move || {
                                kpi.with(|kpi| {
                                    kpi.total_annual_dividends
                                        .map_or("---".to_string(), format_currency)
                                })
                            }}
                        </p>
                    </div>
                    <div class="rounded-lg border border-teal-200 bg-teal-50 px-4 py-4 shadow-sm">
                        <p class="mb-1 text-xs font-medium text-slate-600">"配当利回り"</p>
                        <p
                            class="text-base font-bold sm:text-3xl text-teal-700 tabular-nums"
                            data-testid="portfolio-dividend-yield"
                        >
                            {move || {
                                kpi.with(|kpi| {
                                    kpi.dividend_yield
                                        .map_or("---".to_string(), format_percentage_value)
                                })
                            }}
                        </p>
                    </div>
                    <div class="rounded-lg border border-slate-950/10 bg-white px-4 py-4 shadow-sm">
                        <p class="mb-1 text-xs font-medium text-slate-600">"保有銘柄数"</p>
                        <p class="text-base font-bold sm:text-3xl text-slate-700 tabular-nums">
                            {if is_filtered {
                                format!("{display_count} / {total_count}")
                            } else {
                                display_count.to_string()
                            }}
                            <span class="ml-1 text-sm font-normal text-slate-500">"銘柄"</span>
                        </p>
                    </div>
                </div>
            </section>

            <div data-testid="portfolio-pie-chart">
                <div class="summary-section-header">
                    <div>
                        <h3 class="text-sm font-black text-slate-900">
                            "保有内訳"
                            <span class="ml-1 font-medium text-slate-500 sm:hidden">
                                {format!("（保有{display_count}銘柄）")}
                            </span>
                        </h3>
                    </div>
                </div>
                <div class="p-4">
                    <ChartList items=chart_items dividends=dividends show_all=show_all />
                </div>
            </div>
        </div>
    }
        .into_any()
}
