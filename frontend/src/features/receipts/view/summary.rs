use crate::features::receipts::{ReceiptTabData, ReceiptsTab, select_header_summary};
use crate::settings::format_amount;
use crate::ui::amount::Amount;
use crate::ui::card::StatTone;
use leptos::prelude::*;
use rust_decimal::Decimal;

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

#[component]
pub fn SummaryStrip(items: Vec<(&'static str, Decimal, StatTone)>) -> impl IntoView {
    // 見本では主値(税引後)を金の見出しで先頭に大きく出す。items の末尾が主値
    let mut secondary = items;
    let primary = secondary.pop();
    view! {
        <section
            data-testid="receipt-summary-strip"
            role="region"
            aria-label="集計"
            class="receipt-summary"
        >
            <dl class="receipt-summary-grid" data-testid="kpi-grid">
                {primary
                    .map(|(label, value, tone)| {
                        let negative = matches!(tone, StatTone::Loss);
                        view! {
                            <div class="receipt-summary-item receipt-summary-item-primary">
                                <dt>{label}</dt>
                                <dd>
                                    <Amount
                                        text=format_amount(value)
                                        class=if negative {
                                            "receipt-summary-primary-amount receipt-summary-amount-loss"
                                        } else {
                                            "receipt-summary-primary-amount"
                                        }
                                    />
                                </dd>
                            </div>
                        }
                    })}
                {secondary
                    .into_iter()
                    .map(|(label, value, tone)| {
                        let negative = matches!(tone, StatTone::Loss);
                        view! {
                            <div class="receipt-summary-item receipt-summary-item-secondary">
                                <dt>{label}</dt>
                                <dd>
                                    <Amount
                                        text=format_amount(value)
                                        class=if negative {
                                            "receipt-summary-amount receipt-summary-amount-loss"
                                        } else {
                                            "receipt-summary-amount"
                                        }
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
