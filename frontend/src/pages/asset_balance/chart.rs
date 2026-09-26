use super::cards::{HoldingCard, HoldingValuationCard};
use super::format::format_fixed_percent;
use super::summary::ChartItem;
use crate::asset_balance::portfolio::chart_display;
use crate::dividend_per_share::DividendMaps;
use leptos::prelude::*;

#[component]
pub(crate) fn ChartList(
    items: Vec<ChartItem>,
    dividends: RwSignal<DividendMaps>,
    show_all: RwSignal<bool>,
) -> impl IntoView {
    let total = items.len();
    let percentages: Vec<Option<f64>> = items.iter().map(|item| item.percentage).collect();
    let display = Memo::new(move |_| chart_display(total, &percentages, show_all.get()));
    view! {
        <div
            class=move || display.with(|display| display.grid_class)
            data-testid="portfolio-items-grid"
        >
            {move || {
                items
                    .clone()
                    .into_iter()
                    .enumerate()
                    .filter(|(index, _)| *index < display.with(|d| d.visible_count))
                    .map(|(index, item)| {
                        view! {
                            <div>
                                <HoldingCard
                                    item=item.clone()
                                    index=index
                                    dividends=dividends
                                />
                                <HoldingValuationCard item=item dividends=dividends />
                            </div>
                        }
                    })
                    .collect_view()
            }}
            {move || {
                display
                    .with(|d| d.others.clone())
                    .map(|others| {
                        view! {
                            <div class="rounded-lg border border-dashed border-slate-300 bg-slate-50 p-3">
                                <div class="flex items-center justify-between gap-2 mb-2">
                                    <span class="text-sm text-slate-500">
                                        {format!("その他 {}銘柄", others.count)}
                                    </span>
                                    <span class="text-lg font-bold text-slate-500">
                                        {format_fixed_percent(others.percentage, 1)}
                                    </span>
                                </div>
                                <div class="h-2.5 w-full rounded-full bg-slate-200">
                                    <div
                                        class="h-full rounded-full bg-slate-400 transition-all duration-300"
                                        style=format!("width: {}%", others.percentage.min(100.0))
                                    />
                                </div>
                            </div>
                        }
                    })
            }}
        </div>
        {move || {
            display
                .with(|d| d.toggle_label.clone())
                .map(|label| {
                    view! {
                        <button
                            type="button"
                            class="chart-toggle-button"
                            on:click=move |_| show_all.update(|open| *open = !*open)
                        >
                            {label}
                        </button>
                    }
                })
        }}
    }
}
