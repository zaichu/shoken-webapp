use crate::dto::{Dividend, Mutualfund};
use crate::receipts::{
    select_header_summary, use_receipts_data, ReceiptItem, ReceiptSummary, ReceiptTabData,
    ReceiptsStore, ReceiptsTab, TabState,
};
use crate::receipts_domain::{
    calculate_dividends, calculate_domestic_daily, calculate_domestic_total,
    calculate_mutual_funds, create_year_month_key, format_currency, sort_dividends,
    sort_domestic_stocks, sort_mutual_funds,
};
use crate::receipts_filter::{column_order, search_categories, ReceiptSearch};
use crate::receipts_search::SearchOption;
use crate::receipts_search_group_key::{create_group_key_fn, GroupKeyRule};
use crate::receipts_search_support::group_and_summarize;
use crate::session::use_session;
use leptos::prelude::*;
use rust_decimal::Decimal;
use std::collections::HashMap;

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
                        view! { <ReceiptContent store=store tab=tab data=data /> }.into_any()
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
fn ReceiptContent(store: ReceiptsStore, tab: ReceiptsTab, data: ReceiptTabData) -> impl IntoView {
    let search = store.search;
    let categories = search_categories(tab, &data.rows);
    let filtered = Memo::new(move |_| store.filtered_rows(tab));
    let all_rows = data.rows.clone();
    view! {
        <section>
            <div class="mb-3 rounded-lg border border-slate-200 bg-white p-4" role="search" aria-label="取引明細の検索">
                <div class="mb-3 flex items-center justify-between gap-3">
                    <h2 class="text-sm font-bold text-slate-950">"検索オプション"</h2>
                    <button type="button" class="rounded border border-slate-300 px-3 py-1 text-sm" aria-label="検索条件をクリア"
                        disabled=move || search.with(|s| s.query.is_empty())
                        on:click=move |_| search.set(ReceiptSearch::default())>"絞り込み解除"</button>
                </div>
                <label class="mb-1 block text-sm font-semibold" for="receipt-search-query">"検索"</label>
                <input id="receipt-search-query" type="search" class="mb-3 w-full rounded border border-slate-300 px-3 py-2"
                    placeholder="銘柄・口座・金額など（空白区切りでAND検索）"
                    aria-describedby="receipt-search-help"
                    prop:value=move || search.with(|s| s.query.clone())
                    on:input=move |ev| search.set(ReceiptSearch { query: event_target_value(&ev), ..Default::default() }) />
                <div class="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-4">
                    <SearchCategory search=search index=0 label="銘柄" options=categories.securities />
                    <SearchCategory search=search index=1 label="商品" options=categories.products />
                    <SearchCategory search=search index=2 label="口座" options=categories.accounts />
                    <div>
                        <label class="mb-1 block text-sm font-semibold" for="receipt-search-period">"年・年月・日付・期間"</label>
                        <input id="receipt-search-period" type="text" list="receipt-search-years" class="w-full rounded border border-slate-300 px-3 py-2"
                            placeholder="例: 2024 / 2024-03"
                            prop:value=move || search.with(|s| s.selections[3].clone())
                            on:input=move |ev| search.update(|s| s.select(3, event_target_value(&ev))) />
                        <datalist id="receipt-search-years">
                            {categories.years.into_iter().map(|o| view! { <option value=o.value>{o.label}</option> }).collect_view()}
                        </datalist>
                    </div>
                </div>
                <p id="receipt-search-help" class="mt-3 text-xs text-slate-600">"空白を含む名前は引用符で囲みます。期間は 2024-01-01..2024-12-31 の形式で入力できます。"</p>
            </div>
            {move || {
                if all_rows.is_empty() {
                    return view! { <div><h3>"データがありません"</h3><p>{empty_hint(tab)}</p></div> }.into_any();
                }
                let query = search.with(|s| s.query.clone());
                let rows = filtered.get();
                let header = header_summary(tab, &ReceiptTabData { rows: rows.clone(), summary: data.summary.clone() }, &query);
                let count = rows.len();
                view! {
                    <div class="mb-3 grid grid-cols-1 gap-2 sm:grid-cols-3" aria-label="集計情報">
                        {header.into_iter().map(|(label, value)| view! {
                            <div class="rounded-lg border border-slate-200 bg-white px-4 py-3">
                                <div class="text-xs font-semibold text-slate-600">{label}</div>
                                <div class="mt-1 text-right font-mono text-lg font-bold tabular-nums">{format_currency(value)}</div>
                            </div>
                        }).collect_view()}
                    </div>
                    <p class="mb-2 text-sm text-slate-600" role="status">{format!("{count}件")}</p>
                    {if rows.is_empty() {
                        view! { <p class="rounded-lg border border-slate-200 bg-white p-4">"検索条件に一致するデータがありません"</p> }.into_any()
                    } else {
                        view! { <ReceiptTable tab=tab rows=rows all_rows=all_rows.clone() query=query /> }.into_any()
                    }}
                }.into_any()
            }}
        </section>
    }
}

#[component]
fn SearchCategory(
    search: RwSignal<ReceiptSearch>,
    index: usize,
    label: &'static str,
    options: Vec<SearchOption>,
) -> impl IntoView {
    if options.is_empty() {
        return ().into_any();
    }
    let id = format!("receipt-search-category-{index}");
    view! {
        <div>
            <label class="mb-1 block text-sm font-semibold" for=id.clone()>{label}</label>
            <select id=id class="w-full rounded border border-slate-300 px-3 py-2"
                prop:value=move || search.with(|s| s.selections[index].clone())
                on:change=move |ev| search.update(|s| s.select(index, event_target_value(&ev)))>
                <option value="">"すべて"</option>
                {options.into_iter().map(|o| view! { <option value=o.value>{o.label}</option> }).collect_view()}
            </select>
        </div>
    }.into_any()
}

fn header_summary(
    tab: ReceiptsTab,
    data: &ReceiptTabData,
    query: &str,
) -> Vec<(&'static str, Decimal)> {
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
                query,
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
                query,
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
                query,
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
    let is_date = (key.len() == 7 || key.len() == 10)
        && key.bytes().enumerate().all(|(i, b)| {
            if i == 4 || i == 7 {
                b == b'-'
            } else {
                b.is_ascii_digit()
            }
        });
    if !is_date {
        return key.to_string();
    }
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

fn table_groups(
    tab: ReceiptsTab,
    rows: &[ReceiptItem],
    all_rows: &[ReceiptItem],
    query: &str,
) -> Vec<TableGroup> {
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
            let mut latest: HashMap<&str, &Dividend> = HashMap::new();
            for item in all_rows {
                if let ReceiptItem::Dividend(row) = item {
                    let current = latest.entry(&row.security_code).or_insert(row);
                    if row.settlement_date > current.settlement_date {
                        *current = row;
                    }
                }
            }
            let security_key = |row: &Dividend| {
                latest
                    .get(row.security_code.as_str())
                    .map(|r| r.security_name.clone())
                    .unwrap_or_else(|| row.security_name.clone())
            };
            let rules = [
                GroupKeyRule {
                    test: |r: &Dividend, t| {
                        r.security_code.to_lowercase() == t || r.security_name.to_lowercase() == t
                    },
                    key_fn: &security_key,
                },
                GroupKeyRule {
                    test: |r: &Dividend, t| r.product.to_lowercase() == t,
                    key_fn: &|r: &Dividend| r.product.clone(),
                },
                GroupKeyRule {
                    test: |r: &Dividend, t| r.account.to_lowercase() == t,
                    key_fn: &|r: &Dividend| r.account.clone(),
                },
            ];
            let key =
                create_group_key_fn(query, |r| create_year_month_key(&r.settlement_date), &rules);
            group_and_summarize(
                &sorted,
                &key,
                &[
                    |r| r.dividends_before_tax,
                    |r| r.taxes,
                    |r| r.net_amount_received,
                ],
                true,
            )
            .into_iter()
            .map(|summary| {
                let group_rows = sorted
                    .iter()
                    .filter(|row| key(row) == summary.filter)
                    .cloned()
                    .map(ReceiptItem::Dividend)
                    .map(|item| item.cells())
                    .collect();
                TableGroup {
                    label: group_label(&summary.filter),
                    summary: vec![
                        format_currency(summary.values[0]),
                        format_currency(summary.values[1]),
                        format_currency(summary.values[2]),
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
            let rules = [GroupKeyRule {
                test: |r: &Mutualfund, t| r.fund_name.to_lowercase().contains(t),
                key_fn: &|r: &Mutualfund| r.fund_name.clone(),
            }];
            let key = create_group_key_fn(query, |r| create_year_month_key(&r.trade_date), &rules);
            group_and_summarize(
                &sorted,
                &key,
                &[
                    |r| r.realized_profit_and_loss,
                    |r| r.taxes,
                    |r| r.realized_profit_and_loss_after_tax,
                ],
                true,
            )
            .into_iter()
            .map(|summary| {
                let group_rows = sorted
                    .iter()
                    .filter(|row| key(row) == summary.filter)
                    .cloned()
                    .map(ReceiptItem::MutualFund)
                    .map(|item| item.cells())
                    .collect();
                TableGroup {
                    label: group_label(&summary.filter),
                    summary: vec![
                        format_currency(summary.values[0]),
                        format_currency(summary.values[1]),
                        format_currency(summary.values[2]),
                    ],
                    rows: group_rows,
                }
            })
            .collect()
        }
    }
}

#[component]
fn ReceiptTable(
    tab: ReceiptsTab,
    rows: Vec<ReceiptItem>,
    all_rows: Vec<ReceiptItem>,
    query: String,
) -> impl IntoView {
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
    let groups = table_groups(tab, &rows, &all_rows, &query);
    let order = column_order(tab, &rows, &query);
    let headers: Vec<_> = order.iter().map(|i| headers[*i]).collect();
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
                                        let cells: Vec<_> = order.iter().map(|i| cells[*i].clone()).collect();
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

#[cfg(test)]
mod tests;
