use crate::features::receipts::filter::{DateSegment, ReceiptSearch, SearchKey};
use crate::support::list_search::SearchOption;
use crate::ui::button::{Button, ButtonVariant};
use crate::ui::labels;
use leptos::prelude::*;

#[component]
pub(crate) fn PeriodDraftContent(
    draft: RwSignal<ReceiptSearch>,
    years: Memo<Vec<SearchOption>>,
    has_years: impl Fn() -> bool + Copy + 'static + Send + Sync,
) -> impl IntoView {
    let segment = Memo::new(move |_| draft.with(|s| s.date_segment));
    view! {
        <div class="condition-segment" role="group" aria-label=labels::PERIOD_KIND_LABEL>
            <PeriodSegmentButtons draft=draft has_years=has_years />
        </div>
        {move || {
            if draft.with(|s| s.date_all) {
                return ().into_any();
            }
            match segment.get() {
                DateSegment::Year => view! { <YearPicker search=draft years=years /> }.into_any(),
                DateSegment::Month => view! { <MonthPicker search=draft years=years /> }.into_any(),
                DateSegment::Date => view! { <DatePicker search=draft /> }.into_any(),
                DateSegment::Range => view! { <RangePicker search=draft /> }.into_any(),
            }
        }}
    }
}

#[component]
fn PeriodSegmentButtons(
    draft: RwSignal<ReceiptSearch>,
    has_years: impl Fn() -> bool + Copy + 'static + Send + Sync,
) -> impl IntoView {
    let segment_options = [
        labels::ALL,
        labels::PERIOD_YEAR,
        labels::PERIOD_YEAR_MONTH,
        labels::PERIOD_DAY,
        labels::PERIOD_RANGE,
    ];
    segment_options
        .iter()
        .map(|opt| {
            let opt = *opt;
            let is_selected = move || {
                draft.with(|s| {
                    if s.date_all {
                        opt == labels::ALL
                    } else {
                        s.date_segment.value() == opt
                    }
                })
            };
            view! {
                <Button
                    variant=ButtonVariant::ConditionSegment
                    aria_pressed=Signal::derive(move || is_selected().to_string())
                    on_click=move |_| {
                        draft.update(|s| {
                            if opt == labels::ALL {
                                s.clear_key(SearchKey::Date, has_years());
                            } else if opt == labels::PERIOD_YEAR {
                                s.change_date_segment(DateSegment::Year);
                            } else if opt == labels::PERIOD_YEAR_MONTH {
                                s.change_date_segment(DateSegment::Month);
                            } else if opt == labels::PERIOD_DAY {
                                s.change_date_segment(DateSegment::Date);
                            } else {
                                s.change_date_segment(DateSegment::Range);
                            }
                        });
                    }
                >
                    {opt}
                </Button>
            }
        })
        .collect_view()
}

#[component]
fn YearPicker(search: RwSignal<ReceiptSearch>, years: Memo<Vec<SearchOption>>) -> impl IntoView {
    view! {
        <div class="mt-2 flex flex-wrap gap-1.5">
            {move || {
                let years = years.get();
                if years.is_empty() {
                    return view! { <span class="text-label text-text-muted">{labels::RECEIPT_NO_YEARS}</span> }.into_any();
                }
                view! {
                    {years.iter().map(|y| {
                        let y_value = y.value.clone();
                        let y_label = y.label.clone();
                        let y_value_for_selected = y_value.clone();
                        let selected = Memo::new(move |_| search.with(|s| s.date_inputs.year_value.clone() == y_value_for_selected));
                        view! {
                            <Button
                                variant=ButtonVariant::YearOption
                                aria_pressed=Signal::derive(move || selected.get().to_string())
                                on_click=move |_| search.update(|s| s.select_year(y_value.clone()))
                            >
                                {y_label}
                            </Button>
                        }
                    }).collect_view()}
                }.into_any()
            }}
        </div>
    }
}

#[component]
fn MonthPicker(search: RwSignal<ReceiptSearch>, years: Memo<Vec<SearchOption>>) -> impl IntoView {
    view! {
        <div class="mt-2 flex flex-wrap gap-1.5">
            {move || {
                if years.get().is_empty() {
                    view! { <MonthButtonsWithoutYears search=search /> }.into_any()
                } else {
                    view! { <MonthYearSelects search=search years=years /> }.into_any()
                }
            }}
        </div>
    }
}

#[component]
fn MonthButtonsWithoutYears(search: RwSignal<ReceiptSearch>) -> impl IntoView {
    let months = (1..=12).map(|m| format!("{m:02}")).collect::<Vec<_>>();
    let selected_month = Memo::new(move |_| search.with(|s| s.date_inputs.month_value.clone()));
    view! {
        <div class="flex flex-wrap gap-1.5">
            {months.iter().map(|m| {
                let m_value = m.clone();
                let m_label = labels::receipt_month_label(m_value.parse::<u32>().unwrap_or(0));
                let m_value_for_selected = m_value.clone();
                let is_selected = move || selected_month.get() == m_value_for_selected;
                view! {
                    <Button
                        variant=ButtonVariant::YearOption
                        aria_pressed=Signal::derive(move || is_selected().to_string())
                        on_click=move |_| {
                            search.update(|s| {
                                s.date_inputs.month_value = m_value.clone();
                                let year = s.date_inputs.year_value.clone();
                                if !year.is_empty() {
                                    s.set_date_query(format!("{year}-{m_value}"));
                                }
                            });
                        }
                    >
                        {m_label}
                    </Button>
                }
            }).collect_view()}
        </div>
    }
}

#[component]
fn MonthYearSelects(
    search: RwSignal<ReceiptSearch>,
    years: Memo<Vec<SearchOption>>,
) -> impl IntoView {
    let selected_month = Memo::new(move |_| search.with(|s| s.date_inputs.month_value.clone()));
    let selected_year = Memo::new(move |_| search.with(|s| s.date_inputs.year_value.clone()));
    view! {
        <div class="flex items-center gap-1">
            <select
                class="h-9 px-3 border border-border-strong bg-surface rounded-lg text-label max-sm:min-h-11"
                aria-label=labels::PERIOD_YEAR
                prop:value=move || selected_year.get()
                on:change=move |ev| search.update(|s| s.select_year(event_target_value(&ev)))
            >
                <option value="">{labels::RECEIPT_YEAR_SELECT}</option>
                {move || {
                    years
                        .get()
                        .into_iter()
                        .map(|y| {
                            view! { <option value={y.value}>{y.label}</option> }
                        })
                        .collect_view()
                }}
            </select>
            <select
                class="h-9 px-3 border border-border-strong bg-surface rounded-lg text-label max-sm:min-h-11"
                aria-label=labels::MONTH
                prop:value=move || selected_month.get()
                on:change=move |ev| {
                    let month = event_target_value(&ev);
                    search.update(|s| {
                        s.date_inputs.month_value = month.clone();
                        let year = s.date_inputs.year_value.clone();
                        if year.is_empty() || month.is_empty() {
                            s.set_date_query(year);
                        } else {
                            s.set_date_query(format!("{year}-{month}"));
                        }
                    });
                }
            >
                <option value="">{labels::RECEIPT_MONTH_SELECT}</option>
                {(1..=12).map(|m| {
                    let m_str = format!("{m:02}");
                    let m_label = labels::receipt_month_label(m);
                    view! { <option value={m_str.clone()}>{m_label}</option> }
                }).collect_view()}
            </select>
        </div>
    }
}

#[component]
fn DatePicker(search: RwSignal<ReceiptSearch>) -> impl IntoView {
    let selected_date = Memo::new(move |_| search.with(|s| s.date_inputs.date_value.clone()));

    view! {
        <div class="mt-2 max-w-xs">
            <label class="block text-cell text-text-muted mb-1">{labels::DATE}</label>
            <input
                type="date"
                class="h-10 px-3 border border-border-strong bg-surface rounded-lg text-label w-full max-sm:min-h-11"
                aria-label=labels::DATE
                data-empty=move || selected_date.get().is_empty().then_some("true")
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
                <label class="block text-cell text-text-muted mb-1">{labels::START_DATE}</label>
                <input
                    type="date"
                    class="h-10 px-3 border border-border-strong bg-surface rounded-lg text-label w-full max-sm:min-h-11"
                    aria-label=labels::START_DATE
                    data-empty=move || range_start.get().is_empty().then_some("true")
                    prop:value=move || range_start.get()
                    on:input=move |ev| {
                        search.update(|s| {
                            s.date_inputs.range_start = event_target_value(&ev);
                            s.update_range_query();
                        });
                    }
                />
            </div>
            <span class="text-text-muted">{labels::RANGE_SEPARATOR}</span>
            <div class="flex-1 min-w-40">
                <label class="block text-cell text-text-muted mb-1">{labels::END_DATE}</label>
                <input
                    type="date"
                    class="h-10 px-3 border border-border-strong bg-surface rounded-lg text-label w-full max-sm:min-h-11"
                    aria-label=labels::END_DATE
                    data-empty=move || range_end.get().is_empty().then_some("true")
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
