use crate::features::receipts::model::format_currency;
use crate::features::receipts::{select_header_summary, ReceiptTabData, ReceiptsTab};
use crate::ui::amount::Amount;
use crate::ui::card::{Card, CardVariant, SectionHeader, SectionHeaderVariant, StatTone};
use leptos::prelude::*;
use rust_decimal::Decimal;

fn profit_tone(value: Decimal) -> StatTone {
    if value < Decimal::ZERO {
        StatTone::Loss
    } else {
        StatTone::Neutral
    }
}

pub(crate) fn header_summary(
    tab: ReceiptsTab,
    data: &ReceiptTabData,
    query: &str,
    has_preview: bool,
) -> Vec<(&'static str, Decimal, StatTone)> {
    let api = data
        .summary
        .as_ref()
        .and_then(|s| tab.api_summary_triple(s));
    let values = select_header_summary(
        api.as_ref(),
        has_preview,
        query,
        tab.client_summary_triple(&data.rows),
    );
    tab.header_items()
        .into_iter()
        .enumerate()
        .map(|(index, (label, marks_profit))| {
            let tone = if marks_profit {
                profit_tone(values[index])
            } else {
                StatTone::Neutral
            };
            (label, values[index], tone)
        })
        .collect()
}

pub(crate) fn kpi_value_color(tone: StatTone) -> &'static str {
    match tone {
        StatTone::Loss => "text-negative",
        StatTone::Neutral => "text-text",
    }
}

#[component]
fn KpiGrid(
    items: Vec<(&'static str, Decimal, StatTone)>,
    grid_class: &'static str,
) -> impl IntoView {
    view! {
        <div class=grid_class data-testid="kpi-grid">
            {items
                .into_iter()
                .map(|(label, value, tone)| {
                    let negative = matches!(tone, StatTone::Loss);
                    let value = format_currency(value);
                    view! {
                        <Card variant=CardVariant::StatSmall(tone)>
                            <p class="mb-1 text-xs font-medium text-text-muted">{label}</p>
                            <Amount
                                block=true
                                text=value
                                class=format!("text-2xl font-bold {}", kpi_value_color(tone))
                                negative=negative
                            />
                        </Card>
                    }
                })
                .collect_view()}
        </div>
    }
}

#[component]
pub(crate) fn SummaryStrip(
    items: Vec<(&'static str, Decimal, StatTone)>,
    expanded: RwSignal<bool>,
    #[prop(optional)] preview: bool,
) -> impl IntoView {
    let title = if preview {
        "集計情報(プレビュー)"
    } else {
        "集計情報"
    };
    let mobile_expanded = expanded;
    let mobile_body_id = "receipt-summary-mobile-body";
    let mobile_items = items.clone();
    view! {
        <Card variant=CardVariant::Collapsible testid="receipt-summary-strip" class=Signal::derive(move || {
            if mobile_expanded.get() { "" } else { "max-sm:hidden" }.to_string()
        })>
            <div
                id=mobile_body_id
                class="sm:hidden"
                role="region"
                aria-label=title
                hidden=move || !mobile_expanded.get()
            >
                {move || mobile_expanded.get().then(|| view! {
                    <KpiGrid items=mobile_items.clone() grid_class="grid grid-cols-1 gap-2.5" />
                })}
            </div>
            <div class="hidden sm:block" data-testid="receipt-summary-desktop">
                <SectionHeader variant=SectionHeaderVariant::Divider testid="receipt-header">
                    {title}
                </SectionHeader>
                <div class="pt-3">
                    <KpiGrid
                        items=items
                        grid_class="grid grid-cols-1 gap-2.5 sm:grid-cols-2 xl:grid-cols-3"
                    />
                </div>
            </div>
        </Card>
    }
}
