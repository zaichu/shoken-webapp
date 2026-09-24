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
use crate::receipts_filter::{
    column_order, search_categories, DateSegment, ReceiptSearch, SearchKey,
};
use crate::receipts_search::SearchOption;
use crate::receipts_search_group_key::{create_group_key_fn, GroupKeyRule};
use crate::receipts_search_support::group_and_summarize;
use crate::session::use_session;
use leptos::prelude::*;
use rust_decimal::Decimal;
use std::collections::HashMap;
use wasm_bindgen::JsCast;

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
    let has_years = !categories.years.is_empty();
    let dates = categories.dates;
    let years = categories.years.clone();
    let year_picker_open = RwSignal::new(false);
    if dates
        && !has_years
        && search
            .with_untracked(|state| state.is_default() && state.date_segment == DateSegment::Year)
    {
        search.set(ReceiptSearch::new(false));
    }
    let date_period = if dates {
        view! {
            <div class="mb-3.5">
                <DatePeriod
                    search=search
                    years=years.clone()
                    year_picker_open=year_picker_open
                />
            </div>
        }
        .into_any()
    } else {
        ().into_any()
    };
    let year_dropdown = if !dates && !years.is_empty() {
        view! { <YearDropdown search=search options=years.clone() /> }.into_any()
    } else {
        ().into_any()
    };
    let filtered = Memo::new(move |_| store.filtered_rows(tab));
    let all_rows = data.rows.clone();
    let clear_search = search;
    let clear_picker = year_picker_open;
    view! {
        <section>
            <div
                class="mb-3 rounded-lg border border-slate-200 bg-white p-4"
                role="search"
                aria-label="取引明細の検索"
                data-testid="search-card"
            >
                <div class="mb-3 flex items-center justify-between gap-3">
                    <h2 class="text-sm font-bold text-slate-950">"検索オプション"</h2>
                    <button
                        type="button"
                        class="rounded border border-slate-300 px-3 py-1 text-sm"
                        aria-label="検索条件をクリア"
                        data-testid="search-clear-button"
                        disabled=move || search.with(|state| state.is_default())
                        on:click=move |_| {
                            clear_picker.set(false);
                            clear_search.update(|state| state.clear(has_years));
                        }
                    >"絞り込み解除"</button>
                </div>
                {date_period}
                <div class="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-4">
                    <SecurityDropdown search=search options=categories.securities />
                    {year_dropdown}
                    <ToggleCategory search=search search_key=SearchKey::Products label="商品" options=categories.products />
                    <ToggleCategory search=search search_key=SearchKey::Accounts label="口座" options=categories.accounts />
                </div>
            </div>
            {move || {
                if all_rows.is_empty() {
                    return view! { <div><h3>"データがありません"</h3><p>{empty_hint(tab)}</p></div> }.into_any();
                }
                let query = search.with(|s| s.query.clone());
                let rows = filtered.get();
                let header = header_summary(tab, &ReceiptTabData { rows: rows.clone(), summary: data.summary.clone() }, &query);
                view! {
                    <div class="mb-3 grid grid-cols-1 gap-2 sm:grid-cols-3" aria-label="集計情報">
                        {header.into_iter().map(|(label, value)| view! {
                            <div class="rounded-lg border border-slate-200 bg-white px-4 py-3">
                                <div class="text-xs font-semibold text-slate-600">{label}</div>
                                <div class="mt-1 text-right font-mono text-lg font-bold tabular-nums">{format_currency(value)}</div>
                            </div>
                        }).collect_view()}
                    </div>
                    <ReceiptTable tab=tab rows=rows all_rows=all_rows.clone() query=query />
                }.into_any()
            }}
        </section>
    }
}

#[component]
fn SecurityDropdown(search: RwSignal<ReceiptSearch>, options: Vec<SearchOption>) -> impl IntoView {
    if options.is_empty() {
        return ().into_any();
    }
    view! {
        <div>
            <label class="mb-1 block text-sm font-bold text-slate-800" for="securities-search">"銘柄"</label>
            <select id="securities-search" class="w-full rounded-md border border-slate-300 bg-white px-3 py-2 text-sm"
                prop:value=move || search.with(|state| state.selected_queries.securities.clone())
                on:change=move |event| search.update(|state| state.select_quick(SearchKey::Securities, event_target_value(&event)))>
                <option value="">"全て表示"</option>
                {options.into_iter().map(|o| view! { <option value=o.value>{o.label}</option> }).collect_view()}
            </select>
        </div>
    }.into_any()
}

#[component]
fn YearDropdown(search: RwSignal<ReceiptSearch>, options: Vec<SearchOption>) -> impl IntoView {
    if options.is_empty() {
        return ().into_any();
    }
    view! {
        <div>
            <label class="mb-1 block text-sm font-bold text-slate-800" for="years-search">"西暦"</label>
            <select
                id="years-search"
                class="w-full rounded-md border border-slate-300 bg-white px-3 py-2 text-sm"
                prop:value=move || search.with(|state| state.selected_queries.years.clone())
                on:change=move |event| search.update(|state| {
                    state.select_quick(SearchKey::Years, event_target_value(&event))
                })
            >
                <option value="">"全て表示"</option>
                {options.into_iter().map(|option| view! { <option value=option.value>{option.label}</option> }).collect_view()}
            </select>
        </div>
    }
    .into_any()
}

#[component]
fn ToggleCategory(
    search: RwSignal<ReceiptSearch>,
    search_key: SearchKey,
    label: &'static str,
    options: Vec<SearchOption>,
) -> impl IntoView {
    if options.is_empty() {
        return ().into_any();
    }
    view! {
        <div>
            <div class="mb-1 text-sm font-bold text-slate-800">{label}</div>
            <div class="flex flex-wrap gap-1">
                {options.into_iter().map(|option| {
                    let selected_value = option.value.clone();
                    let aria_value = option.value.clone();
                    let clicked_value = option.value.clone();
                    view! {
                        <button
                            type="button"
                            class=move || if search.with(|state| state.selected_queries.get(search_key) == selected_value) {
                                "rounded border border-amber-500 bg-amber-50 px-3 py-1.5 text-sm font-bold text-amber-900"
                            } else {
                                "rounded border border-slate-300 bg-white px-3 py-1.5 text-sm text-slate-700"
                            }
                            aria-pressed=move || search.with(|state| state.selected_queries.get(search_key) == aria_value)
                            aria-label=move || search.with(|state| {
                                if state.selected_queries.get(search_key) == option.value {
                                    format!("{}（選択中）", option.value)
                                } else {
                                    option.value.clone()
                                }
                            })
                            on:click=move |_| search.update(|state| state.select_quick(search_key, clicked_value.clone()))
                        >
                            {option.label}
                        </button>
                    }
                }).collect_view()}
            </div>
        </div>
    }.into_any()
}

#[derive(Clone, Copy)]
enum DateInputField {
    Month,
    Date,
    RangeStart,
    RangeEnd,
}

fn date_input_value(state: &ReceiptSearch, field: DateInputField) -> String {
    match field {
        DateInputField::Month => state.date_inputs.month_value.clone(),
        DateInputField::Date => state.date_inputs.date_value.clone(),
        DateInputField::RangeStart => state.date_inputs.range_start.clone(),
        DateInputField::RangeEnd => state.date_inputs.range_end.clone(),
    }
}

fn date_input_type(field: DateInputField) -> &'static str {
    match field {
        DateInputField::Month => "month",
        DateInputField::Date | DateInputField::RangeStart | DateInputField::RangeEnd => "date",
    }
}

fn format_date_input_label(value: &str, fallback: &str) -> String {
    if value.is_empty() {
        fallback.to_string()
    } else {
        value.replace('-', "/")
    }
}

fn visible_date_segment(state: &ReceiptSearch, has_years: bool) -> DateSegment {
    if !has_years && state.date_segment == DateSegment::Year {
        DateSegment::Month
    } else {
        state.date_segment
    }
}

#[component]
fn CalendarDateButton(
    search: RwSignal<ReceiptSearch>,
    field: DateInputField,
    label: &'static str,
) -> impl IntoView {
    let input_ref = NodeRef::<leptos::html::Input>::new();
    let picker_ref = input_ref;
    let input_type = date_input_type(field);
    let value_prop = move || search.with(|state| date_input_value(state, field));
    let button_class = move || {
        if search
            .with(|state| date_input_value(state, field))
            .is_empty()
        {
            "w-full rounded-md border border-slate-300 bg-white px-3 py-2 text-left text-sm text-slate-500 transition-colors hover:border-slate-400"
        } else {
            "w-full rounded-md border border-amber-500 bg-amber-50 px-3 py-2 text-left text-sm font-semibold text-amber-900 transition-colors"
        }
    };
    let display_value = move || {
        let value = search.with(|state| date_input_value(state, field));
        format_date_input_label(&value, label)
    };
    let on_change = move |event| {
        let value = event_target_value(&event);
        search.update(|state| match field {
            DateInputField::Month => state.set_month(value),
            DateInputField::Date => state.set_date(value),
            DateInputField::RangeStart => state.set_range_start(value),
            DateInputField::RangeEnd => state.set_range_end(value),
        });
    };
    view! {
        <div class="relative">
            <button
                type="button"
                class=button_class
                on:click=move |_| {
                    if let Some(input) = picker_ref.get() {
                        if input.show_picker().is_err() {
                            input.click();
                        }
                    }
                }
            >
                {display_value}
            </button>
            <input
                node_ref=input_ref
                type=input_type
                prop:value=value_prop
                aria-hidden="true"
                tabindex="-1"
                class="pointer-events-none absolute h-px w-px overflow-hidden opacity-0"
                on:change=on_change
            />
        </div>
    }
}

fn next_year_option_index(current: Option<usize>, option_count: usize, key: &str) -> Option<usize> {
    if option_count == 0 {
        return None;
    }
    match key {
        "ArrowDown" | "ArrowRight" => Some(current.map_or(0, |i| (i + 1) % option_count)),
        "ArrowUp" | "ArrowLeft" => {
            Some(current.map_or(option_count - 1, |i| (i + option_count - 1) % option_count))
        }
        "Home" => Some(0),
        "End" => Some(option_count - 1),
        _ => None,
    }
}

fn focus_year_option(index: usize) {
    let Some(element) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id(&format!("receipts-year-option-{index}")))
    else {
        return;
    };
    if let Some(option) = element.dyn_ref::<web_sys::HtmlElement>() {
        let _ = option.focus();
    }
}

#[component]
fn YearPicker(
    search: RwSignal<ReceiptSearch>,
    years: Vec<SearchOption>,
    is_open: RwSignal<bool>,
) -> impl IntoView {
    let option_count = years.len();
    let trigger_ref = NodeRef::<leptos::html::Button>::new();
    let label_search = search;
    let label_years = years.clone();
    let selected_label = move || {
        let value = label_search.with(|state| state.date_inputs.year_value.clone());
        if value.is_empty() {
            "年を選択".to_string()
        } else {
            label_years
                .iter()
                .find(|year| year.value == value)
                .map(|year| year.label.clone())
                .unwrap_or(value)
        }
    };
    let toggle_search = search;
    let toggle_open = is_open;
    let options = years.clone();
    view! {
        <div class="relative">
            <button
                node_ref=trigger_ref
                type="button"
                aria-label="年を選択"
                aria-haspopup="listbox"
                aria-expanded=move || is_open.get()
                class=move || if search.with(|state| state.date_inputs.year_value.is_empty()) {
                    "w-full rounded-md border border-slate-300 bg-white px-3 py-2 text-left text-sm text-slate-500 transition-colors hover:border-slate-400"
                } else {
                    "w-full rounded-md border border-amber-500 bg-amber-50 px-3 py-2 text-left text-sm font-semibold text-amber-900 transition-colors"
                }
                on:click=move |_| toggle_open.update(|open| *open = !*open)
                on:keydown=move |event| {
                    if !is_open.get() {
                        return;
                    }
                    let index = match event.key().as_str() {
                        "ArrowDown" | "ArrowRight" => Some(0),
                        "ArrowUp" | "ArrowLeft" => option_count.checked_sub(1),
                        "Escape" => {
                            event.prevent_default();
                            is_open.set(false);
                            None
                        }
                        _ => None,
                    };
                    if let Some(index) = index {
                        event.prevent_default();
                        focus_year_option(index);
                    }
                }
            >
                {selected_label}
            </button>
            {move || {
                if is_open.get() {
                    view! {
                        <div
                            role="listbox"
                            aria-label="年候補"
                            class="absolute z-10 w-full grid grid-cols-3 gap-1 rounded-b-md border border-t-0 border-slate-300 bg-white px-2 pb-2 pt-1.5"
                        >
                            {options
                                .clone()
                                .into_iter()
                                .enumerate()
                                .map(|(index, year)| {
                                    let value = year.value.clone();
                                    let selected_value = year.value.clone();
                                    let class_value = value.clone();
                                    let click_search = toggle_search;
                                    let click_open = toggle_open;
                                    let keydown_open = toggle_open;
                                    view! {
                                        <button
                                            id={format!("receipts-year-option-{index}")}
                                            type="button"
                                            role="option"
                                            aria-selected=move || click_search.with(|state| state.date_inputs.year_value == selected_value)
                                            class=move || if click_search.with(|state| state.date_inputs.year_value == class_value) {
                                                "rounded bg-amber-50 px-1 py-1.5 text-center text-sm font-semibold text-amber-900 ring-1 ring-inset ring-amber-400"
                                            } else {
                                                "rounded px-1 py-1.5 text-center text-sm text-slate-600 hover:bg-slate-100 hover:text-slate-900"
                                            }
                                            on:click=move |_| {
                                                click_open.set(false);
                                                click_search.update(|state| state.select_year(value.clone()));
                                            }
                                            on:keydown=move |event| {
                                                let key = event.key();
                                                if key == "Escape" {
                                                    event.prevent_default();
                                                    keydown_open.set(false);
                                                    if let Some(trigger) = trigger_ref.get() {
                                                        let _ = trigger.focus();
                                                    }
                                                } else if let Some(next) =
                                                    next_year_option_index(Some(index), option_count, &key)
                                                {
                                                    event.prevent_default();
                                                    focus_year_option(next);
                                                }
                                            }
                                        >
                                            {year.label}
                                        </button>
                                    }
                                })
                                .collect_view()}
                        </div>
                    }
                    .into_any()
                } else {
                    ().into_any()
                }
            }}
        </div>
    }
}

#[component]
fn DatePeriod(
    search: RwSignal<ReceiptSearch>,
    years: Vec<SearchOption>,
    year_picker_open: RwSignal<bool>,
) -> impl IntoView {
    let has_years = !years.is_empty();
    let segments = [
        DateSegment::Year,
        DateSegment::Month,
        DateSegment::Date,
        DateSegment::Range,
    ];
    view! {
        <div class="space-y-2">
            <div class="text-sm font-bold text-slate-800">"期間"</div>
            <div class="flex gap-1">
                {segments
                    .into_iter()
                    .filter(|segment| has_years || *segment != DateSegment::Year)
                    .map(|segment| {
                        let click_search = search;
                        let click_picker = year_picker_open;
                        view! {
                            <button
                                type="button"
                                class=move || if search.with(|state| visible_date_segment(state, has_years) == segment) {
                                    "flex-1 rounded bg-slate-950 px-2 py-1 text-xs font-semibold text-white"
                                } else {
                                    "flex-1 rounded bg-slate-100 px-2 py-1 text-xs font-semibold text-slate-600"
                                }
                                aria-pressed=move || search.with(|state| visible_date_segment(state, has_years) == segment)
                                on:click=move |_| {
                                    click_picker.set(false);
                                    click_search.update(|state| state.change_date_segment(segment));
                                }
                            >
                                {segment.label()}
                            </button>
                        }
                    })
                    .collect_view()}
            </div>
            {move || match search.with(|state| visible_date_segment(state, has_years)) {
                DateSegment::Year => view! {
                    <YearPicker
                        search=search
                        years=years.clone()
                        is_open=year_picker_open
                    />
                }
                .into_any(),
                DateSegment::Month => view! {
                    <CalendarDateButton search=search field=DateInputField::Month label="月を選択" />
                }
                .into_any(),
                DateSegment::Date => view! {
                    <CalendarDateButton search=search field=DateInputField::Date label="日を選択" />
                }
                .into_any(),
                DateSegment::Range => view! {
                    <div class="flex flex-col gap-2">
                        <CalendarDateButton search=search field=DateInputField::RangeStart label="開始日" />
                        <CalendarDateButton search=search field=DateInputField::RangeEnd label="終了日" />
                    </div>
                }
                .into_any(),
            }}
        </div>
    }
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
