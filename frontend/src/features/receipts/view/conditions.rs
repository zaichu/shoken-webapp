use crate::features::receipts::filter::{
    DateSegment, ReceiptSearch, SearchKey, build_combined_query, search_categories,
};
use crate::features::receipts::{ReceiptsTab, TabState, use_receipts_data};
use crate::session::use_session;
use crate::ui::page_frame::PageFrame;
use leptos::ev::MouseEvent;
use leptos::prelude::*;

#[component]
fn ConditionSection(
    search: RwSignal<ReceiptSearch>,
    key: SearchKey,
    label: &'static str,
    options: Memo<Vec<crate::support::list_search::SearchOption>>,
    on_select: impl Fn(String) + Copy + 'static + Send,
    on_clear: impl Fn(MouseEvent) + Copy + 'static + Send,
) -> impl IntoView {
    let selected = Memo::new(move |_| search.with(|s| s.selected_queries.get(key).to_string()));
    let has_selection = move || !selected.get().is_empty();

    view! {
        <div class="mb-4">
            <div class="flex items-center justify-between mb-2">
                <span class="text-sm font-semibold text-text">{label}</span>
                {move || {
                    if has_selection() {
                        view! {
                            <button
                                type="button"
                                class="text-xs text-info underline hover:text-info-deep"
                                on:click=on_clear
                            >
                                "解除"
                            </button>
                        }.into_any()
                    } else {
                        ().into_any()
                    }
                }}
            </div>
            <div class="flex flex-wrap gap-1.5">
                {move || options.get().iter().map(|opt| {
                    let opt_value = opt.value.clone();
                    let opt_label = opt.label.clone();
                    let opt_value_for_selected = opt_value.clone();
                    let is_selected = move || selected.get() == opt_value_for_selected;
                    view! {
                        <button
                            type="button"
                            class=move || format!(
                                "px-3 py-1.5 rounded-full text-sm font-medium transition-colors {}",
                                if is_selected() {
                                    "bg-info-soft border border-info-border text-info"
                                } else {
                                    "bg-surface border border-border-strong text-text-soft hover:bg-surface-sunken"
                                }
                            )
                            on:click=move |_| on_select(opt_value.clone())
                        >
                            {opt_label}
                        </button>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}

#[component]
fn DateSegmentSelector(
    search: RwSignal<ReceiptSearch>,
    years: Memo<Vec<crate::support::list_search::SearchOption>>,
) -> impl IntoView {
    let segment = Memo::new(move |_| search.with(|s| s.date_segment));
    let segment_options = ["すべて", "年", "年月", "日", "範囲"];

    view! {
        <div class="mb-4">
            <span class="text-sm font-semibold text-text">"期間"</span>
            <div class="flex flex-wrap gap-1.5 mt-2">
                {segment_options.iter().map(|opt| {
                    let opt = *opt;
                    let is_selected = move || {
                        let seg = segment.get();
                        match opt {
                            "すべて" => seg == DateSegment::Year && seg.value() == "年" && search.with(|s| s.selected_queries.date.is_empty()),
                            _ => seg.value() == opt,
                        }
                    };
                    view! {
                        <button
                            type="button"
                            class=move || format!(
                                "px-3 py-1.5 rounded-full text-sm font-medium transition-colors {}",
                                if is_selected() {
                                    "bg-info-soft border border-info-border text-info"
                                } else {
                                    "bg-surface border border-border-strong text-text-soft hover:bg-surface-sunken"
                                }
                            )
                            on:click=move |_| search.update(|s| {
                                match opt {
                                    "すべて" => {
                                        s.selected_queries.date.clear();
                                        s.date_segment = DateSegment::Year;
                                        s.date_inputs = Default::default();
                                        s.query = build_combined_query(&s.selected_queries);
                                    }
                                    "年" => s.change_date_segment(DateSegment::Year),
                                    "年月" => s.change_date_segment(DateSegment::Month),
                                    "日" => s.change_date_segment(DateSegment::Date),
                                    "範囲" => s.change_date_segment(DateSegment::Range),
                                    _ => {}
                                }
                            })
                        >
                            {opt}
                        </button>
                    }
                }).collect_view()}
            </div>
            {move || {
                let seg = segment.get();
                match seg {
                    DateSegment::Year => view! {
                        <YearPicker search=search years=years />
                    }.into_any(),
                    DateSegment::Month => view! {
                        <MonthPicker search=search years=years />
                    }.into_any(),
                    DateSegment::Date => view! {
                        <DatePicker search=search />
                    }.into_any(),
                    DateSegment::Range => view! {
                        <RangePicker search=search />
                    }.into_any(),
                }
            }}
        </div>
    }
}

#[component]
fn YearPicker(
    search: RwSignal<ReceiptSearch>,
    years: Memo<Vec<crate::support::list_search::SearchOption>>,
) -> impl IntoView {
    view! {
        <div class="mt-2 flex flex-wrap gap-1.5">
            {move || {
                let years = years.get();
                if years.is_empty() {
                    return view! { <span class="text-sm text-text-muted">"該当する年がありません"</span> }.into_any();
                }
                view! {
                    {years.iter().map(|y| {
                        let y_value = y.value.clone();
                        let y_label = y.label.clone();
                        let y_value_for_selected = y_value.clone();
                        let selected = Memo::new(move |_| search.with(|s| s.date_inputs.year_value.clone() == y_value_for_selected));
                        view! {
                            <button
                                type="button"
                                class=move || format!(
                                    "px-3 py-1.5 rounded-full text-sm font-medium transition-colors {}",
                                    if selected.get() {
                                        "bg-info-soft border border-info-border text-info"
                                    } else {
                                        "bg-surface border border-border-strong text-text-soft hover:bg-surface-sunken"
                                    }
                                )
                                on:click=move |_| search.update(|s| s.select_year(y_value.clone()))
                            >
                                {y_label}
                            </button>
                        }
                    }).collect_view()}
                }.into_any()
            }}
        </div>
    }
}

#[component]
fn MonthPicker(
    search: RwSignal<ReceiptSearch>,
    years: Memo<Vec<crate::support::list_search::SearchOption>>,
) -> impl IntoView {
    let months = (1..=12).map(|m| format!("{:02}", m)).collect::<Vec<_>>();
    let selected_month = Memo::new(move |_| search.with(|s| s.date_inputs.month_value.clone()));
    let selected_year = Memo::new(move |_| search.with(|s| s.date_inputs.year_value.clone()));

    view! {
        <div class="mt-2 flex flex-wrap gap-1.5">
            {move || {
                let years = years.get();
                if years.is_empty() {
                    return view! {
                        <div class="flex flex-wrap gap-1.5">
                            {months.iter().map(|m| {
                                let m_value = m.clone();
                                let m_label = format!("{}月", m_value.parse::<u32>().unwrap_or(0));
                                let m_value_for_selected = m_value.clone();
                                let is_selected = move || selected_month.get() == m_value_for_selected;
                                view! {
                                    <button
                                        type="button"
                                        class=move || format!(
                                            "px-3 py-1.5 rounded-full text-sm font-medium transition-colors {}",
                                            if is_selected() {
                                                "bg-info-soft border border-info-border text-info"
                                            } else {
                                                "bg-surface border border-border-strong text-text-soft hover:bg-surface-sunken"
                                            }
                                        )
                                        on:click=move |_| {
                                            search.update(|s| {
                                                s.date_inputs.month_value = m_value.clone();
                                                let year = s.date_inputs.year_value.clone();
                                                if !year.is_empty() {
                                                    s.set_date_query(format!("{}-{}", year, m_value));
                                                }
                                            });
                                        }
                                    >
                                        {m_label}
                                    </button>
                                }
                            }).collect_view()}
                        </div>
                    }.into_any();
                }
                view! {
                    <div class="flex items-center gap-1">
                        <select
                            class="h-9 px-3 border border-border-strong bg-surface rounded-lg text-sm"
                            prop:value=move || selected_year.get()
                            on:change=move |ev| search.update(|s| s.select_year(event_target_value(&ev)))
                        >
                            <option value="">"年を選択"</option>
                            {years.iter().map(|y| {
                                view! { <option value={y.value.clone()}>{y.label.clone()}</option> }
                            }).collect_view()}
                        </select>
                        <select
                            class="h-9 px-3 border border-border-strong bg-surface rounded-lg text-sm"
                            prop:value=move || selected_month.get()
                            on:change=move |ev| {
                                let month = event_target_value(&ev);
                                search.update(|s| {
                                    s.date_inputs.month_value = month.clone();
                                    let year = s.date_inputs.year_value.clone();
                                    if !year.is_empty() {
                                        s.set_date_query(format!("{}-{}", year, month));
                                    }
                                });
                            }
                        >
                            {(1..=12).map(|m| {
                                let m_str = format!("{:02}", m);
                                let m_label = format!("{}月", m);
                                view! { <option value={m_str.clone()}>{m_label}</option> }
                            }).collect_view()}
                        </select>
                    </div>
                }.into_any()
            }}
        </div>
    }
}

#[component]
fn DatePicker(search: RwSignal<ReceiptSearch>) -> impl IntoView {
    let selected_date = Memo::new(move |_| search.with(|s| s.date_inputs.date_value.clone()));

    view! {
        <div class="mt-2">
            <input
                type="date"
                class="h-10 px-3 border border-border-strong bg-surface rounded-lg text-sm max-w-xs"
                prop:value=move || selected_date.get()
                on:input=move |ev| {
                    search.update(|s| {
                        s.date_inputs.date_value = event_target_value(&ev);
                        s.set_date_query(event_target_value(&ev));
                    });
                }
            />
        </div>
    }
}

#[component]
fn RangePicker(search: RwSignal<ReceiptSearch>) -> impl IntoView {
    let range_start = Memo::new(move |_| search.with(|s| s.date_inputs.range_start.clone()));
    let range_end = Memo::new(move |_| search.with(|s| s.date_inputs.range_end.clone()));

    view! {
        <div class="mt-2 flex flex-wrap gap-2 items-end">
            <div class="flex-1 min-w-40">
                <label class="block text-xs text-text-muted mb-1">"開始日"</label>
                <input
                    type="date"
                    class="h-10 px-3 border border-border-strong bg-surface rounded-lg text-sm w-full"
                    prop:value=move || range_start.get()
                    on:input=move |ev| {
                        search.update(|s| {
                            s.date_inputs.range_start = event_target_value(&ev);
                            s.update_range_query();
                        });
                    }
                />
            </div>
            <span class="text-text-muted">"〜"</span>
            <div class="flex-1 min-w-40">
                <label class="block text-xs text-text-muted mb-1">"終了日"</label>
                <input
                    type="date"
                    class="h-10 px-3 border border-border-strong bg-surface rounded-lg text-sm w-full"
                    prop:value=move || range_end.get()
                    on:input=move |ev| {
                        search.update(|s| {
                            s.date_inputs.range_end = event_target_value(&ev);
                            s.update_range_query();
                        });
                    }
                />
            </div>
        </div>
    }
}

#[component]
pub fn ReceiptsConditionsPage() -> impl IntoView {
    let store = use_receipts_data(use_session(), ReceiptsTab::Dividend);
    let search = store.search;
    let active_tab = store.active_tab;

    let display_rows = Memo::new(move |_| match store.tab_state(active_tab.get()) {
        TabState::Ready(data) => data.rows.clone(),
        _ => Vec::new(),
    });

    let categories = Memo::new(move |_| search_categories(active_tab.get(), &display_rows.get()));
    let years = Memo::new(move |_| categories.with(|c| c.years.clone()));
    let products = Memo::new(move |_| categories.with(|c| c.products.clone()));
    let accounts = Memo::new(move |_| categories.with(|c| c.accounts.clone()));
    let securities = Memo::new(move |_| categories.with(|c| c.securities.clone()));
    let has_years = move || !years.get().is_empty();

    view! {
        <PageFrame title="検索条件">
            <div class="page-gutter flex flex-col gap-4 py-4 max-w-2xl">
                <div class="flex items-center justify-between mb-4">
                    <h2 class="text-lg font-bold text-ink">"検索条件"</h2>
                    <button
                        type="button"
                        class="text-sm text-info underline hover:text-info-deep"
                        on:click=move |_| search.update(|s| s.clear(has_years()))
                    >
                        "すべてクリア"
                    </button>
                </div>

                <ConditionSection
                    search=search
                    key=SearchKey::Securities
                    label="銘柄"
                    options=securities
                    on_select=move |v| search.update(|s| s.select_quick(SearchKey::Securities, v))
                    on_clear=move |_| search.update(|s| s.clear_key(SearchKey::Securities, has_years()))
                />

                {move || {
                    let categories = categories.get();
                    if categories.dates {
                        view! { <DateSegmentSelector search=search years=years /> }.into_any()
                    } else {
                        ().into_any()
                    }
                }}

                <ConditionSection
                    search=search
                    key=SearchKey::Products
                    label="商品"
                    options=products
                    on_select=move |v| search.update(|s| s.select_quick(SearchKey::Products, v))
                    on_clear=move |_| search.update(|s| s.clear_key(SearchKey::Products, has_years()))
                />

                <ConditionSection
                    search=search
                    key=SearchKey::Accounts
                    label="口座"
                    options=accounts
                    on_select=move |v| search.update(|s| s.select_quick(SearchKey::Accounts, v))
                    on_clear=move |_| search.update(|s| s.clear_key(SearchKey::Accounts, has_years()))
                />

                <div class="pt-4 border-t border-border">
                    <a
                        href="/receipts"
                        class="inline-flex items-center justify-center w-full h-11 text-sm font-semibold text-text-inverse rounded-md border border-ink bg-ink hover:bg-ink-hover transition-colors"
                    >
                        "取引明細に戻る"
                    </a>
                </div>
            </div>
        </PageFrame>
    }
}
