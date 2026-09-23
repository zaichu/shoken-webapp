use crate::receipts::{
    select_header_summary, use_receipts_data, ReceiptItem, ReceiptSummary, ReceiptTabData,
    ReceiptsStore, ReceiptsTab, TabState,
};
use crate::receipts_domain::{
    calculate_dividends, calculate_domestic_daily, calculate_domestic_total,
    calculate_mutual_funds, create_year_month_key, format_currency, group_dividends_by_month,
    group_mutual_funds_by_month, sort_dividends, sort_domestic_stocks, sort_mutual_funds,
};
use crate::session::use_session;
use leptos::prelude::*;
use rust_decimal::Decimal;

const TAB_IDS: [&str; 3] = ["dividend", "domesticstock", "mutualfund"];

#[component]
pub fn ReceiptsPage() -> impl IntoView {
    let store = use_receipts_data(use_session(), ReceiptsTab::Dividend);

    view! {
        <div class="page-surface">
            <div class="mb-5 max-sm:mb-2">
                <div class="flex flex-col gap-3 border-l-4 border-amber-500 pl-4 sm:flex-row sm:items-end sm:justify-between">
                    <div>
                        <p class="mb-1 text-[11px] font-bold uppercase tracking-[0.22em] text-slate-500 max-sm:hidden">
                            "Transactions"
                        </p>
                        <h1 class="text-2xl font-black leading-tight tracking-normal text-slate-950 max-sm:text-lg">
                            "取引明細"
                        </h1>
                        <p class="mt-1 text-sm font-medium text-slate-600 max-sm:hidden">
                            "配当金・国内株式・投資信託の取引明細を管理します。"
                        </p>
                    </div>
                </div>
            </div>
            {{
                let failed = store.clone();
                move || {
                    failed.error().map(|message| {
                        view! { <div role="alert">{message}</div> }
                    })
                }
            }}
            <nav class="mb-2 no-print" aria-label="取引明細タブ">
                <div
                    class="flex flex-wrap gap-1.5 rounded-xl border border-slate-950/10 bg-white/70 p-1 shadow-sm max-sm:flex-nowrap max-sm:gap-1 max-sm:overflow-x-auto"
                    role="tablist"
                >
                    {ReceiptsTab::ALL
                        .iter()
                        .copied()
                        .map(|tab| {
                            view! { <TabButton store=store.clone() tab=tab /> }
                        })
                        .collect_view()}
                </div>
            </nav>
            <div class="mt-0">
                <div data-testid="receipts-workspace">
                    {ReceiptsTab::ALL
                        .iter()
                        .copied()
                        .map(|tab| {
                            view! { <TabPanel store=store.clone() tab=tab /> }
                        })
                        .collect_view()}
                </div>
            </div>
        </div>
    }
}

#[component]
fn TabButton(store: ReceiptsStore, tab: ReceiptsTab) -> impl IntoView {
    let slug = TAB_IDS[tab as usize];
    let label = tab.label();
    let selected = store.clone();
    let index = store.clone();
    let clicked = store.clone();
    let counted = store.clone();
    view! {
        <button
            id={format!("tab-{slug}")}
            type="button"
            role="tab"
            aria-selected=move || selected.active_tab.get() == tab
            aria-controls={format!("tabpanel-{slug}")}
            tabindex=move || if index.active_tab.get() == tab { "0" } else { "-1" }
            on:click=move |_| clicked.select_tab(tab)
        >
            {label}
            <span data-testid={format!("tab-count-{slug}")}>
                {move || counted.count(tab).to_string()}
            </span>
        </button>
    }
}

#[component]
fn TabPanel(store: ReceiptsStore, tab: ReceiptsTab) -> impl IntoView {
    let slug = TAB_IDS[tab as usize];
    let hidden = store.clone();
    let rendered = store.clone();
    view! {
        <div
            id={format!("tabpanel-{slug}")}
            role="tabpanel"
            aria-labelledby={format!("tab-{slug}")}
            hidden=move || hidden.active_tab.get() != tab
        >
            {move || {
                let store = rendered.clone();
                if store.active_tab.get() != tab {
                    return ().into_any();
                }
                if !store.is_authenticated() {
                    return view! { <p role="status">"読み込み中..."</p> }.into_any();
                }
                match store.tab_state(tab) {
                    TabState::Loading => view! { <p role="status">"読み込み中..."</p> }.into_any(),
                    TabState::Failed(message) => {
                        view! {
                            <div role="alert">
                                <strong>"エラー:"</strong>
                                " "
                                {message}
                            </div>
                        }
                            .into_any()
                    }
                    TabState::Ready(data) => {
                        if data.rows.is_empty() {
                            view! {
                                <div>
                                    <h3>"データがありません"</h3>
                                    <p>{empty_hint(tab)}</p>
                                </div>
                            }
                                .into_any()
                        } else {
                            view! { <ReceiptContent tab=tab data=data /> }.into_any()
                        }
                    }
                }
            }}
        </div>
    }
}

fn empty_hint(tab: ReceiptsTab) -> &'static str {
    match tab {
        ReceiptsTab::Dividend => "配当金明細をCSVで追加してください",
        ReceiptsTab::DomesticStock => "国内株式明細をCSVで追加してください",
        ReceiptsTab::MutualFund => "投資信託明細をCSVで追加してください",
    }
}

#[component]
fn ReceiptContent(tab: ReceiptsTab, data: ReceiptTabData) -> impl IntoView {
    let header = header_summary(tab, &data);
    view! {
        <section>
            <div class="mb-3 grid grid-cols-1 gap-2 sm:grid-cols-3" aria-label="集計情報">
                {header
                    .into_iter()
                    .map(|(label, value)| {
                        view! {
                            <div class="rounded-lg border border-slate-200 bg-white px-4 py-3">
                                <div class="text-xs font-semibold text-slate-600">{label}</div>
                                <div class="mt-1 text-right font-mono text-lg font-bold tabular-nums">
                                    {format_currency(value)}
                                </div>
                            </div>
                        }
                    })
                    .collect_view()}
            </div>
            <ReceiptTable tab=tab rows=data.rows />
        </section>
    }
}

fn header_summary(tab: ReceiptsTab, data: &ReceiptTabData) -> Vec<(&'static str, Decimal)> {
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
                false,
                "",
                [
                    client.total_dividends_before_tax,
                    client.total_taxes,
                    client.total_net_amount_received,
                ],
            );
            vec![
                ("配当金", values[0]),
                ("税額", values[1]),
                ("受取金額", values[2]),
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
                false,
                "",
                [
                    client.total_realized_profit_and_loss,
                    client.total_taxes,
                    client.total_realized_profit_and_loss_after_tax,
                ],
            );
            vec![
                ("実現損益", values[0]),
                ("税額", values[1]),
                ("実現損益(税引)", values[2]),
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
                false,
                "",
                [
                    client.total_realized_profit_and_loss,
                    client.total_taxes,
                    client.total_realized_profit_and_loss_after_tax,
                ],
            );
            vec![
                ("実現損益", values[0]),
                ("税額", values[1]),
                ("実現損益(税引)", values[2]),
            ]
        }
    }
}

struct TableGroup {
    label: String,
    summary: Vec<String>,
    rows: Vec<Vec<String>>,
}

fn group_label(key: &str) -> String {
    let parts: Vec<_> = key.split('-').collect();
    match parts.as_slice() {
        [year, month, day] => format!(
            "{year}年{}月{}日",
            month.parse::<u32>().unwrap_or(0),
            day.parse::<u32>().unwrap_or(0)
        ),
        [year, month] => format!("{year}年{}月", month.parse::<u32>().unwrap_or(0)),
        _ => key.to_string(),
    }
}

fn table_groups(tab: ReceiptsTab, rows: &[ReceiptItem]) -> Vec<TableGroup> {
    match tab {
        ReceiptsTab::Dividend => {
            let typed: Vec<_> = rows
                .iter()
                .filter_map(|item| match item {
                    ReceiptItem::Dividend(row) => Some(row.clone()),
                    _ => None,
                })
                .collect();
            let sorted = sort_dividends(&typed);
            group_dividends_by_month(&sorted)
                .into_iter()
                .map(|summary| {
                    let group_rows = sorted
                        .iter()
                        .filter(|row| create_year_month_key(&row.settlement_date) == summary.filter)
                        .cloned()
                        .map(ReceiptItem::Dividend)
                        .map(|item| item.cells())
                        .collect();
                    TableGroup {
                        label: group_label(&summary.filter),
                        summary: vec![
                            format_currency(summary.total_dividends_before_tax),
                            format_currency(summary.total_taxes),
                            format_currency(summary.total_net_amount_received),
                        ],
                        rows: group_rows,
                    }
                })
                .collect()
        }
        ReceiptsTab::DomesticStock => {
            let typed: Vec<_> = rows
                .iter()
                .filter_map(|item| match item {
                    ReceiptItem::DomesticStock(row) => Some(row.clone()),
                    _ => None,
                })
                .collect();
            let sorted = sort_domestic_stocks(&typed);
            calculate_domestic_daily(&sorted)
                .into_iter()
                .map(|summary| {
                    let group_rows = sorted
                        .iter()
                        .filter(|row| row.trade_date == summary.filter)
                        .cloned()
                        .map(ReceiptItem::DomesticStock)
                        .map(|item| item.cells())
                        .collect();
                    TableGroup {
                        label: group_label(&summary.filter),
                        summary: vec![
                            format_currency(summary.total_realized_profit_and_loss),
                            format_currency(summary.total_taxes),
                            format_currency(summary.total_realized_profit_and_loss_after_tax),
                        ],
                        rows: group_rows,
                    }
                })
                .collect()
        }
        ReceiptsTab::MutualFund => {
            let typed: Vec<_> = rows
                .iter()
                .filter_map(|item| match item {
                    ReceiptItem::MutualFund(row) => Some(row.clone()),
                    _ => None,
                })
                .collect();
            let sorted = sort_mutual_funds(&typed);
            group_mutual_funds_by_month(&sorted)
                .into_iter()
                .map(|summary| {
                    let group_rows = sorted
                        .iter()
                        .filter(|row| create_year_month_key(&row.trade_date) == summary.filter)
                        .cloned()
                        .map(ReceiptItem::MutualFund)
                        .map(|item| item.cells())
                        .collect();
                    TableGroup {
                        label: group_label(&summary.filter),
                        summary: vec![
                            format_currency(summary.realized_profit_and_loss),
                            format_currency(summary.taxes),
                            format_currency(summary.realized_profit_and_loss_after_tax),
                        ],
                        rows: group_rows,
                    }
                })
                .collect()
        }
    }
}

#[component]
fn ReceiptTable(tab: ReceiptsTab, rows: Vec<ReceiptItem>) -> impl IntoView {
    let headers: &[&str] = match tab {
        ReceiptsTab::Dividend => &[
            "入金日",
            "商品",
            "口座",
            "銘柄コード",
            "銘柄名",
            "単価",
            "数量",
            "配当金",
            "税額",
            "受取額",
        ],
        ReceiptsTab::DomesticStock => &[
            "約定日",
            "銘柄コード",
            "銘柄名",
            "口座",
            "数量",
            "売却単価",
            "売却額",
            "取得価額",
            "損益",
            "税額",
            "税引後",
        ],
        ReceiptsTab::MutualFund => &[
            "約定日",
            "ファンド名",
            "口座",
            "数量",
            "解約単価",
            "解約額",
            "取得価額",
            "実現損益",
            "税額",
            "税引損益",
        ],
    };
    let groups = table_groups(tab, &rows);
    view! {
        <div class="overflow-x-auto rounded-lg border border-slate-200 bg-white">
            <table class="min-w-full border-collapse text-sm">
                <thead>
                    <tr class="bg-slate-800 text-left text-white">
                        {headers
                            .iter()
                            .map(|header| view! { <th class="whitespace-nowrap px-3 py-2">{*header}</th> })
                            .collect_view()}
                    </tr>
                </thead>
                <tbody>
                    {groups
                        .into_iter()
                        .map(|group| {
                            let count = group.rows.len();
                            view! {
                                <tr class="border-t-2 border-slate-300 bg-slate-100 font-semibold">
                                    <td colspan={headers.len() - 3} class="whitespace-nowrap px-3 py-2">
                                        {group.label}
                                        <span class="ml-2 text-xs text-slate-600">{format!("{count}件")}</span>
                                    </td>
                                    {group
                                        .summary
                                        .into_iter()
                                        .map(|value| view! { <td class="whitespace-nowrap px-3 py-2 text-right font-mono">{value}</td> })
                                        .collect_view()}
                                </tr>
                                {group
                                    .rows
                                    .into_iter()
                                    .map(|cells| {
                                        view! {
                                            <tr class="border-t border-slate-200">
                                                {cells
                                                    .into_iter()
                                                    .map(|cell| view! { <td class="whitespace-nowrap px-3 py-2">{cell}</td> })
                                                    .collect_view()}
                                            </tr>
                                        }
                                    })
                                    .collect_view()}
                            }
                        })
                        .collect_view()}
                </tbody>
            </table>
        </div>
    }
}
