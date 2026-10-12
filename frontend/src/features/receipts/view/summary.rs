use crate::features::receipts::{ReceiptTabData, ReceiptsTab, select_header_summary};
use crate::ui::amount::Amount;
use crate::ui::card::StatTone;
use leptos::prelude::*;
use rust_decimal::Decimal;
use shared::format::format_currency;

pub fn profit_tone(value: Decimal) -> StatTone {
    if value < Decimal::ZERO {
        StatTone::Loss
    } else {
        StatTone::Neutral
    }
}

pub fn header_summary(
    tab: ReceiptsTab,
    data: &ReceiptTabData,
    query: &str,
) -> Vec<(&'static str, Decimal, StatTone)> {
    let api = data
        .summary
        .as_ref()
        .and_then(|s| tab.api_summary_triple(s));
    let values = select_header_summary(api.as_ref(), query, tab.client_summary_triple(&data.rows));
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

pub fn kpi_value_color(tone: StatTone) -> &'static str {
    match tone {
        StatTone::Loss => "text-negative",
        StatTone::Neutral => "text-text",
    }
}

#[component]
pub fn SummaryStrip(
    items: Vec<(&'static str, Decimal, StatTone)>,
    expanded: RwSignal<bool>,
) -> impl IntoView {
    let _ = expanded;
    view! {
        <section
            data-testid="receipt-summary-strip"
            role="region"
            aria-label="集計"
            class="receipt-summary"
        >
            <dl class="receipt-summary-grid" data-testid="kpi-grid">
                {items
                    .into_iter()
                    .enumerate()
                    .map(|(index, (label, value, tone))| {
                        let negative = matches!(tone, StatTone::Loss);
                        let item_class = if index == 2 {
                            "receipt-summary-item receipt-summary-item-primary"
                        } else {
                            "receipt-summary-item"
                        };
                        view! {
                            <div class=item_class>
                                <dt>{label}</dt>
                                <dd>
                                    <Amount
                                        text=format_currency(value)
                                        class=format!("receipt-summary-amount {}", kpi_value_color(tone))
                                        negative=negative
                                    />
                                </dd>
                            </div>
                        }
                    })
                    .collect_view()}
            </dl>
        </section>
    }
}
