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
            <div class="sum-card mb-3">
                <div class="sum-card-body">
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
            </div>
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
            <div class="sum-comp" data-testid="portfolio-composition">
                <p class="sum-comp-title">"構成比"</p>
                {move || {
                    let display = chart_display(
                        composition_total,
                        &composition_percentages,
                        show_all.get(),
                    );
                    view! {
                        <div
                            class="sum-comp-bar"
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
                                                class=format!("sum-comp-segment {}", holding_bar_class(index))
                                                style=format!("width: {}%", percentage.min(100.0))
                                            />
                                        }
                                    })
                                })
                                .collect_view()}
                            {display.others.as_ref().map(|others| {
                                view! {
                                    <div
                                        class="sum-comp-segment sum-comp-others"
                                        style=format!("width: {}%", others.percentage.min(100.0))
                                    />
                                }
                            })}
                        </div>
                        <ul class="sum-comp-legend">
                            {composition_items
                                .iter()
                                .take(display.visible_count)
                                .enumerate()
                                .map(|(index, item)| {
                                    view! {
                                        <li class="sum-comp-legend-item">
                                            <span
                                                class=format!("sum-comp-legend-dot {}", holding_bar_class(index))
                                                aria-hidden="true"
                                            />
                                            <span class="sum-comp-legend-name">{item.view.name.clone()}</span>
                                            <span class="sum-comp-legend-pct">
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
                                    <li class="sum-comp-legend-item">
                                        <span class="sum-comp-legend-dot sum-comp-legend-dot-others" aria-hidden="true" />
                                        <span class="sum-comp-legend-name">
                                            {format!("その他 {}銘柄", others.count)}
                                        </span>
                                        <span class="sum-comp-legend-pct">
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

    let total_annual_dividends = Signal::derive(move || {
        kpi.with(|kpi| {
            kpi.total_annual_dividends
                .map_or("—".to_string(), format_currency)
        })
    });
    let dividend_yield = Signal::derive(move || {
        kpi.with(|kpi| {
            kpi.dividend_yield
                .map_or("—".to_string(), format_percentage_value)
        })
    });
    let display_count_str = move || {
        if is_filtered {
            format!("{display_count} / {total_count}")
        } else {
            display_count.to_string()
        }
    };

    view! {
        <section id="assetbalance-summary" class="sum-card mb-3" data-testid="asset-portfolio-summary">
            <header class="sum-header">
                <strong>"集計情報"</strong>
                {is_filtered.then(|| view! {
                    <div class="sum-header-badges">
                        <Badge variant=BadgeVariant::Info>
                            {format!("絞り込み中: {display_count}/{total_count}件")}
                        </Badge>
                        <Chip variant=ChipVariant::Pill on_click=move |_| on_clear_filter()>
                            "解除"
                        </Chip>
                    </div>
                })}
            </header>
            <div data-testid="portfolio-kpi-strip">
            <dl class="kpis" data-testid="portfolio-kpi-grid">
                <div class="kpi">
                    <dt>"合計取得総額"</dt>
                    <dd class="kpi-value">
                        <Amount
                            block=true
                            text=format_currency(total_purchase_amount)
                            class="kpi-amount"
                        />
                    </dd>
                </div>
                <div class="kpi">
                    <dt>"年間配当金額"</dt>
                    <dd class="kpi-value">
                        <Amount
                            block=true
                            text=total_annual_dividends
                            class="kpi-amount"
                        />
                    </dd>
                </div>
                <div class="kpi">
                    <dt>"配当利回り（年間）"</dt>
                    <dd class="kpi-value">
                        <Amount
                            block=true
                            text=dividend_yield
                            class="kpi-amount"
                        />
                        <small>"年間配当金額 ÷ 合計取得総額"</small>
                    </dd>
                </div>
                <div class="kpi">
                    <dt>"保有銘柄数"</dt>
                    <dd class="kpi-value kpi-count">
                        <span class="tabular-nums">{display_count_str}</span>
                        <span class="kpi-unit">"銘柄"</span>
                    </dd>
                </div>
            </dl>
            </div>
            {composition}
        </section>
    }
        .into_any()
}
