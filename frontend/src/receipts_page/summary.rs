use crate::receipts::{
    select_header_summary, ReceiptItem, ReceiptSummary, ReceiptTabData, ReceiptsTab,
};
use crate::receipts_domain::{
    calculate_dividends, calculate_domestic_total, calculate_mutual_funds, format_currency,
};
use leptos::prelude::*;
use rust_decimal::Decimal;

pub(crate) fn header_summary(
    tab: ReceiptsTab,
    data: &ReceiptTabData,
    query: &str,
    has_preview: bool,
) -> Vec<(&'static str, Decimal, &'static str)> {
    match tab {
        ReceiptsTab::Dividend => {
            let rows: Vec<_> = data
                .rows
                .iter()
                .filter_map(|item| match item {
                    ReceiptItem::Dividend(row) => Some(row.clone()),
                    _ => None,
                })
                .collect();
            let client = calculate_dividends(&rows);
            let api = match &data.summary {
                Some(ReceiptSummary::Dividend(summary)) => Some([
                    summary.total_dividends_before_tax,
                    summary.total_taxes,
                    summary.total_net_amount_received,
                ]),
                _ => None,
            };
            let values = select_header_summary(
                api.as_ref(),
                has_preview,
                query,
                [
                    client.total_dividends_before_tax,
                    client.total_taxes,
                    client.total_net_amount_received,
                ],
            );
            vec![
                ("配当金", values[0], "emerald"),
                ("税額", values[1], "red"),
                ("受取金額", values[2], "emerald"),
            ]
        }
        ReceiptsTab::DomesticStock => {
            let rows: Vec<_> = data
                .rows
                .iter()
                .filter_map(|item| match item {
                    ReceiptItem::DomesticStock(row) => Some(row.clone()),
                    _ => None,
                })
                .collect();
            let client = calculate_domestic_total(&rows);
            let api = match &data.summary {
                Some(ReceiptSummary::DomesticStock(summary)) => Some([
                    summary.total_realized_profit_and_loss,
                    summary.total_taxes,
                    summary.total_realized_profit_and_loss_after_tax,
                ]),
                _ => None,
            };
            let values = select_header_summary(
                api.as_ref(),
                has_preview,
                query,
                [
                    client.total_realized_profit_and_loss,
                    client.total_taxes,
                    client.total_realized_profit_and_loss_after_tax,
                ],
            );
            vec![
                ("実現損益", values[0], "emerald"),
                ("税額", values[1], "red"),
                ("実現損益(税引)", values[2], "emerald"),
            ]
        }
        ReceiptsTab::MutualFund => {
            let rows: Vec<_> = data
                .rows
                .iter()
                .filter_map(|item| match item {
                    ReceiptItem::MutualFund(row) => Some(row.clone()),
                    _ => None,
                })
                .collect();
            let client = calculate_mutual_funds(&rows);
            let api = match &data.summary {
                Some(ReceiptSummary::MutualFund(summary)) => Some([
                    summary.total_realized_profit_and_loss,
                    summary.total_taxes,
                    summary.total_realized_profit_and_loss_after_tax,
                ]),
                _ => None,
            };
            let values = select_header_summary(
                api.as_ref(),
                has_preview,
                query,
                [
                    client.total_realized_profit_and_loss,
                    client.total_taxes,
                    client.total_realized_profit_and_loss_after_tax,
                ],
            );
            vec![
                ("実現損益", values[0], "emerald"),
                ("税額", values[1], "red"),
                ("実現損益(税引)", values[2], "emerald"),
            ]
        }
    }
}

pub(crate) fn kpi_card_bg(tone: &str) -> &'static str {
    match tone {
        "emerald" => "border-teal-200 bg-teal-50",
        "red" => "border-rose-100 bg-rose-50",
        _ => "border-slate-200 bg-white",
    }
}

pub(crate) fn kpi_value_color(tone: &str) -> &'static str {
    match tone {
        "emerald" => "text-teal-700",
        "red" => "text-red-500",
        _ => "text-slate-800",
    }
}

#[component]
fn KpiGrid(
    items: Vec<(&'static str, Decimal, &'static str)>,
    grid_class: &'static str,
) -> impl IntoView {
    view! {
        <div class=grid_class data-testid="kpi-grid">
            {items
                .into_iter()
                .map(|(label, value, tone)| {
                    view! {
                        <div class=format!(
                            "rounded-lg border px-3.5 py-3 {}",
                            kpi_card_bg(tone)
                        )>
                            <p class="mb-1 text-xs font-medium text-slate-600">{label}</p>
                            <p
                                class=format!(
                                    "text-2xl font-bold tabular-nums {}",
                                    kpi_value_color(tone)
                                )
                                data-negative=(value < Decimal::ZERO).then_some("true")
                            >
                                {format_currency(value)}
                            </p>
                        </div>
                    }
                })
                .collect_view()}
        </div>
    }
}

#[component]
pub(crate) fn SummaryStrip(
    items: Vec<(&'static str, Decimal, &'static str)>,
    expanded: RwSignal<bool>,
) -> impl IntoView {
    let mobile_expanded = expanded;
    let mobile_body_id = "receipt-summary-mobile-body";
    let primary = items.last().copied();
    let mobile_items = items.clone();
    view! {
        <section
            class="collapsible-card"
            data-testid="receipt-summary-strip"
        >
            <div class="sm:hidden" data-testid="receipt-summary-compact">
                <button
                    type="button"
                    class="collapsible-trigger"
                    on:click=move |_| mobile_expanded.update(|expanded| *expanded = !*expanded)
                    aria-expanded=move || {
                        if mobile_expanded.get() {
                            "true"
                        } else {
                            "false"
                        }
                    }
                    aria-controls=mobile_body_id
                    aria-label=primary
                        .map(|(label, value, _)| format!("{label} {}", format_currency(value)))
                        .unwrap_or_else(|| "集計情報".to_string())
                    data-testid="receipt-summary-compact-toggle"
                >
                    {primary
                        .map(|(label, value, tone)| {
                            view! {
                                <span class="flex min-w-0 items-baseline gap-2">
                                    <span class="shrink-0 text-xs font-medium text-slate-600">
                                        {label}
                                    </span>
                                    <span
                                        class=format!(
                                            "truncate text-base font-bold tabular-nums {}",
                                            kpi_value_color(tone)
                                        )
                                        data-negative=(value < Decimal::ZERO).then_some("true")
                                    >
                                        {format_currency(value)}
                                    </span>
                                </span>
                            }
                                .into_any()
                        })
                        .unwrap_or_else(|| {
                            view! {
                                <span class="text-sm font-black text-slate-950">"集計情報"</span>
                            }
                                .into_any()
                        })}
                    <span class="flex shrink-0 items-center gap-1 text-slate-700">
                        <span class="text-xs font-semibold">
                            {move || if mobile_expanded.get() { "閉じる" } else { "開く" }}
                        </span>
                        <svg
                            aria-hidden="true"
                            class=move || {
                                if mobile_expanded.get() {
                                    "h-4 w-4 text-slate-500 transition-transform duration-200 rotate-180"
                                } else {
                                    "h-4 w-4 text-slate-500 transition-transform duration-200"
                                }
                            }
                            fill="none"
                            stroke="currentColor"
                            viewBox="0 0 24 24"
                        >
                            <path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                stroke-width="2"
                                d="M19 9l-7 7-7-7"
                            />
                        </svg>
                    </span>
                </button>
                <div
                    id=mobile_body_id
                    role="region"
                    aria-label="集計情報"
                    hidden=move || !mobile_expanded.get()
                    class="border-t border-slate-950/10 py-3"
                >
                    {move || {
                        mobile_expanded
                            .get()
                            .then(|| {
                                view! {
                                    <KpiGrid
                                        items=mobile_items.clone()
                                        grid_class="grid grid-cols-1 gap-2.5"
                                    />
                                }
                            })
                    }}
                </div>
            </div>
            <div class="hidden sm:block" data-testid="receipt-summary-desktop">
                <div
                    class="flex items-start justify-between gap-3 border-b border-slate-950/10 pb-2.5"
                    data-testid="receipt-header"
                >
                    <div>
                        <h2 class="text-sm font-black text-slate-950">"集計情報"</h2>
                    </div>
                </div>
                <div class="pt-3">
                    <KpiGrid
                        items=items
                        grid_class="grid grid-cols-1 gap-2.5 sm:grid-cols-2 xl:grid-cols-3"
                    />
                </div>
            </div>
        </section>
    }
}
