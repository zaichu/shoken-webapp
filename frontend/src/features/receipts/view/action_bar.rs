use crate::features::receipts::filter::{DateSegment, ReceiptSearch, SearchKey, search_categories};
use crate::features::receipts::{ReceiptsStore, ReceiptsTab, TabState};
use leptos::prelude::*;

#[component]
fn ActiveConditionBadge(
    search: RwSignal<ReceiptSearch>,
    key: SearchKey,
    label: &'static str,
    on_clear: impl Fn() + Copy + 'static + Send,
) -> impl IntoView {
    let value = Memo::new(move |_| search.with(|s| s.selected_queries.get(key).to_string()));
    move || {
        let v = value.get();
        (!v.is_empty()).then(|| {
            view! {
                <button
                    type="button"
                    class="inline-flex items-center gap-1 h-7 px-2.5 rounded-full border border-info-border bg-info-soft text-xs font-semibold text-info"
                    on:click=move |_| on_clear()
                >
                    {label}
                    <span aria-hidden="true">"✕"</span>
                </button>
            }
        })
    }
}

#[component]
pub fn ReceiptsActionBar(store: ReceiptsStore, active_tab: RwSignal<ReceiptsTab>) -> impl IntoView {
    let search = store.search;
    let tab = active_tab;

    let display_rows = Memo::new(move |_| match store.tab_state(tab.get()) {
        TabState::Ready(data) => data.rows.clone(),
        _ => Vec::new(),
    });

    let categories = Memo::new(move |_| search_categories(tab.get(), &display_rows.get()));
    let has_active_search = Signal::derive(move || !search.with(|s| s.is_default()));
    let years = Memo::new(move |_| categories.with(|c| c.years.clone()));
    let has_years = move || !years.get().is_empty();
    let _has_dates = move || categories.with(|c| c.dates);

    view! {
        <div class="sm:hidden no-print action-bar-mobile">
            <div class="page-gutter flex flex-col gap-2 py-2">
                {move || {
                    has_active_search.get().then(|| {
                        view! {
                            <div class="flex flex-wrap items-center gap-1.5" role="group" aria-label="適用中の検索条件">
                                <span class="text-xs font-semibold text-text-muted">"条件"</span>
                                <ActiveConditionBadge search=search key=SearchKey::Date label="期間" on_clear=move || search.update(|s| s.clear(has_years())) />
                                <ActiveConditionBadge search=search key=SearchKey::Products label="商品" on_clear=move || search.update(|s| s.clear(has_years())) />
                                <ActiveConditionBadge search=search key=SearchKey::Accounts label="口座" on_clear=move || search.update(|s| s.clear(has_years())) />
                                <button
                                    type="button"
                                    class="text-xs font-semibold text-info underline underline-offset-1 hover:text-info-deep"
                                    on:click=move |_| search.update(|s| s.clear(has_years()))
                                >
                                    "全解除"
                                </button>
                            </div>
                        }
                    })
                }}
                <div class="flex items-center gap-2">
                    <label class="flex-1 search-input-frame">
                        <svg aria-hidden="true" class="h-5 w-5 shrink-0 mx-2 text-text-muted" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                            <circle cx="11" cy="11" r="7"></circle>
                            <path d="M20 20l-3.5-3.5"></path>
                        </svg>
                        <input
                            type="search"
                            class="search-input"
                            placeholder="銘柄名・コード"
                            prop:value=move || search.with(|s| s.query.clone())
                            on:input=move |ev| search.update(|s| s.query = event_target_value(&ev))
                        />
                    </label>
                    <a href="/receipts/conditions" class="shrink-0 h-11 px-4 text-sm font-semibold text-text-soft border border-border-strong rounded-lg hover:bg-surface-raised hover:text-text-deep transition-colors">
                        "検索条件"
                    </a>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn ReceiptsPcSearchBar(
    store: ReceiptsStore,
    active_tab: RwSignal<ReceiptsTab>,
) -> impl IntoView {
    let search = store.search;
    let tab = active_tab;

    let display_rows = Memo::new(move |_| match store.tab_state(tab.get()) {
        TabState::Ready(data) => data.rows.clone(),
        _ => Vec::new(),
    });

    let categories = Memo::new(move |_| search_categories(tab.get(), &display_rows.get()));
    let years = Memo::new(move |_| categories.with(|c| c.years.clone()));
    let products = Memo::new(move |_| categories.with(|c| c.products.clone()));
    let accounts = Memo::new(move |_| categories.with(|c| c.accounts.clone()));
    let has_years = move || !years.get().is_empty();
    let has_dates = move || categories.with(|c| c.dates);
    let has_products = move || !products.get().is_empty();
    let has_accounts = move || !accounts.get().is_empty();

    let _year_picker_open = RwSignal::new(false);
    let period_value = Memo::new(move |_| search.with(|s| s.date_segment.value()));
    let period_options = ["すべて", "年", "年月", "日", "範囲"];

    view! {
            <div class="pc-search-bar no-print" role="search" aria-label="取引明細の検索">
                <label class="flex-1 min-w-[260px] search-input-frame">
                    <svg aria-hidden="true" class="h-5 w-5 shrink-0 mx-2 text-text-muted" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <circle cx="11" cy="11" r="7"></circle>
                        <path d="M20 20l-3.5-3.5"></path>
                    </svg>
                    <input
                        type="search"
                        class="search-input"
                        placeholder="銘柄名・コード"
                        prop:value=move || search.with(|s| s.query.clone())
                        on:input=move |ev| search.update(|s| s.query = event_target_value(&ev))
                    />
                </label>

                {move || {
                    has_dates().then(|| {
                        view! {
                            <div class="flex items-center gap-2">
                                <span class="text-sm text-text-muted">"期間"</span>
                                <select
                                    class="h-9 px-3 border border-border-strong bg-surface rounded-lg text-sm"
                                    prop:value=move || period_value.get().to_string()
                                    on:change=move |ev| {
                                        let val = event_target_value(&ev);
                                        search.update(|s| {
                                            s.change_date_segment(match val.as_str() {
                                                "年" => DateSegment::Year,
                                                "月" => DateSegment::Month,
                                                "日" => DateSegment::Date,
                                                "範囲" => DateSegment::Range,
                                                _ => DateSegment::Year,
                                            })
                                        });
                                    }
                                >
                                    {period_options.iter().map(|opt| {
                                        view! { <option value=*opt>{*opt}</option> }
                                    }).collect_view()}
                                </select>
    {move || {
                    has_years().then(|| {
                        view! {
                            <select
                                class="h-9 px-3 border border-border-strong bg-surface rounded-lg text-sm"
                                prop:value=move || search.with(|s| s.date_inputs.year_value.clone())
                                on:change=move |ev| search.update(|s| s.select_year(event_target_value(&ev)))
                            >
                                {years.get().iter().map(|y| {
                                    view! { <option value={y.value.clone()}>{y.label.clone()}</option> }
                                }).collect_view()}
                            </select>
                        }
                    })
                }}
                            </div>
                        }
                    })
                }}

                {move || {
                    has_products().then(|| {
                        view! {
                            <div class="flex items-center gap-2">
                                <span class="text-sm text-text-muted">"商品"</span>
                                <div class="flex gap-1">
                                    {products.get().iter().map(|p| {
                                        let p_value = p.value.clone();
                                        let p_label = p.label.clone();
                                        let p_value_for_selected = p_value.clone();
                                        let p_value_for_click = p_value.clone();
                                        let selected = search.with(move |s| s.selected_queries.get(SearchKey::Products) == p_value_for_selected);
                                        view! {
                                            <button
                                                type="button"
                                                class=move || format!(
                                                    "h-8 px-3 rounded-full text-sm font-medium transition-colors {}",
                                                    if selected {
                                                        "bg-info-soft border border-info-border text-info"
                                                    } else {
                                                        "bg-surface border border-border-strong text-text-soft hover:bg-surface-sunken"
                                                    }
                                                )
                                                aria_pressed=selected
                                                on:click=move |_| search.update(|s| s.select_quick(SearchKey::Products, p_value_for_click.clone()))
                                            >
                                                {p_label}
                                            </button>
                                        }
                                    }).collect_view()}
                                </div>
                            </div>
                        }
                    })
                }}

                {move || {
                    has_accounts().then(|| {
                        view! {
                            <div class="flex items-center gap-2">
                                <span class="text-sm text-text-muted">"口座"</span>
                                <div class="flex gap-1">
                                    {accounts.get().iter().map(|a| {
                                        let a_value = a.value.clone();
                                        let a_label = a.label.clone();
                                        let a_value_for_selected = a_value.clone();
                                        let a_value_for_click = a_value.clone();
                                        let selected = search.with(move |s| s.selected_queries.get(SearchKey::Accounts) == a_value_for_selected);
                                        view! {
                                            <button
                                                type="button"
                                                class=move || format!(
                                                    "h-8 px-3 rounded-full text-sm font-medium transition-colors {}",
                                                    if selected {
                                                        "bg-info-soft border border-info-border text-info"
                                                    } else {
                                                        "bg-surface border border-border-strong text-text-soft hover:bg-surface-sunken"
                                                    }
                                                )
                                                aria_pressed=selected
                                                on:click=move |_| search.update(|s| s.select_quick(SearchKey::Accounts, a_value_for_click.clone()))
                                            >
                                                {a_label}
                                            </button>
                                        }
                                    }).collect_view()}
                                </div>
                            </div>
                        }
                    })
                }}

                <button
                    type="button"
                    class="ml-auto h-8 px-3 text-sm font-semibold text-info hover:underline"
                    on:click=move |_| search.update(|s| s.clear(has_years()))
                >
                    "条件をクリア"
                </button>
            </div>
        }
}
