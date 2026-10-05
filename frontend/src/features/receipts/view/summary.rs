use crate::features::receipts::{select_header_summary, ReceiptTabData, ReceiptsTab};
use crate::ui::amount::Amount;
use crate::ui::badge::{Badge, BadgeVariant};
use crate::ui::card::StatTone;
use leptos::prelude::*;
use rust_decimal::Decimal;
use shared::format::format_currency;

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
            <section
                data-testid="receipt-summary-strip"
                role="region"
                aria-label=title
                class=move || {
                    if mobile_expanded.get() {
                        "sum-card mb-3".to_string()
                    } else {
                        "sum-card mb-3 max-sm:hidden".to_string()
                    }
                }
            >
                <header class="sum-header">
                    <strong>{title}</strong>
                    {preview.then(|| view! {
                        <Badge variant=BadgeVariant::AccentFlat>"プレビュー"</Badge>
                    })}
                </header>
                <div class="hidden sm:block" data-testid="receipt-summary-desktop">
                <dl class="kpis" data-testid="kpi-grid">
                    {items
                        .into_iter()
                        .map(|(label, value, tone)| {
                            let negative = matches!(tone, StatTone::Loss);
                            view! {
                                <div class="kpi">
                                    <dt>{label}</dt>
                                    <dd class="kpi-value">
                                        <Amount
                                            text=format_currency(value)
                                            class=format!("kpi-amount {}", kpi_value_color(tone))
                                            negative=negative
                                        />
                                    </dd>
                                </div>
                            }
                        })
                        .collect_view()}
                </dl>
                </div>
                <div
                    id=mobile_body_id
                    class="sm:hidden sum-mobile"
                    hidden=move || !mobile_expanded.get()
                >
                    {move || mobile_expanded.get().then(|| view! {
                        <div class="flex flex-col gap-2">
                            {mobile_items
                                .clone()
                                .into_iter()
    .map(|(label, value, tone)| {
                                    let negative = matches!(tone, StatTone::Loss);
                                    view! {
                                        <div class="flex items-baseline justify-between gap-3">
                                            <span class="text-sm text-text-muted">{label}</span>
                                            <Amount
                                                text=format_currency(value)
                                                class=format!("text-base font-bold {}", kpi_value_color(tone))
                                                negative=negative
                                            />
                                        </div>
                                    }
                                })
                                .collect_view()}
                        </div>
                    })}
                </div>
            </section>
        }
}
