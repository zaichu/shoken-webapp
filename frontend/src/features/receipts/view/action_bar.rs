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
            let aria_label = format!("{label} {v} を解除");
            view! {
                <button
                    type="button"
                    class="active-condition-chip"
                    aria-label=aria_label
                    on:click=move |_| on_clear()
                >
                    {format!("{label}: {v}")}
                    <span aria-hidden="true">"✕"</span>
                </button>
            }
        })
    }
}

// 銘柄名・コードの文字列検索は一覧側の入力欄で行うため、条件として数えるのは
// 期間・商品・口座の3種類だけにする
const CONDITION_KEYS: [SearchKey; 3] = [SearchKey::Date, SearchKey::Products, SearchKey::Accounts];

pub(crate) fn condition_count(search: &ReceiptSearch) -> usize {
    CONDITION_KEYS
        .iter()
        .filter(|key| !search.selected_queries.get(**key).is_empty())
        .count()
}

/// 絞り込み中の条件を集計の帯の直下にまとめて出す(スマホ)
#[component]
pub fn ReceiptsActiveBand(store: ReceiptsStore, tab: ReceiptsTab) -> impl IntoView {
    let search = store.search;

    let display_rows = Memo::new(move |_| match store.tab_state(tab) {
        TabState::Ready(data) => data.rows.clone(),
        _ => Vec::new(),
    });
    let categories = Memo::new(move |_| search_categories(tab, &display_rows.get()));
    let years = Memo::new(move |_| categories.with(|c| c.years.clone()));
    let has_years = move || !years.get().is_empty();
    let visible = Signal::derive(move || search.with(|s| condition_count(s) > 0));

    view! {
        <Show when=move || visible.get()>
            <div class="receipts-active-band no-print lg:hidden" role="group" aria-label="適用中の検索条件">
                <span class="receipts-active-band-label">"絞り込み中"</span>
                <ActiveConditionBadge search=search key=SearchKey::Date label="期間" on_clear=move || search.update(|s| s.clear_key(SearchKey::Date, has_years())) />
                <ActiveConditionBadge search=search key=SearchKey::Products label="商品" on_clear=move || search.update(|s| s.clear_key(SearchKey::Products, has_years())) />
                <ActiveConditionBadge search=search key=SearchKey::Accounts label="口座" on_clear=move || search.update(|s| s.clear_key(SearchKey::Accounts, has_years())) />
                <button
                    type="button"
                    class="receipts-active-band-clear"
                    on:click=move |_| search.update(|s| s.clear(has_years()))
                >
                    "条件をクリア"
                </button>
            </div>
        </Show>
    }
}

#[component]
pub fn ReceiptsMobileBottomSection(store: ReceiptsStore) -> impl IntoView {
    let search = store.search;

    let active_count = Signal::derive(move || search.with(condition_count));

    view! {
        <div class="receipts-mobile-bottom-section no-print lg:hidden" role="search" aria-label="検索">
            <div class="page-gutter">
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
                            aria-label="銘柄名・コード"
                            prop:value=move || search.with(|s| s.selected_queries.get(SearchKey::Securities).to_string())
                            on:input=move |ev| search.update(|s| s.set_securities_query(event_target_value(&ev)))
                        />
                    </label>
                    <a href="/receipts/conditions" class="receipts-conditions-link shrink-0">
                        "検索条件"
                        {move || {
                            (active_count.get() > 0).then(|| {
                                view! {
                                    <span class="receipts-conditions-count">{active_count.get()}</span>
                                }
                            })
                        }}
                    </a>
                </div>
            </div>
        </div>
    }
}

/// type=date は未入力だとブラウザの形式表示(年/月/日 曜日など)が出るため、
/// 空のあいだは欄の名前を重ねて出す
#[component]
fn PcDateInput(
    label: &'static str,
    value: Memo<String>,
    on_input: impl Fn(String) + Copy + 'static + Send,
) -> impl IntoView {
    view! {
        <span class="pc-date-field">
            <input
                type="date"
                class="h-9 px-3 border border-border-strong bg-surface rounded-lg text-sm"
                aria-label=label
                data-empty=move || value.get().is_empty().then_some("true")
                prop:value=move || value.get()
                on:input=move |ev| on_input(event_target_value(&ev))
            />
            {move || {
                value.get().is_empty().then(|| {
                    view! {
                        <span class="pc-date-field-label" aria-hidden="true">{label}</span>
                    }
                })
            }}
        </span>
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

    let period_value = Memo::new(move |_| {
        search.with(|s| {
            if s.date_all {
                "すべて".to_string()
            } else {
                s.date_segment.value().to_string()
            }
        })
    });
    let current_segment = Memo::new(move |_| search.with(|s| s.date_segment));
    let date_value = Memo::new(move |_| search.with(|s| s.date_inputs.date_value.clone()));
    let range_start = Memo::new(move |_| search.with(|s| s.date_inputs.range_start.clone()));
    let range_end = Memo::new(move |_| search.with(|s| s.date_inputs.range_end.clone()));
    let slot_active = move |segment: DateSegment| {
        !search.with(|s| s.date_all) && current_segment.get() == segment
    };
    let period_options = ["すべて", "年", "年月", "日", "範囲"];

    view! {
        <div class="pc-search-bar no-print" role="search" aria-label="取引明細の検索">
            <label class="flex-1 min-w-[260px] search-input-frame">
                <svg aria-hidden="true" class="h-4 w-4 shrink-0 mx-2 text-text-muted" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <circle cx="11" cy="11" r="7"></circle>
                    <path d="M20 20l-3.5-3.5"></path>
                </svg>
                <input
                    type="search"
                    class="search-input"
                    placeholder="銘柄名・コード"
                    aria-label="銘柄名・コード"
                    prop:value=move || search.with(|s| s.selected_queries.get(SearchKey::Securities).to_string())
                    on:input=move |ev| search.update(|s| s.set_securities_query(event_target_value(&ev)))
                />
            </label>

            {move || {
                has_dates().then(|| {
                    view! {
                        <div class="flex items-center gap-2">
                            <span class="text-xs text-text-muted">"期間"</span>
                            <select
                                class="h-9 px-3 border border-border-strong bg-surface rounded-lg text-sm"
                                aria-label="期間"
                                prop:value=move || period_value.get().to_string()
                                on:change=move |ev| {
                                    let val = event_target_value(&ev);
                                    search.update(|s| match val.as_str() {
                                        "すべて" => s.clear_key(SearchKey::Date, has_years()),
                                        "年" | "年月" | "日" | "範囲" => {
                                            s.change_date_segment(match val.as_str() {
                                                "年" => DateSegment::Year,
                                                "年月" => DateSegment::Month,
                                                "日" => DateSegment::Date,
                                                _ => DateSegment::Range,
                                            })
                                        }
                                        _ => {}
                                    });
                                }
                            >
                                {period_options.iter().map(|opt| {
                                    view! { <option value=*opt>{*opt}</option> }
                                }).collect_view()}
                            </select>
                            // モードごとに入力部の幅が違うと後続の商品・口座・クリアの位置が動くため、
                            // 最も広い範囲モードの幅を重ねたスロットで全モード分確保する
                            <div class="pc-period-slots">
                                <div class="pc-period-slot" data-active=move || slot_active(DateSegment::Year).then_some("true")>
                                    {move || {
                                        has_years().then(|| {
                                            view! {
                                                <select
                                                    class="h-9 px-3 border border-border-strong bg-surface rounded-lg text-sm"
                                                    aria-label="年"
                                                    prop:value=move || search.with(|s| s.date_inputs.year_value.clone())
                                                    on:change=move |ev| search.update(|s| s.select_year(event_target_value(&ev)))
                                                >
                                                    <option value="">"年を選択"</option>
                                                    {years.get().iter().map(|y| {
                                                        view! { <option value={y.value.clone()}>{y.label.clone()}</option> }
                                                    }).collect_view()}
                                                </select>
                                            }
                                        })
                                    }}
                                </div>
                                <div class="pc-period-slot" data-active=move || slot_active(DateSegment::Month).then_some("true")>
                                    {move || {
                                        has_years().then(|| {
                                            view! {
                                                <select
                                                    class="h-9 px-3 border border-border-strong bg-surface rounded-lg text-sm"
                                                    aria-label="年"
                                                    prop:value=move || search.with(|s| s.date_inputs.year_value.clone())
                                                    on:change=move |ev| search.update(|s| s.select_year(event_target_value(&ev)))
                                                >
                                                    <option value="">"年を選択"</option>
                                                    {years.get().iter().map(|y| {
                                                        view! { <option value={y.value.clone()}>{y.label.clone()}</option> }
                                                    }).collect_view()}
                                                </select>
                                                <select
                                                    class="h-9 px-3 border border-border-strong bg-surface rounded-lg text-sm"
                                                    aria-label="月"
                                                    prop:value=move || search.with(|s| s.date_inputs.month_value.clone())
                                                    on:change=move |ev| {
                                                        let month = event_target_value(&ev);
                                                        search.update(|s| {
                                                            s.date_inputs.month_value = month.clone();
                                                            let year = s.date_inputs.year_value.clone();
                                                            if year.is_empty() || month.is_empty() {
                                                                s.set_date_query(year);
                                                            } else {
                                                                s.set_date_query(format!("{}-{}", year, month));
                                                            }
                                                        });
                                                    }
                                                >
                                                    <option value="">"月を選択"</option>
                                                    {(1..=12).map(|m| {
                                                        let m_str = format!("{:02}", m);
                                                        let m_label = format!("{}月", m);
                                                        view! { <option value={m_str.clone()}>{m_label}</option> }
                                                    }).collect_view()}
                                                </select>
                                            }
                                        })
                                    }}
                                </div>
                                <div class="pc-period-slot" data-active=move || slot_active(DateSegment::Date).then_some("true")>
                                    <PcDateInput
                                        label="日付"
                                        value=date_value
                                        on_input=move |v| search.update(|s| {
                                            s.date_inputs.date_value = v.clone();
                                            s.set_date_query(v);
                                        })
                                    />
                                </div>
                                <div class="pc-period-slot" data-active=move || slot_active(DateSegment::Range).then_some("true")>
                                    <PcDateInput
                                        label="開始日"
                                        value=range_start
                                        on_input=move |v| search.update(|s| {
                                            s.date_inputs.range_start = v;
                                            s.update_range_query();
                                        })
                                    />
                                    <span class="text-text-muted">"〜"</span>
                                    <PcDateInput
                                        label="終了日"
                                        value=range_end
                                        on_input=move |v| search.update(|s| {
                                            s.date_inputs.range_end = v;
                                            s.update_range_query();
                                        })
                                    />
                                </div>
                            </div>
                        </div>
                    }
                })
            }}

            {move || {
                has_products().then(|| {
                    view! {
                        <div class="flex items-center gap-2">
                            <span class="text-xs text-text-muted mr-1">"商品"</span>
                            <div class="flex gap-1">
                                {products.get().iter().map(|p| {
                                    let p_value = p.value.clone();
                                    let p_label = p.label.clone();
                                    let p_value_for_selected = p_value.clone();
                                    let p_value_for_click = p_value.clone();
                                    let selected = Memo::new(move |_| search.with(|s| s.selected_queries.get(SearchKey::Products) == p_value_for_selected));
                                    view! {
                                        <button
                                            type="button"
                                            class="receipt-filter-chip"
                                            data-selected=move || selected.get().then_some("true")
                                            aria-pressed=move || selected.get().to_string()
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
                            <span class="text-xs text-text-muted mr-1">"口座"</span>
                            <div class="flex gap-1">
                                {accounts.get().iter().map(|a| {
                                    let a_value = a.value.clone();
                                    let a_label = a.label.clone();
                                    let a_value_for_selected = a_value.clone();
                                    let a_value_for_click = a_value.clone();
                                    let selected = Memo::new(move |_| search.with(|s| s.selected_queries.get(SearchKey::Accounts) == a_value_for_selected));
                                    view! {
                                        <button
                                            type="button"
                                            class="receipt-filter-chip"
                                            data-selected=move || selected.get().then_some("true")
                                            aria-pressed=move || selected.get().to_string()
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
                class="ml-auto h-8 px-3 text-sm font-medium text-info hover:text-info-deep"
                on:click=move |_| search.update(|s| s.clear(has_years()))
            >
                "条件をクリア"
            </button>
        </div>
    }
}
