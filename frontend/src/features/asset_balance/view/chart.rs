use super::cards::{HoldingCard, HoldingValuationCard};
use super::summary::ChartItem;
use crate::features::asset_balance::format::format_fixed_percent;
use crate::features::asset_balance::portfolio::chart_display;
use crate::features::dividend_per_share::DividendMaps;
use crate::ui::button::{Button, ButtonVariant};
use crate::ui::card::{Card, CardVariant};
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
                            <Card variant=CardVariant::DashedCompact>
                                <div class="flex items-center justify-between gap-2 mb-2">
                                    <span class="text-sm text-text-subtle">
                                        {format!("その他 {}銘柄", others.count)}
                                    </span>
                                    <span class="text-lg font-bold text-text-subtle">
                                        {format_fixed_percent(others.percentage, 1)}
                                    </span>
                                </div>
                                <div class="h-2.5 w-full rounded-full bg-fill">
                                    <div
                                        class="h-full rounded-full bg-fill-strong transition-all duration-300"
                                        style=format!("width: {}%", others.percentage.min(100.0))
                                    />
                                </div>
                            </Card>
                        }
                    })
            }}
        </div>
        {move || {
            display
                .with(|d| d.toggle_label.clone())
                .map(|label| {
                    let label = Signal::stored(label);
                    view! {
                        <Button
                            variant=ButtonVariant::Quiet
                            on_click=move |_| show_all.update(|open| *open = !*open)
                        >
                            {label}
                        </Button>
                    }
                })
        }}
    }
}
