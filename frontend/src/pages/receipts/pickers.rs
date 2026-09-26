use crate::list_search::SearchOption;
use crate::receipts::filter::{DateSegment, ReceiptSearch, SearchKey};
use leptos::prelude::*;
use wasm_bindgen::JsCast;

#[component]
pub(crate) fn SecurityDropdown(
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
pub(crate) fn YearDropdown(
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
pub(crate) fn ToggleCategory(
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
pub(crate) enum DateInputField {
    Month,
    Date,
    RangeStart,
    RangeEnd,
}

pub(crate) fn date_input_value(state: &ReceiptSearch, field: DateInputField) -> String {
    match field {
        DateInputField::Month => state.date_inputs.month_value.clone(),
        DateInputField::Date => state.date_inputs.date_value.clone(),
        DateInputField::RangeStart => state.date_inputs.range_start.clone(),
        DateInputField::RangeEnd => state.date_inputs.range_end.clone(),
    }
}

pub(crate) fn date_input_type(field: DateInputField) -> &'static str {
    match field {
        DateInputField::Month => "month",
        DateInputField::Date | DateInputField::RangeStart | DateInputField::RangeEnd => "date",
    }
}

pub(crate) fn format_date_input_label(value: &str, fallback: &str) -> String {
    if value.is_empty() {
        fallback.to_string()
    } else {
        value.replace('-', "/")
    }
}

pub(crate) fn visible_date_segment(state: &ReceiptSearch, has_years: bool) -> DateSegment {
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

pub(crate) fn next_year_option_index(
    current: Option<usize>,
    option_count: usize,
    key: &str,
) -> Option<usize> {
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
                            class="year-picker-dropdown"
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
pub(crate) fn DatePeriod(
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
