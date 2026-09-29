use super::chart::ChartList;
use super::palette::holding_bar_class;
use crate::api::dto::AssetBalanceSummary;
use crate::features::asset_balance::format::{
    dec_to_f64, format_currency, format_fixed_percent, format_percentage_value,
    format_valuation_amount, format_valuation_rate, valuation_tone,
};
use crate::features::asset_balance::holdings::HoldingView;
use crate::features::asset_balance::model::{
    calculate_portfolio_kpi, summarize_valuation_with_summary, total_purchase_amount, KpiHolding,
    SummaryOverride, ValuationItem, ValuationSummary,
};
use crate::features::asset_balance::portfolio::{chart_display, chart_plan};
use crate::features::dividend_per_share::DividendMaps;
use crate::ui::amount::Amount;
use crate::ui::badge::{Badge, BadgeVariant};
use crate::ui::button::{Button, ButtonVariant};
use crate::ui::card::{Card, CardVariant, SectionHeader, SectionHeaderVariant};
use crate::ui::choice::{Chip, ChipVariant};
use crate::ui::empty_state::EmptyState;
use leptos::prelude::*;

#[derive(Clone, Debug)]
pub(crate) struct ChartItem {
    pub(crate) view: HoldingView,
    pub(crate) percentage: Option<f64>,
}

pub(crate) fn valuation_summary_override(
    views: &[HoldingView],
    summary: Option<&AssetBalanceSummary>,
) -> Option<SummaryOverride> {
    summary
        .map(|summary| SummaryOverride {
            total_purchase_amount: summary.total_purchase_amount,
            total_market_value: summary.total_market_value,
        })
        .or_else(|| {
            SummaryOverride::sum(
                views
                    .iter()
                    .map(|view| (view.market_dec, view.purchase_dec)),
            )
        })
}

pub(crate) fn portfolio_valuation(
    views: &[HoldingView],
    summary: Option<&AssetBalanceSummary>,
) -> ValuationSummary {
    let valuation_items: Vec<ValuationItem> = views
        .iter()
        .map(|view| ValuationItem {
            market_value: Some(view.market),
            total_purchase_amount: Some(view.purchase),
        })
        .collect();
    summarize_valuation_with_summary(
        &valuation_items,
        valuation_summary_override(views, summary).as_ref(),
    )
}

#[component]
pub(crate) fn PortfolioSummary(
    views: Vec<HoldingView>,
    total_count: usize,
    is_filtered: bool,
    on_clear_filter: impl Fn() + Send + Sync + 'static,
    summary: Option<AssetBalanceSummary>,
    dividends: RwSignal<DividendMaps>,
    show_all: RwSignal<bool>,
) -> impl IntoView {
    if views.is_empty() {
        show_all.set(false);
        return view! {
            <Card variant=CardVariant::Soft class="mb-3">
                <div>
                    <EmptyState
                        title="該当する銘柄がありません"
                        description="検索条件を変更するか、絞り込みを解除してください。"
                    />
                    <div class="mt-3 text-center">
                        <Button variant=ButtonVariant::Ghost on_click=move |_| on_clear_filter()>
                            "絞り込みを解除"
                        </Button>
                    </div>
                </div>
            </Card>
        }
        .into_any();
    }
    let valuation = portfolio_valuation(&views, summary.as_ref());
    let (valuation_class, valuation_negative) = valuation_tone(valuation.amount);
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
    let composition_items = chart_items.clone();
    let composition_percentages: Vec<Option<f64>> = composition_items
        .iter()
        .map(|item| item.percentage)
        .collect();
    let composition_total = composition_items.len();
    let has_composition = composition_items
        .iter()
        .any(|item| item.percentage.is_some());
    let composition = has_composition.then(move || {
        view! {
            <div class="mt-4 border-t border-ink/10 pt-4" data-testid="portfolio-composition">
                <p class="mb-2 text-xs font-medium text-text-muted">"構成比"</p>
                {move || {
                    let display = chart_display(
                        composition_total,
                        &composition_percentages,
                        show_all.get(),
                    );
                    view! {
                        <div
                            class="flex h-3 overflow-hidden rounded-full bg-fill"
                            aria-hidden="true"
                        >
                            {composition_items
                                .iter()
                                .take(display.visible_count)
                                .enumerate()
                                .filter_map(|(index, item)| {
                                    item.percentage.map(|percentage| {
                                        view! {
                                            <div
                                                class=format!("h-full {}", holding_bar_class(index))
                                                style=format!("width: {}%", percentage.min(100.0))
                                            />
                                        }
                                    })
                                })
                                .collect_view()}
                            {display.others.as_ref().map(|others| {
                                view! {
                                    <div
                                        class="h-full bg-fill-strong"
                                        style=format!("width: {}%", others.percentage.min(100.0))
                                    />
                                }
                            })}
                        </div>
                        <ul class="mt-2 flex flex-wrap gap-x-4 gap-y-1 text-xs">
                            {composition_items
                                .iter()
                                .take(display.visible_count)
                                .enumerate()
                                .map(|(index, item)| {
                                    view! {
                                        <li class="flex min-w-0 items-center gap-1.5">
                                            <span
                                                class=format!("h-2.5 w-2.5 shrink-0 rounded-sm {}", holding_bar_class(index))
                                                aria-hidden="true"
                                            />
                                            <span class="text-text">{item.view.name.clone()}</span>
                                            <span class="shrink-0 tabular-nums text-text-subtle">
                                                {item.percentage.map_or("—".to_string(), |percentage| {
                                                    format_fixed_percent(percentage, 1)
                                                })}
                                            </span>
                                        </li>
                                    }
                                })
                                .collect_view()}
                            {display.others.as_ref().map(|others| {
                                view! {
                                    <li class="flex min-w-0 items-center gap-1.5">
                                        <span
                                            class="h-2.5 w-2.5 shrink-0 rounded-sm bg-fill-strong"
                                            aria-hidden="true"
                                        />
                                        <span class="text-text">
                                            {format!("その他 {}銘柄", others.count)}
                                        </span>
                                        <span class="shrink-0 tabular-nums text-text-subtle">
                                            {format_fixed_percent(others.percentage, 1)}
                                        </span>
                                    </li>
                                }
                            })}
                        </ul>
                    }
                }}
            </div>
        }
        .into_any()
    });

    let market_value = valuation.market_value;
    if total_purchase_amount == 0.0 && matches!(market_value, None | Some(0.0)) {
        show_all.set(false);
        return ().into_any();
    }

    view! {
        <div class="mb-3 space-y-4" data-testid="asset-portfolio-summary">
            <Card variant=CardVariant::Summary testid="portfolio-kpi-strip">
                <SectionHeader
                    variant=SectionHeaderVariant::Card
                    trailing=view! {
                        {is_filtered.then(|| {
                            view! {
                                <div class="flex flex-wrap items-center gap-2">
                                    <Badge variant=BadgeVariant::Info>
                                        {format!("絞り込み中: {display_count}/{total_count}件")}
                                    </Badge>
                                    <Chip
                                        variant=ChipVariant::Pill
                                        on_click=move |_| on_clear_filter()
                                    >
                                        "解除"
                                    </Chip>
                                </div>
                            }
                        })}
                    }
                    .into_any()
                >
                    "資産サマリー"
                </SectionHeader>
                <div class="mt-4" data-testid="portfolio-valuation-summary">
                    <p class="text-sm font-medium text-text-muted">"保有資産の評価額"</p>
                    <Amount
                        block=true
                        text=market_value.map_or("—".to_string(), format_currency)
                        class="mt-1 text-3xl font-black text-ink"
                    />
                    <p
                        class=format!("mt-2 text-sm font-bold {valuation_class}")
                        data-negative=valuation_negative
                    >
                        "評価損益 "
                        <Amount
                            text=match valuation.amount {
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
                            }
                        />
                    </p>
                    <p class="mt-1 text-xs text-text-subtle">"取込データ時点"</p>
                    {valuation
                        .incomplete
                        .then(|| {
                            view! {
                                <p class="mt-1 text-xs text-accent-deep">
                                    "一部の銘柄の評価額が不足しているため、合計を算出できません"
                                </p>
                            }
                        })}
                </div>
                <div
                    class="mt-4 grid grid-cols-2 gap-2 sm:gap-3 xl:grid-cols-4"
                    data-testid="portfolio-kpi-grid"
                >
                    <Card variant=CardVariant::Stat>
                        <p class="mb-1 text-xs font-medium text-text-muted">"合計取得総額"</p>
                        <Amount
                            block=true
                            text=format_currency(total_purchase_amount)
                            class="whitespace-nowrap text-base font-bold sm:text-3xl xl:text-2xl text-text-deep"
                        />
                    </Card>
                    <Card variant=CardVariant::Stat>
                        <p class="mb-1 text-xs font-medium text-text-muted">"年間配当金額"</p>
                        <Amount
                            block=true
                            text=Signal::derive(move || {
                                kpi.with(|kpi| {
                                    kpi.total_annual_dividends
                                        .map_or("—".to_string(), format_currency)
                                })
                            })
                            class="whitespace-nowrap text-base font-bold sm:text-3xl xl:text-2xl text-ink"
                            testid="portfolio-annual-dividends"
                        />
                    </Card>
                    <Card variant=CardVariant::Stat>
                        <p class="mb-1 text-xs font-medium text-text-muted">"配当利回り"</p>
                        <Amount
                            block=true
                            text=Signal::derive(move || {
                                kpi.with(|kpi| {
                                    kpi.dividend_yield
                                        .map_or("—".to_string(), format_percentage_value)
                                })
                            })
                            class="whitespace-nowrap text-base font-bold sm:text-3xl xl:text-2xl text-ink"
                            testid="portfolio-dividend-yield"
                        />
                    </Card>
                    <Card variant=CardVariant::Stat>
                        <p class="mb-1 text-xs font-medium text-text-muted">"保有銘柄数"</p>
                        <p class="whitespace-nowrap text-base font-bold sm:text-3xl xl:text-2xl text-text-soft tabular-nums">
                            {if is_filtered {
                                format!("{display_count} / {total_count}")
                            } else {
                                display_count.to_string()
                            }}
                            <span class="ml-1 text-sm font-normal text-text-subtle">"銘柄"</span>
                        </p>
                    </Card>
                </div>
                {composition}
            </Card>

            <div data-testid="portfolio-pie-chart">
                <SectionHeader variant=SectionHeaderVariant::Band>
                    "保有内訳"
                    <span class="ml-1 font-medium text-text-subtle sm:hidden">
                        {format!("（保有{display_count}銘柄）")}
                    </span>
                </SectionHeader>
                <div class="p-4">
                    <ChartList items=chart_items dividends=dividends show_all=show_all />
                </div>
            </div>
        </div>
    }
        .into_any()
}
