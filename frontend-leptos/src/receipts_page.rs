use crate::collapsible_search_card::{is_narrow_viewport, CollapsibleSearchCard};
use crate::confirm_modal::ConfirmDeleteModal;
use crate::csv_flow::row_error_text;
use crate::csv_rail::CsvActionRail;
use crate::dividend_info::{search_security_code, DividendInfoStore, DividendSummarySection};
use crate::dto::{Dividend, Mutualfund};
use crate::receipts::{
    select_header_summary, truncated_list_warning, use_receipts_data, ReceiptCell, ReceiptItem,
    ReceiptSummary, ReceiptTabData, ReceiptsStore, ReceiptsTab, TabState,
};
use crate::receipts_csv::CsvPreviewRow;
use crate::receipts_domain::{
    calculate_dividends, calculate_domestic_daily, calculate_domestic_total,
    calculate_mutual_funds, create_year_month_key, format_currency, sort_dividends,
    sort_domestic_stocks, sort_mutual_funds,
};
use crate::receipts_filter::{
    column_order, filter_receipts, search_categories, DateSegment, ReceiptSearch, SearchKey,
};
use crate::receipts_search::SearchOption;
use crate::receipts_search_group_key::{create_group_key_fn, GroupKeyRule};
use crate::receipts_search_support::group_and_summarize;
use crate::security_link::{copy_to_clipboard, CopyableInstrumentName, SecurityCodeLink};
use crate::session::use_session;
use crate::ui::{Loading, PageHeader, Spinner};
use leptos::ev;
use leptos::prelude::*;
use rust_decimal::Decimal;
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet, VecDeque};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

const TAB_IDS: [&str; 3] = ["dividend", "domesticstock", "mutualfund"];

#[component]
pub fn ReceiptsPage() -> impl IntoView {
    let store = use_receipts_data(use_session(), ReceiptsTab::Dividend);
    let busy = store.clone();
    let panels_store = store.clone();
    let modal_store = store.clone();
    // 他タブの取得進捗で workspace 全体を再生成するとレール開閉などのローカル状態が
    // 巻き戻るため、分岐条件だけを memo 化して再生成を実際の切替時に限定する
    let workspace_store = store.clone();
    let panels_loading = Memo::new(move |_| {
        workspace_store.auth_loading()
            || matches!(
                workspace_store.tab_state(workspace_store.active_tab.get()),
                TabState::Loading
            )
    });
    let tabs = store.clone();
    // クリックとキー操作の両経路をカバーするため select_tab ではなく active_tab の変化に追従する
    Effect::new(move |_| scroll_tab_into_view(tabs.active_tab.get()));

    view! {
        <PageHeader
            title="取引明細"
            eyebrow="Transactions"
            description="配当金・国内株式・投資信託の取引明細を管理します。"
        />
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
        <div
            class="mt-0"
            aria-busy=move || {
                busy.auth_loading()
                    || busy.any_tab_fetching()
                    || ReceiptsTab::ALL.iter().any(|tab| {
                        let state = busy.csv_state(*tab);
                        state.saving || state.deleting
                    })
            }
        >
            <div data-testid="receipts-workspace">
                {move || {
                    let workspace = panels_store.clone();
                    if panels_loading.get() {
                        view! {
                            <section class="px-5 py-4" role="status">
                                <div class="flex items-center gap-2 text-slate-600">
                                    <Spinner size="sm" class="" />
                                    <p class="text-sm">"データを読み込んでいます..."</p>
                                </div>
                            </section>
                        }
                            .into_any()
                    } else {
                        view! {
                            {ReceiptsTab::ALL
                                .iter()
                                .copied()
                                .map(|tab| {
                                    view! { <TabPanel store=workspace.clone() tab=tab /> }
                                })
                                .collect_view()}
                        }
                            .into_any()
                    }
                }}
            </div>
            {move || {
                let tab = modal_store.active_tab.get();
                if !modal_store.csv_state(tab).show_delete_confirm {
                    return ().into_any();
                }
                let count = modal_store.count(tab);
                let deleting_store = modal_store.clone();
                let deleting = Memo::new(move |_| deleting_store.csv_state(tab).deleting);
                let confirm = modal_store.clone();
                let cancel = modal_store.clone();
                view! {
                    <ConfirmDeleteModal
                        title=format!("{}データの全件削除", tab.label())
                        description=format!("【{}】のデータをすべて削除します。", tab.label())
                        item_count=count
                        confirm_label="削除する"
                        loading=deleting
                        on_confirm=move || confirm.confirm_delete_all(tab)
                        on_cancel=move || cancel.close_delete_confirm(tab)
                    />
                }
                    .into_any()
            }}
        </div>
    }
}

const TAB_BUTTON_BASE: &str = "inline-flex items-center gap-2 rounded-lg border px-4 py-2.5 text-sm font-bold max-sm:min-h-[44px] max-sm:shrink-0 max-sm:px-3";
const TAB_BUTTON_ACTIVE: &str =
    "border-slate-950 bg-slate-950 text-white shadow-[inset_0_-2px_0_#f59e0b]";
const TAB_BUTTON_INACTIVE: &str =
    "border-slate-300 bg-white text-slate-800 hover:border-slate-400 hover:bg-white hover:text-slate-950";
const TAB_COUNT_BASE: &str =
    "inline-flex min-w-6 items-center justify-center rounded-full px-2 py-0.5 text-xs font-semibold";
const TAB_COUNT_ACTIVE: &str = "border border-white/20 bg-white text-slate-950";
const TAB_COUNT_INACTIVE: &str = "border border-slate-200 bg-white text-slate-700";

fn next_tab_index(current: usize, key: &str) -> Option<usize> {
    let count = TAB_IDS.len();
    match key {
        "ArrowRight" => Some((current + 1) % count),
        "ArrowLeft" => Some((current + count - 1) % count),
        "Home" => Some(0),
        "End" => Some(count - 1),
        _ => None,
    }
}

fn focus_tab(index: usize) {
    let Some(element) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id(&format!("tab-{}", TAB_IDS[index])))
    else {
        return;
    };
    if let Some(element) = element.dyn_ref::<web_sys::HtmlElement>() {
        let _ = element.focus();
    }
}

fn scroll_tab_into_view(tab: ReceiptsTab) {
    let Some(element) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id(&format!("tab-{}", TAB_IDS[tab as usize])))
    else {
        return;
    };
    let options = web_sys::ScrollIntoViewOptions::new();
    options.set_block(web_sys::ScrollLogicalPosition::Nearest);
    options.set_inline(web_sys::ScrollLogicalPosition::Nearest);
    element.scroll_into_view_with_scroll_into_view_options(&options);
}

#[component]
fn TabButton(store: ReceiptsStore, tab: ReceiptsTab) -> impl IntoView {
    let slug = TAB_IDS[tab as usize];
    let label = tab.label();
    let selected = store.clone();
    let keyed = store.clone();
    let clicked = store.clone();
    let counted = store.clone();
    let counted_store = store.clone();
    view! {
        <button
            id={format!("tab-{slug}")}
            type="button"
            role="tab"
            class=move || {
                format!(
                    "{TAB_BUTTON_BASE} {}",
                    if selected.active_tab.get() == tab {
                        TAB_BUTTON_ACTIVE
                    } else {
                        TAB_BUTTON_INACTIVE
                    }
                )
            }
            aria-selected=move || {
                if selected.active_tab.get() == tab {
                    "true"
                } else {
                    "false"
                }
            }
            aria-controls={format!("tabpanel-{slug}")}
            tabindex=move || if selected.active_tab.get() == tab { "0" } else { "-1" }
            on:click=move |_| clicked.select_tab(tab)
            on:keydown=move |event| {
                let current = ReceiptsTab::ALL
                    .iter()
                    .position(|item| *item == keyed.active_tab.get_untracked())
                    .unwrap_or(0);
                if let Some(next) = next_tab_index(current, &event.key()) {
                    event.prevent_default();
                    keyed.select_tab(ReceiptsTab::ALL[next]);
                    focus_tab(next);
                }
            }
        >
            {label}
            <span
                data-testid={format!("tab-count-{slug}")}
                class=move || {
                    format!(
                        "{TAB_COUNT_BASE} {}",
                        if counted.active_tab.get() == tab {
                            TAB_COUNT_ACTIVE
                        } else {
                            TAB_COUNT_INACTIVE
                        }
                    )
                }
            >
                {move || counted_store.count(tab).to_string()}
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
                    return view! { <Loading /> }.into_any();
                }
                view! { <ReceiptWorkspace store=store tab=tab /> }.into_any()
            }}
        </div>
    }
}

fn empty_tab_data() -> ReceiptTabData {
    ReceiptTabData {
        rows: Vec::new(),
        summary: None,
        truncated: false,
    }
}

#[component]
fn ReceiptWorkspace(store: ReceiptsStore, tab: ReceiptsTab) -> impl IntoView {
    let csv_store = store.clone();
    let preview_store = store.clone();
    let loading_store = store.clone();
    let rail_store = store.clone();
    let main_store = store.clone();
    let alert_store = store.clone();
    // cache は全タブ共有の1 signal なので、他タブの取得進捗でも評価自体は走る。
    // memo で実際にこのタブの状態が変わった時だけビューを再生成させる
    let panel_state = Memo::new(move |_| store.tab_state(tab));
    view! {
        // DOM 順は rail 先(キーボード・読み上げ順のため)、lg 以上は order で見た目を main 先に戻す
        <div
            class="grid gap-3 sm:gap-4 lg:grid-cols-[minmax(0,1fr)_19rem] lg:items-start xl:gap-5 xl:grid-cols-[minmax(0,1fr)_20rem]"
            data-testid="receipt-workspace"
        >
            <aside class="order-1 lg:order-2" data-testid="receipt-utility-rail">
                // スマホでは帯と別カードの積み上げを維持するため枠は sm 以上だけにする
                // 年ピッカーのドロップダウンを切らないよう overflow は掛けない。
                // backdrop-blur が作る stack context に listbox が閉じ込められるため、1カラム幅でも表より前面に出す
                <div class="sm:relative sm:z-20 sm:divide-y sm:divide-slate-950/10 sm:rounded-xl sm:border sm:border-slate-950/10 sm:bg-white/90 sm:shadow-[0_18px_58px_-42px_rgba(15,23,42,0.9)] sm:backdrop-blur-sm">
                    <ReceiptsCsvSection store=csv_store.clone() tab=tab />
                    {move || {
                        let Some(message) = alert_store.rail_error(tab)
                        else {
                            return ().into_any();
                        };
                        view! {
                            <section class="px-5 py-4">
                                <div
                                    class="rounded-lg border border-red-200 bg-red-50 px-4 py-3 text-sm font-medium text-red-800"
                                    role="alert"
                                    aria-live="assertive"
                                >
                                    <strong>"エラー:"</strong>
                                    " "
                                    {message}
                                </div>
                            </section>
                        }
                            .into_any()
                    }}
                    {move || {
                        match panel_state.get() {
                            TabState::Ready(data) if data.truncated => {
                                view! {
                                    <section class="px-5 py-4" role="status" aria-live="polite">
                                        <div class="rounded-lg border border-amber-200 bg-amber-50 px-4 py-3 text-sm font-medium text-amber-900">
                                            {truncated_list_warning()}
                                        </div>
                                    </section>
                                }
                                    .into_any()
                            }
                            _ => ().into_any(),
                        }
                    }}
                    {move || {
                        let state = preview_store.csv_state(tab);
                        let authenticated = preview_store.is_authenticated();
                        let has_file = state.file_name.is_some();
                        let previewing = state.previewing;
                        let Some(preview) = state
                            .preview
                            .filter(|_| authenticated && has_file && !previewing)
                        else {
                            return ().into_any();
                        };
                        let has_errors = !preview.errors.is_empty();
                        let alert_class = if has_errors {
                            "border-amber-200 bg-amber-50 text-amber-900"
                        } else {
                            "border-blue-200 bg-blue-50 text-blue-800"
                        };
                        view! {
                            <section class="px-5 py-4" role="status" aria-live="polite">
                                <div class=format!(
                                    "rounded-lg border px-4 py-3 text-sm font-medium shadow-sm {alert_class}"
                                )>
                                    <p>
                                        <strong>{format!("{}件 追加で保存されます", preview.valid_rows)}</strong>
                                        {has_errors.then(|| format!(" / {}件エラー", preview.errors.len()))}
                                        <span class="ml-2 text-xs text-secondary">"（保存モード: 追加）"</span>
                                    </p>
                                    {has_errors.then(|| {
                                        view! {
                                            <ul class="mt-2 list-disc list-inside text-sm space-y-1">
                                                {preview
                                                    .errors
                                                    .iter()
                                                    .map(|error| view! { <li>{row_error_text(error)}</li> })
                                                    .collect_view()}
                                            </ul>
                                        }
                                    })}
                                </div>
                            </section>
                        }
                            .into_any()
                    }}
                    {move || {
                        let auth_loading = loading_store.auth_loading();
                        let fetching = loading_store.any_tab_fetching();
                        if !auth_loading && !fetching {
                            return ().into_any();
                        }
                        view! {
                            <div aria-live="polite" aria-atomic="true">
                                <section class="px-5 py-4" role="status">
                                    <div class="flex items-center gap-2 text-slate-600">
                                        <Spinner size="sm" class="" />
                                        <p class="text-sm">
                                            {auth_loading.then_some("認証状態を確認しています...")}
                                            {fetching.then_some("データを読み込んでいます...")}
                                        </p>
                                    </div>
                                </section>
                            </div>
                        }
                            .into_any()
                    }}
                    {move || match panel_state.get() {
                        TabState::Ready(data) => {
                            view! { <ReceiptsSearchCard store=rail_store.clone() tab=tab data=data /> }
                                .into_any()
                        }
                        TabState::Failed(_) => {
                            view! {
                                <ReceiptsSearchCard
                                    store=rail_store.clone()
                                    tab=tab
                                    data=empty_tab_data()
                                />
                            }
                                .into_any()
                        }
                        _ => ().into_any(),
                    }}
                </div>
            </aside>
            <div class="min-w-0 order-2 lg:order-1" data-testid="receipt-main-stage">
                {move || match panel_state.get() {
                    TabState::Loading => view! { <Loading /> }.into_any(),
                    TabState::Ready(data) => {
                        view! { <ReceiptsMainContent store=main_store.clone() tab=tab data=data /> }
                            .into_any()
                    }
                    TabState::Failed(_) => {
                        view! {
                            <ReceiptsMainContent
                                store=main_store.clone()
                                tab=tab
                                data=empty_tab_data()
                            />
                        }
                            .into_any()
                    }
                }}
            </div>
        </div>
    }
}

#[component]
fn ReceiptsCsvSection(store: ReceiptsStore, tab: ReceiptsTab) -> impl IntoView {
    let input_id = match tab {
        ReceiptsTab::Dividend => "csv-file-input-dividend",
        ReceiptsTab::DomesticStock => "csv-file-input-domesticstock",
        ReceiptsTab::MutualFund => "csv-file-input-mutualfund",
    };
    let selected = store.clone();
    let selected_file_name =
        Memo::new(move |_| selected.csv_state(tab).file_name.unwrap_or_default());
    let disabled_store = store.clone();
    let file_input_disabled = Memo::new(move |_| {
        disabled_store.auth_loading()
            || !disabled_store.is_authenticated()
            || disabled_store.csv_busy(tab)
            || disabled_store.any_tab_fetching()
    });
    let has_file = store.clone();
    let has_csv_file = Memo::new(move |_| {
        has_file.is_authenticated() && has_file.csv_state(tab).file_name.is_some()
    });
    let label_store = store.clone();
    let save_label = Memo::new(move |_| label_store.csv_state(tab).save_label("追加で保存"));
    let save_dis = store.clone();
    let save_disabled = Memo::new(move |_| save_dis.csv_busy(tab));
    let has_db = store.clone();
    let has_db_data = Memo::new(move |_| has_db.is_authenticated() && has_db.count(tab) > 0);
    let del_label = store.clone();
    let delete_label =
        Memo::new(move |_| del_label.csv_state(tab).delete_label(del_label.count(tab)));
    let del_dis = store.clone();
    let delete_disabled = Memo::new(move |_| {
        let state = del_dis.csv_state(tab);
        state.saving || state.deleting || del_dis.any_tab_fetching()
    });
    let result_store = store.clone();
    let save_result = Memo::new(move |_| result_store.csv_state(tab).import_result);
    let file_select = store.clone();
    let save = store.clone();
    let delete_request = store.clone();
    view! {
        <CsvActionRail
            input_id=input_id
            on_file_select=move |file| file_select.select_file(tab, file)
            selected_file_name=selected_file_name
            file_input_disabled=file_input_disabled
            has_csv_file=has_csv_file
            save_label=save_label
            on_save=move || save.save_csv(tab)
            save_disabled=save_disabled
            has_db_data=has_db_data
            delete_label=delete_label
            on_delete_request=move || delete_request.open_delete_confirm(tab)
            delete_disabled=delete_disabled
            save_result=save_result
            mode_label="追加保存"
            section_class="sm:rounded-t-xl"
        />
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
fn ReceiptsSearchCard(
    store: ReceiptsStore,
    tab: ReceiptsTab,
    data: ReceiptTabData,
) -> impl IntoView {
    let search = store.search;
    let display_store = store.clone();
    let display_rows = Memo::new(move |_| match display_store.csv_state(tab).preview {
        Some(preview) if !preview.rows.is_empty() => preview
            .rows
            .iter()
            .map(CsvPreviewRow::to_receipt_item)
            .collect(),
        _ => data.rows.clone(),
    });
    let categories = Memo::new(move |_| search_categories(tab, &display_rows.get()));
    let securities = Memo::new(move |_| categories.with(|c| c.securities.clone()));
    let products = Memo::new(move |_| categories.with(|c| c.products.clone()));
    let accounts = Memo::new(move |_| categories.with(|c| c.accounts.clone()));
    let years = Memo::new(move |_| categories.with(|c| c.years.clone()));
    let has_years = move || categories.with(|c| !c.years.is_empty());
    let has_dates = move || categories.with(|c| c.dates);
    let year_picker_open = RwSignal::new(false);
    if categories.with_untracked(|c| c.dates)
        && categories.with_untracked(|c| c.years.is_empty())
        && search
            .with_untracked(|state| state.is_default() && state.date_segment == DateSegment::Year)
    {
        search.set(ReceiptSearch::new(false));
    }
    let clear_search = search;
    let clear_picker = year_picker_open;
    let collapse_picker = year_picker_open;
    view! {
        <section
            class="max-sm:rounded-xl max-sm:border max-sm:border-slate-950/10 max-sm:bg-white/90 max-sm:shadow-[0_18px_58px_-42px_rgba(15,23,42,0.9)]"
            role="search"
            aria-label="取引明細の検索"
            data-testid="search-card"
        >
            <CollapsibleSearchCard
                initial_expanded=!is_narrow_viewport()
                has_active_search=Signal::derive(move || {
                    !search.with(|state| state.is_default())
                })
                is_default_state=Signal::derive(move || {
                    search.with(|state| state.is_default())
                })
                on_clear=move || {
                    clear_picker.set(false);
                    clear_search.update(|state| state.clear(has_years()));
                }
                on_expand_toggle=Callback::new(move |(open,): (bool,)| {
                    if !open {
                        collapse_picker.set(false);
                    }
                })
            >
                {move || {
                    if has_dates() {
                        view! {
                            <div class="mb-3.5">
                                <DatePeriod
                                    search=search
                                    years=years
                                    year_picker_open=year_picker_open
                                />
                            </div>
                        }
                            .into_any()
                    } else {
                        ().into_any()
                    }
                }}
                <div class="grid grid-cols-1 gap-3">
                    <SecurityDropdown search=search options=securities />
                    {move || {
                        if !has_dates() && has_years() {
                            view! { <YearDropdown search=search options=years /> }.into_any()
                        } else {
                            ().into_any()
                        }
                    }}
                    <ToggleCategory search=search search_key=SearchKey::Products label="商品" options=products />
                    <ToggleCategory search=search search_key=SearchKey::Accounts label="口座" options=accounts />
                </div>
            </CollapsibleSearchCard>
        </section>
    }
}

#[component]
fn ReceiptsMainContent(
    store: ReceiptsStore,
    tab: ReceiptsTab,
    data: ReceiptTabData,
) -> impl IntoView {
    let search = store.search;
    let summary = data.summary.clone();
    let display_store = store.clone();
    let display_rows = Memo::new(move |_| match display_store.csv_state(tab).preview {
        Some(preview) if !preview.rows.is_empty() => preview
            .rows
            .iter()
            .map(CsvPreviewRow::to_receipt_item)
            .collect(),
        _ => data.rows.clone(),
    });
    let preview_store = store.clone();
    let preview_active = Memo::new(move |_| preview_store.has_csv_preview(tab));
    let filtered =
        Memo::new(move |_| filter_receipts(tab, &display_rows.get(), &search.get().query));
    // 配当タブの銘柄コード検索時に DividendInfo を出すための取得状態。
    // フィルタ結果・クエリ・セッション変化に追随して対象コードを更新する。
    let session = use_session();
    let dividend_info = (tab == ReceiptsTab::Dividend).then(|| DividendInfoStore::new(session));
    let summary_expanded = RwSignal::new(true);
    if let Some(info) = dividend_info {
        let filtered_rows = filtered;
        let search_query = search;
        Effect::new(move |_| {
            let authenticated = session.user.get().is_some();
            let generation = session.generation.get();
            let query = search_query.with(|state| state.query.clone());
            let rows = filtered_rows.get();
            let dividends: Vec<Dividend> = rows
                .iter()
                .filter_map(|item| match item {
                    ReceiptItem::Dividend(row) => Some(row.clone()),
                    _ => None,
                })
                .collect();
            let code = search_security_code(&dividends, &query);
            info.set_code(generation, authenticated, &code);
        });

        // 別タブで更新された保有数を取り込むため、タブ復帰時に再取得する
        let refresh = move || {
            if web_sys::window()
                .and_then(|window| window.document())
                .is_some_and(|document| !document.hidden())
            {
                info.refresh_balance();
            }
        };
        let on_visible = window_event_listener(
            ev::Custom::<web_sys::Event>::new("visibilitychange"),
            move |_| refresh(),
        );
        let on_focus = window_event_listener(ev::focus, move |_| refresh());
        on_cleanup(move || {
            on_visible.remove();
            on_focus.remove();
        });
    }
    view! {
        {move || {
            let display = display_rows.get();
            if display.is_empty() {
                return view! {
                    <div
                        class="overflow-hidden rounded-xl border border-slate-950/10 bg-white/95 shadow-[0_16px_44px_-36px_rgba(15,23,42,0.9)]"
                        data-testid="receipt-card"
                    >
                        <div class="p-0" data-testid="receipt-card-body">
                            <div class="flex flex-col items-center justify-center rounded-xl border border-dashed border-slate-300 bg-slate-50/70 px-4 py-8 text-center">
                                <h3 class="text-base font-black text-slate-950">
                                    "データがありません"
                                </h3>
                                <p class="mt-1.5 max-w-md text-sm font-medium text-slate-600">
                                    {empty_hint(tab)}
                                </p>
                            </div>
                        </div>
                    </div>
                }
                    .into_any();
            }
            let query = search.with(|s| s.query.clone());
            let rows = filtered.get();
            // 銘柄コード検索時は上段の集計カードの代わりに
            // React の DividendInfo（embedded）を折り畳み式で出す
            if let Some(info) = dividend_info {
                let dividends: Vec<Dividend> = rows
                    .iter()
                    .filter_map(|item| match item {
                        ReceiptItem::Dividend(row) => Some(row.clone()),
                        _ => None,
                    })
                    .collect();
                if !search_security_code(&dividends, &query).is_empty() {
                    let totals = calculate_dividends(&dividends);
                    return view! {
                        <DividendSummarySection
                            store=info
                            totals=totals
                            expanded=summary_expanded
                            mobile_expanded=store.mobile_summary_expanded
                        />
                        <ReceiptTable
                            tab=tab
                            rows=rows
                            all_rows=display
                            query=query
                            expanded_ids=store.expanded
                        />
                    }
                    .into_any();
                }
            }
            let header = header_summary(
                tab,
                &ReceiptTabData {
                    rows: rows.clone(),
                    summary: summary.clone(),
                    truncated: false,
                },
                &query,
                preview_active.get(),
            );
            view! {
                <SummaryStrip items=header expanded=store.mobile_summary_expanded />
                <ReceiptTable
                    tab=tab
                    rows=rows
                    all_rows=display
                    query=query
                    expanded_ids=store.expanded
                />
            }.into_any()
        }}
    }
}

#[component]
fn SecurityDropdown(
    search: RwSignal<ReceiptSearch>,
    options: Memo<Vec<SearchOption>>,
) -> impl IntoView {
    view! {
        {move || {
            let options = options.get();
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
        }}
    }
}

#[component]
fn YearDropdown(
    search: RwSignal<ReceiptSearch>,
    options: Memo<Vec<SearchOption>>,
) -> impl IntoView {
    view! {
        {move || {
            let options = options.get();
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
        }}
    }
}

#[component]
fn ToggleCategory(
    search: RwSignal<ReceiptSearch>,
    search_key: SearchKey,
    label: &'static str,
    options: Memo<Vec<SearchOption>>,
) -> impl IntoView {
    view! {
        {move || {
            let options = options.get();
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
        }}
    }
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
    years: Memo<Vec<SearchOption>>,
    is_open: RwSignal<bool>,
) -> impl IntoView {
    let option_count = move || years.with_untracked(|years| years.len());
    let trigger_ref = NodeRef::<leptos::html::Button>::new();
    let label_search = search;
    let selected_label = move || {
        let value = label_search.with(|state| state.date_inputs.year_value.clone());
        if value.is_empty() {
            "年を選択".to_string()
        } else {
            years
                .with(|years| {
                    years
                        .iter()
                        .find(|year| year.value == value)
                        .map(|year| year.label.clone())
                })
                .unwrap_or(value)
        }
    };
    let toggle_search = search;
    let toggle_open = is_open;
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
                        "ArrowUp" | "ArrowLeft" => option_count().checked_sub(1),
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
                            class="absolute z-10 w-full grid grid-cols-3 gap-1 rounded-b-md border border-t-0 border-slate-300 bg-white px-2 pb-2 pt-1.5 max-h-72 overflow-y-auto"
                        >
                            {years
                                .get()
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
                                                    next_year_option_index(Some(index), option_count(), &key)
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
    years: Memo<Vec<SearchOption>>,
    year_picker_open: RwSignal<bool>,
) -> impl IntoView {
    let has_years = move || years.with(|years| !years.is_empty());
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
                {move || {
                    segments
                        .into_iter()
                        .filter(|segment| has_years() || *segment != DateSegment::Year)
                        .map(|segment| {
                            let click_search = search;
                            let click_picker = year_picker_open;
                            view! {
                                <button
                                    type="button"
                                    class=move || if search.with(|state| visible_date_segment(state, has_years()) == segment) {
                                        "flex-1 rounded bg-slate-950 px-2 py-1 text-xs font-semibold text-white"
                                    } else {
                                        "flex-1 rounded bg-slate-100 px-2 py-1 text-xs font-semibold text-slate-600"
                                    }
                                    aria-pressed=move || search.with(|state| visible_date_segment(state, has_years()) == segment)
                                    on:click=move |_| {
                                        click_picker.set(false);
                                        click_search.update(|state| state.change_date_segment(segment));
                                    }
                                >
                                    {segment.label()}
                                </button>
                            }
                        })
                        .collect_view()
                }}
            </div>
            {move || match search.with(|state| visible_date_segment(state, has_years())) {
                DateSegment::Year => view! {
                    <YearPicker
                        search=search
                        years=years
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

fn kpi_card_bg(tone: &str) -> &'static str {
    match tone {
        "emerald" => "border-teal-200 bg-teal-50",
        "red" => "border-rose-100 bg-rose-50",
        _ => "border-slate-200 bg-white",
    }
}

fn kpi_value_color(tone: &str) -> &'static str {
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
fn SummaryStrip(
    items: Vec<(&'static str, Decimal, &'static str)>,
    expanded: RwSignal<bool>,
) -> impl IntoView {
    let mobile_expanded = expanded;
    let mobile_body_id = "receipt-summary-mobile-body";
    let primary = items.last().copied();
    let mobile_items = items.clone();
    view! {
        <section
            class="mb-3 rounded-xl border border-slate-950/10 bg-white/95 px-4 py-3 shadow-[0_12px_34px_-30px_rgba(15,23,42,0.85)] max-sm:px-3 max-sm:py-0"
            data-testid="receipt-summary-strip"
        >
            <div class="sm:hidden" data-testid="receipt-summary-compact">
                <button
                    type="button"
                    class="flex min-h-[40px] w-full items-center justify-between gap-3 text-left focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-slate-950"
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

struct TableGroup {
    key: String,
    label: String,
    summary: Vec<String>,
    rows: Vec<(String, String, Vec<ReceiptCell>)>,
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
                    .map(|item| (item.id().to_string(), item.raw_key(), item.cells()))
                    .collect();
                TableGroup {
                    key: summary.filter.clone(),
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
                        .map(|item| (item.id().to_string(), item.raw_key(), item.cells()))
                        .collect();
                    TableGroup {
                        key: summary.filter.clone(),
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
                    .map(|item| (item.id().to_string(), item.raw_key(), item.cells()))
                    .collect();
                TableGroup {
                    key: summary.filter.clone(),
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

#[derive(Clone, Copy)]
struct CardFields {
    name: usize,
    primary: usize,
    date: usize,
    account: usize,
}

fn card_fields(tab: ReceiptsTab) -> CardFields {
    match tab {
        ReceiptsTab::Dividend => CardFields {
            name: 4,
            primary: 9,
            date: 0,
            account: 2,
        },
        ReceiptsTab::DomesticStock => CardFields {
            name: 2,
            primary: 10,
            date: 0,
            account: 3,
        },
        ReceiptsTab::MutualFund => CardFields {
            name: 1,
            primary: 9,
            date: 0,
            account: 2,
        },
    }
}

fn summary_labels(tab: ReceiptsTab) -> [&'static str; 3] {
    match tab {
        ReceiptsTab::Dividend => ["配当金", "税額", "受取額"],
        ReceiptsTab::DomesticStock => ["損益", "税額", "税引後"],
        ReceiptsTab::MutualFund => ["実現損益", "税額", "税引損益"],
    }
}

fn is_negative_text(value: &str) -> bool {
    let normalized: String = value
        .trim()
        .chars()
        .filter(|c| !matches!(c, '¥' | '￥' | '$' | '€' | '£' | ',') && !c.is_whitespace())
        .collect();
    let digits = normalized
        .strip_prefix('-')
        .or_else(|| normalized.strip_prefix('+'))
        .unwrap_or(normalized.as_str());
    let valid = !digits.is_empty()
        && digits
            .split('.')
            .all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()));
    valid && normalized.parse::<f64>().is_ok_and(|n| n < 0.0)
}

// グループ見出しに年月があるため、カード先頭の日付は年を落として MM/DD にする
fn short_date(formatted: &str) -> &str {
    match formatted.split_once('/') {
        Some((year, rest)) if year.len() == 4 && year.bytes().all(|b| b.is_ascii_digit()) => rest,
        _ => formatted,
    }
}

enum CardDetailValue {
    Text { text: String, negative: bool },
    SecurityCode(String),
    CopyName { display: String, copy: String },
}

struct CardDetail {
    label: String,
    value: CardDetailValue,
}

struct CardRowData {
    key: String,
    name: String,
    amount: String,
    amount_negative: bool,
    date: String,
    account: String,
    details: Vec<CardDetail>,
}

fn is_security_code(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'.')
}

fn card_detail_value(index: usize, cells: &[ReceiptCell]) -> CardDetailValue {
    match cells.get(index) {
        Some(ReceiptCell::SecurityCode(raw)) => CardDetailValue::SecurityCode(raw.clone()),
        Some(ReceiptCell::InstrumentName { name, code }) => {
            let trimmed = name.trim();
            let display = if trimmed.is_empty() {
                "-".to_string()
            } else {
                trimmed.to_string()
            };
            let copy = code
                .as_deref()
                .map(crate::receipts_domain::normalize_security_code)
                .filter(|code| !code.is_empty())
                .map(|code| format!("{display}({code})"))
                .unwrap_or_else(|| display.clone());
            CardDetailValue::CopyName { display, copy }
        }
        _ => {
            let text = cells
                .get(index)
                .map(|cell| cell_text(cell).to_string())
                .unwrap_or_default();
            CardDetailValue::Text {
                negative: is_negative_text(&text),
                text,
            }
        }
    }
}

fn cell_text(cell: &ReceiptCell) -> &str {
    match cell {
        ReceiptCell::Text(value) | ReceiptCell::SecurityCode(value) => value,
        ReceiptCell::InstrumentName { name, .. } => name,
    }
}

// id を持たない行(CSVプレビュー等)は、同一内容の行と区別するため一覧内の位置も含めて識別する。
// 表示は数量等を丸めるため、行の同一性は丸め前の raw_key で判定する
fn card_key(slug: &str, id: &str, raw_key: &str, ordinal: usize) -> String {
    if id.is_empty() {
        format!("{slug}:p:{ordinal}:{raw_key}")
    } else {
        format!("{slug}:r:{id}")
    }
}

// 絞り込みや並べ替えで表示位置が変わっても同じ行を同じカードキーへ対応させるため、
// id を持たない行の通し番号は絞り込み前の全行内での位置から引く。
fn idless_row_ordinals(all_rows: &[ReceiptItem]) -> HashMap<String, VecDeque<usize>> {
    let mut ordinals: HashMap<String, VecDeque<usize>> = HashMap::new();
    for (index, item) in all_rows.iter().enumerate() {
        if item.id().is_empty() {
            ordinals.entry(item.raw_key()).or_default().push_back(index);
        }
    }
    ordinals
}

fn card_row_data(
    key: String,
    cells: &[ReceiptCell],
    headers: &[&'static str],
    order: &[usize],
    fields: CardFields,
) -> CardRowData {
    let text = |index: usize| {
        cells
            .get(index)
            .map(|cell| cell_text(cell).to_string())
            .unwrap_or_default()
    };
    let amount = text(fields.primary);
    CardRowData {
        key,
        name: text(fields.name),
        amount_negative: is_negative_text(&amount),
        amount,
        date: short_date(&text(fields.date)).to_string(),
        account: text(fields.account),
        details: order
            .iter()
            .map(|&i| CardDetail {
                label: headers[i].to_string(),
                value: card_detail_value(i, cells),
            })
            .collect(),
    }
}

fn card_detail_view(value: &CardDetailValue) -> (AnyView, Option<String>) {
    match value {
        CardDetailValue::Text { text, .. } => {
            (view! { {text.clone()} }.into_any(), Some(text.clone()))
        }
        CardDetailValue::SecurityCode(raw) => {
            let code = crate::receipts_domain::normalize_security_code(raw);
            if code.is_empty() {
                (view! { <span>"-"</span> }.into_any(), None)
            } else if !is_security_code(&code) {
                (view! { <span>{code}</span> }.into_any(), None)
            } else {
                let href = format!("/search?code={code}");
                (
                    view! {
                        <a
                            href=href
                            class="security-code-link text-blue-700 underline-offset-2 hover:text-blue-900 hover:underline focus-visible:rounded-sm focus-visible:outline-2 focus-visible:outline-blue-500 font-bold"
                            data-search=code.clone()
                        >
                            {code.clone()}
                        </a>
                    }
                        .into_any(),
                    None,
                )
            }
        }
        CardDetailValue::CopyName { display, copy } => {
            let copy_text = copy.clone();
            (
                view! {
                    <button
                        type="button"
                        aria-label=format!("{copy} をコピー")
                        on:click=move |_| copy_to_clipboard(copy_text.clone())
                        class="group inline-flex cursor-pointer items-center gap-0.5 border-0 bg-transparent p-0 text-left text-inherit focus-visible:rounded-sm focus-visible:outline-2 focus-visible:outline-blue-500"
                    >
                        <span>{display.clone()}</span>
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            width="12"
                            height="12"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            aria-hidden="true"
                            class="rounded p-0.5 text-gray-400 opacity-100 group-hover:text-gray-600 group-hover:opacity-100 group-focus-visible:opacity-100 sm:opacity-0 sm:group-hover:opacity-100"
                        >
                            <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
                            <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
                        </svg>
                    </button>
                }
                    .into_any(),
                None,
            )
        }
    }
}

#[component]
fn ReceiptItemCard(
    card: CardRowData,
    id_prefix: String,
    expanded_ids: RwSignal<HashSet<String>>,
) -> impl IntoView {
    let expanded_id = card.key.clone();
    let toggle_id = expanded_id.clone();
    let expanded = Memo::new(move |_| expanded_ids.with(|set| set.contains(&expanded_id)));
    let button_id = format!("{id_prefix}-button");
    let details_id = format!("{id_prefix}-details");
    let CardRowData {
        name,
        amount,
        amount_negative,
        date,
        account,
        details,
        ..
    } = card;
    let aria_label = format!("{name} {amount}");
    let amount_class = if amount_negative {
        "min-w-[11ch] shrink-0 whitespace-nowrap text-right font-mono text-base font-semibold tabular-nums text-red-800"
    } else {
        "min-w-[11ch] shrink-0 whitespace-nowrap text-right font-mono text-base font-semibold tabular-nums text-slate-950"
    };
    view! {
        <div data-testid="receipt-card" class="rounded-lg border border-slate-300 bg-white">
            <button
                id=button_id.clone()
                type="button"
                aria-label=aria_label
                aria-expanded=move || if expanded.get() { "true" } else { "false" }
                aria-controls=details_id.clone()
                on:click=move |_| {
                    expanded_ids.update(|set| {
                        if !set.remove(&toggle_id) {
                            set.insert(toggle_id.clone());
                        }
                    })
                }
                class="flex min-h-[90px] w-full flex-col justify-center gap-2 rounded-lg px-3 py-3 text-left focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-blue-700"
            >
                <span class="flex w-full items-baseline gap-2">
                    <span class="min-w-0 flex-1 truncate text-base font-semibold text-slate-950">
                        {name}
                    </span>
                    <span class=amount_class>{amount}</span>
                </span>
                <span
                    class="flex w-full items-center gap-2 text-xs text-slate-600"
                    aria-hidden="true"
                >
                    <span class="shrink-0">{date}</span>
                    <span class="min-w-0 flex-1 truncate">{account}</span>
                    <svg
                        class=move || {
                            if expanded.get() {
                                "h-4 w-4 shrink-0 rotate-180"
                            } else {
                                "h-4 w-4 shrink-0"
                            }
                        }
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        aria-hidden="true"
                    >
                        <path
                            d="m6 9 6 6 6-6"
                            stroke-width="2"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                        />
                    </svg>
                </span>
            </button>
            <div
                id=details_id
                role="region"
                aria-labelledby=button_id
                hidden=move || !expanded.get()
                class="border-t border-slate-300 px-3 py-2"
            >
                {move || {
                    expanded
                        .get()
                        .then(|| {
                            view! {
                                <dl>
                                    {details
                                        .iter()
                                        .map(|detail| {
                                            let negative = matches!(
                                                &detail.value,
                                                CardDetailValue::Text { negative: true, .. }
                                            );
                                            let value_class = if negative {
                                                "min-w-0 break-words text-right text-sm font-semibold tabular-nums text-red-800"
                                            } else {
                                                "min-w-0 break-words text-right text-sm font-semibold tabular-nums text-slate-800"
                                            };
                                            let (content, title) = card_detail_view(&detail.value);
                                            view! {
                                                <div class="flex items-start justify-between gap-3 border-b border-slate-100 py-1.5 last:border-b-0">
                                                    <dt class="shrink-0 pt-0.5 text-xs text-slate-600">
                                                        {detail.label.clone()}
                                                    </dt>
                                                    <dd class=value_class title=title>
                                                        {content}
                                                    </dd>
                                                </div>
                                            }
                                        })
                                        .collect_view()}
                                </dl>
                            }
                        })
                }}
            </div>
        </div>
    }
}

#[component]
fn MobileCardGroup(
    label: String,
    count: usize,
    summary: Vec<(&'static str, String)>,
    cards: Vec<CardRowData>,
    id_prefix: String,
    expanded_key: String,
    expanded_ids: RwSignal<HashSet<String>>,
) -> impl IntoView {
    let expanded_id = expanded_key;
    let toggle_id = expanded_id.clone();
    let expanded = Memo::new(move |_| expanded_ids.with(|set| set.contains(&expanded_id)));
    let button_id = format!("{id_prefix}-group-button");
    let details_id = format!("{id_prefix}-group-details");
    let card_list = view! {
        <div class="mt-2 space-y-2">
            {cards
                .into_iter()
                .enumerate()
                .map(|(index, card)| {
                    view! {
                        <ReceiptItemCard
                            card=card
                            id_prefix=format!("{id_prefix}-card-{index}")
                            expanded_ids=expanded_ids
                        />
                    }
                })
                .collect_view()}
        </div>
    };
    if summary.is_empty() {
        return view! {
            <section data-testid="receipt-card-group">
                <div class="flex min-h-[44px] items-center rounded-lg bg-slate-700 px-3 py-2">
                    <span class="text-sm font-semibold text-white">{label}</span>
                    <span class="ml-2 inline-flex items-center rounded bg-white/20 px-2 py-0.5 text-xs font-medium text-white">
                        {format!("{count}件")}
                    </span>
                </div>
                {card_list}
            </section>
        }
        .into_any();
    }
    let (primary_label, primary_value) = summary.last().cloned().unwrap_or_default();
    let aria_label = format!("{label} {count}件 {primary_label} {primary_value}");
    view! {
        <section data-testid="receipt-card-group">
            <div class="overflow-hidden rounded-lg border border-slate-700">
                <button
                    id=button_id.clone()
                    type="button"
                    aria-label=aria_label
                    aria-expanded=move || if expanded.get() { "true" } else { "false" }
                    aria-controls=details_id.clone()
                    on:click=move |_| {
                        expanded_ids.update(|set| {
                            if !set.remove(&toggle_id) {
                                set.insert(toggle_id.clone());
                            }
                        })
                    }
                    class="flex min-h-[44px] w-full items-center gap-2 bg-slate-700 px-3 py-2 text-left focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-white"
                >
                    <span class="min-w-0 flex-1 truncate text-sm font-semibold text-white">
                        {label}
                        <span class="ml-2 inline-flex items-center rounded bg-white/20 px-2 py-0.5 text-xs font-medium text-white">
                            {format!("{count}件")}
                        </span>
                    </span>
                    <span
                        class="flex shrink-0 items-baseline gap-1 whitespace-nowrap"
                        aria-hidden="true"
                    >
                        <span class="text-xs text-slate-200">{primary_label}</span>
                        <span class="font-mono text-sm font-semibold tabular-nums text-white">
                            {primary_value}
                        </span>
                        <svg
                            class=move || {
                                if expanded.get() {
                                    "h-4 w-4 shrink-0 self-center text-slate-200 rotate-180"
                                } else {
                                    "h-4 w-4 shrink-0 self-center text-slate-200"
                                }
                            }
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            aria-hidden="true"
                        >
                            <path
                                d="m6 9 6 6 6-6"
                                stroke-width="2"
                                stroke-linecap="round"
                                stroke-linejoin="round"
                            />
                        </svg>
                    </span>
                </button>
                <div
                    id=details_id
                    role="region"
                    aria-labelledby=button_id
                    hidden=move || !expanded.get()
                    class="border-t border-slate-200 bg-white px-3 py-1"
                >
                    {move || {
                        expanded
                            .get()
                            .then(|| {
                                view! {
                                    <dl>
                                        {summary
                                            .iter()
                                            .map(|(label, value)| {
                                                let negative = is_negative_text(value);
                                                let value_class = if negative {
                                                    "min-w-0 break-words text-right text-sm font-semibold tabular-nums text-red-800"
                                                } else {
                                                    "min-w-0 break-words text-right text-sm font-semibold tabular-nums text-slate-800"
                                                };
                                                view! {
                                                    <div class="flex items-start justify-between gap-3 border-b border-slate-100 py-1.5 last:border-b-0">
                                                        <dt class="shrink-0 pt-0.5 text-xs text-slate-600">
                                                            {*label}
                                                        </dt>
                                                        <dd
                                                            class=value_class
                                                            data-negative=negative.then_some("true")
                                                        >
                                                            {value.clone()}
                                                        </dd>
                                                    </div>
                                                }
                                            })
                                            .collect_view()}
                                    </dl>
                                }
                            })
                    }}
                </div>
            </div>
            {card_list}
        </section>
    }
    .into_any()
}

fn table_headers(tab: ReceiptsTab) -> &'static [&'static str] {
    match tab {
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
    }
}

// 幅は列に追随させるため基本順で持ち、表示時に column_order と同じ並びにする
fn table_column_widths(tab: ReceiptsTab) -> &'static [&'static str] {
    match tab {
        ReceiptsTab::Dividend => &[
            "84px", "64px", "64px", "72px", "160px", "72px", "56px", "84px", "64px", "84px",
        ],
        ReceiptsTab::DomesticStock => &[
            "84px", "72px", "156px", "60px", "56px", "76px", "82px", "82px", "82px", "64px", "84px",
        ],
        ReceiptsTab::MutualFund => &[
            "112px", "300px", "60px", "112px", "98px", "128px", "116px", "112px", "106px", "118px",
        ],
    }
}

// 幅と同じく基本順で持ち、td には対応する text-* と tabular-nums を付ける
fn table_column_aligns(tab: ReceiptsTab) -> &'static [&'static str] {
    match tab {
        ReceiptsTab::Dividend => &[
            "left", "left", "left", "center", "left", "right", "right", "right", "right", "right",
        ],
        ReceiptsTab::DomesticStock => &[
            "left", "center", "left", "left", "right", "right", "right", "right", "right", "right",
            "right",
        ],
        ReceiptsTab::MutualFund => &[
            "left", "left", "left", "right", "right", "right", "right", "right", "right", "right",
        ],
    }
}

// Closure は Send/Sync でないためシグナルや on_cleanup の捕捉に置けず、
// マウント中だけ生存させたいので thread_local で管理する
type TableHeightObserver = (
    web_sys::ResizeObserver,
    Closure<dyn FnMut(Vec<web_sys::ResizeObserverEntry>)>,
);
thread_local! {
    static TABLE_HEIGHT_OBSERVERS: RefCell<HashMap<usize, TableHeightObserver>> =
        RefCell::new(HashMap::new());
    static TABLE_HEIGHT_OBSERVER_NEXT_ID: Cell<usize> = const { Cell::new(0) };
}

#[component]
fn ReceiptTable(
    tab: ReceiptsTab,
    rows: Vec<ReceiptItem>,
    all_rows: Vec<ReceiptItem>,
    query: String,
    expanded_ids: RwSignal<HashSet<String>>,
) -> impl IntoView {
    let headers: &[&'static str] = table_headers(tab);
    let groups = table_groups(tab, &rows, &all_rows, &query);
    let order = column_order(tab, &rows, &query);
    let fields = card_fields(tab);
    let labels = summary_labels(tab);
    let slug = TAB_IDS[tab as usize];
    let mut card_ordinals = idless_row_ordinals(&all_rows);
    let card_groups: Vec<_> = groups
        .iter()
        .enumerate()
        .map(|(group_index, group)| {
            let summary: Vec<(&'static str, String)> = labels
                .iter()
                .copied()
                .zip(group.summary.iter().cloned())
                .collect();
            let cards: Vec<CardRowData> = group
                .rows
                .iter()
                .map(|(id, raw_key, cells)| {
                    let ordinal = if id.is_empty() {
                        card_ordinals
                            .get_mut(raw_key)
                            .and_then(|queue| queue.pop_front())
                            .unwrap_or_default()
                    } else {
                        0
                    };
                    card_row_data(
                        card_key(slug, id, raw_key, ordinal),
                        cells,
                        headers,
                        &order,
                        fields,
                    )
                })
                .collect();
            (
                group_index,
                group.key.clone(),
                group.label.clone(),
                group.rows.len(),
                summary,
                cards,
            )
        })
        .collect();
    let headers: Vec<_> = order.iter().map(|i| headers[*i]).collect();
    let widths: Vec<_> = order.iter().map(|i| table_column_widths(tab)[*i]).collect();
    let cell_classes: Vec<&'static str> = order
        .iter()
        .map(|i| match table_column_aligns(tab)[*i] {
            "center" => "text-center",
            "right" => "text-right tabular-nums",
            _ => "text-left",
        })
        .collect();
    let table_scroll = NodeRef::<leptos::html::Div>::new();
    let table_max_height = RwSignal::new(Option::<f64>::None);
    let measure_table = move || {
        let Some(element) = table_scroll.get() else {
            return;
        };
        let Some(viewport) = web_sys::window()
            .and_then(|window| window.inner_height().ok())
            .and_then(|height| height.as_f64())
        else {
            return;
        };
        let available = viewport - element.get_bounding_client_rect().top() - 20.0;
        table_max_height.set(Some(available.max(200.0)));
    };
    let on_table_resize = window_event_listener(ev::resize, move |_| measure_table());
    let observer_key = web_sys::window().and_then(|_| {
        let callback = Closure::<dyn FnMut(Vec<web_sys::ResizeObserverEntry>)>::new(move |_| {
            measure_table();
        });
        web_sys::ResizeObserver::new(callback.as_ref().unchecked_ref())
            .ok()
            .map(|observer| {
                let key = TABLE_HEIGHT_OBSERVER_NEXT_ID.with(|next| {
                    let key = next.get();
                    next.set(key + 1);
                    key
                });
                TABLE_HEIGHT_OBSERVERS.with(|observers| {
                    observers.borrow_mut().insert(key, (observer, callback));
                });
                key
            })
    });
    Effect::new(move |_| {
        measure_table();
        let (Some(element), Some(key)) = (table_scroll.get(), observer_key) else {
            return;
        };
        TABLE_HEIGHT_OBSERVERS.with(|observers| {
            let observers = observers.borrow();
            let Some((observer, _)) = observers.get(&key) else {
                return;
            };
            observer.observe(&element);
            if let Some(parent) = element.parent_element() {
                observer.observe(&parent);
            }
            if let Some(body) = web_sys::window()
                .and_then(|window| window.document())
                .and_then(|document| document.body())
            {
                observer.observe(&body);
            }
        });
    });
    on_cleanup(move || {
        on_table_resize.remove();
        if let Some(key) = observer_key {
            TABLE_HEIGHT_OBSERVERS.with(|observers| {
                if let Some((observer, _callback)) = observers.borrow_mut().remove(&key) {
                    observer.disconnect();
                }
            });
        }
    });
    view! {
        <div
            class="overflow-hidden rounded-xl border border-slate-950/10 bg-white/95 shadow-[0_16px_44px_-36px_rgba(15,23,42,0.9)] print:border-black print:shadow-none"
            data-testid="receipt-card"
        >
            <div class="p-0" data-testid="receipt-card-body">
                <div class="hidden sm:block">
                    <div
                        node_ref=table_scroll
                        class="relative w-full overflow-auto rounded-lg border border-slate-950/10 bg-white print:overflow-visible! print:max-h-none! [&::-webkit-scrollbar]:h-2.5 [&::-webkit-scrollbar-track]:bg-slate-100 [&::-webkit-scrollbar-thumb]:bg-slate-400 [&::-webkit-scrollbar-thumb]:rounded-full [&::-webkit-scrollbar-thumb]:hover:bg-slate-500"
                        style:max-height=move || {
                            table_max_height
                                .get()
                                .map(|height| format!("{height}px"))
                                .unwrap_or_default()
                        }
                    >
                        <table class="w-full table-fixed border-collapse text-left text-[12px] leading-5 sm:text-[13px] [&_th]:border [&_th]:border-slate-200 [&_td]:border [&_td]:border-slate-200 [&_th]:py-1.5 [&_th]:px-2 [&_td]:py-1.5 [&_td]:px-2 sm:[&_th]:py-2 sm:[&_th]:px-2.5 sm:[&_td]:py-2 sm:[&_td]:px-2.5 [&_th]:whitespace-nowrap [&_td]:whitespace-nowrap [&_th]:overflow-hidden [&_td]:overflow-hidden [&_th]:text-ellipsis [&_td]:text-ellipsis [&_tbody_td]:border-b [&_tbody_td]:border-slate-100 [&_tbody_th]:border-b [&_tbody_th]:border-slate-100">
                            <thead class="sticky top-0 z-10 bg-slate-100 text-slate-800">
                                <tr class="bg-slate-50">
                                    {headers
                                        .iter()
                                        .zip(widths.iter())
                                        .map(|(header, width)| {
                                            view! {
                                                <th
                                                    class="text-center font-black text-slate-800"
                                                    style:width=*width
                                                    style:max-width=*width
                                                >
                                                    {*header}
                                                </th>
                                            }
                                        })
                                        .collect_view()}
                                </tr>
                            </thead>
                            <tbody>
                                {groups
                                    .iter()
                                    .enumerate()
                                    .map(|(group_index, group)| {
                                        let count = group.rows.len();
                                        let top = if group_index > 0 {
                                            " border-t-2 border-slate-300"
                                        } else {
                                            ""
                                        };
                                        view! {
                                            <tr>
                                                <td
                                                    colspan={headers.len() - 3}
                                                    class=format!(
                                                        "whitespace-normal bg-slate-100 text-slate-800 font-semibold border-l-2 border-slate-500{top}"
                                                    )
                                                >
                                                    <span class="text-sm font-medium">{group.label.clone()}</span>
                                                    <span class="ml-2 inline-flex items-center rounded bg-slate-600 px-2 py-0.5 text-xs font-medium text-white">
                                                        {format!("{count}件")}
                                                    </span>
                                                </td>
                                                {group
                                                    .summary
                                                    .iter()
                                                    .map(|value| {
                                                        let negative = is_negative_text(value);
                                                        view! {
                                                            <td
                                                                class=format!(
                                                                    "bg-slate-100 text-slate-800 text-right font-semibold{top}"
                                                                )
                                                                data-negative=negative.then_some("true")
                                                            >
                                                                {value.clone()}
                                                            </td>
                                                        }
                                                    })
                                                    .collect_view()}
                                            </tr>
                                            {group
                                                .rows
                                                .iter()
                                                .map(|(_, _, cells)| {
                                                    let cells: Vec<_> = order.iter().map(|i| cells[*i].clone()).collect();
                                                    view! {
                                                        <tr>
                                                            {cells
                                                                .into_iter()
                                                                .enumerate()
                                                                .map(|(col_index, cell)| {
                                                                    let align = cell_classes[col_index];
                                                                    match cell {
                                                                        ReceiptCell::SecurityCode(code) => view! {
                                                                            <td class=align>
                                                                                <SecurityCodeLink value=code />
                                                                            </td>
                                                                        }
                                                                        .into_any(),
                                                                        ReceiptCell::InstrumentName { name, code } => view! {
                                                                            <td class=align>
                                                                                <CopyableInstrumentName name=name code=code.unwrap_or_default() />
                                                                            </td>
                                                                        }
                                                                        .into_any(),
                                                                        ReceiptCell::Text(value) => {
                                                                            let negative = is_negative_text(&value);
                                                                            let title = value.clone();
                                                                            view! {
                                                                                <td
                                                                                    class=align
                                                                                    title=title
                                                                                    data-negative=negative.then_some("true")
                                                                                >
                                                                                    {value}
                                                                                </td>
                                                                            }
                                                                            .into_any()
                                                                        }
                                                                    }
                                                                })
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
                </div>
                <div class="sm:hidden" data-testid="receipt-card-list">
                    <div class="space-y-4">
                        {card_groups
                            .into_iter()
                            .map(|(group_index, key, label, count, summary, cards)| {
                                view! {
                                    <MobileCardGroup
                                        label=label
                                        count=count
                                        summary=summary
                                        cards=cards
                                        id_prefix=format!("receipt-{slug}-group-{group_index}")
                                        expanded_key=format!("{slug}:g:{key}")
                                        expanded_ids=expanded_ids
                                    />
                                }
                            })
                            .collect_view()}
                    </div>
                </div>
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests;
