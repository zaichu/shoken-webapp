use super::chart::ChartList;
use super::palette::holding_bar_class;
use crate::api::dto::AssetBalanceSummary;
use crate::features::asset_balance::format::{
    dec_to_f64, format_currency, format_fixed_percent, format_percentage_value,
};
use crate::features::asset_balance::holdings::HoldingView;
use crate::features::asset_balance::model::{
    calculate_portfolio_kpi, total_purchase_amount, KpiHolding,
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
    let plan = chart_plan(&chart_values);
    let chart_items: Vec<ChartItem> = plan
        .order
        .iter()
        .zip(&plan.percentages)
        .map(|(&index, &percentage)| ChartItem {
            view: views[index].clone(),
            percentage,
        })
        .collect();
    // 帯・凡例は構成比を持つ行だけ。取得額0の行はカードには出すが帯には置かない
    let composition_items: Vec<ChartItem> = chart_items
        .iter()
        .filter(|item| item.percentage.is_some())
        .cloned()
        .collect();
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
