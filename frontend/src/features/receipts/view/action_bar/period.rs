use crate::features::receipts::filter::{DateSegment, ReceiptSearch, SearchKey};
use crate::support::list_search::SearchOption;
use crate::ui::button::{Button, ButtonVariant};
use crate::ui::labels;
use crate::ui::receipts::{
    PeriodDateField, PeriodFlatOptions, PeriodNativeSelect, PeriodOptionWrap, PeriodRangeField,
    PeriodRangeWrap, PeriodYearMonthRow,
};
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
        <PeriodOptionWrap>
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
        </PeriodOptionWrap>
    }
}

#[component]
fn MonthPicker(search: RwSignal<ReceiptSearch>, years: Memo<Vec<SearchOption>>) -> impl IntoView {
    view! {
        <PeriodOptionWrap>
            {move || {
                if years.get().is_empty() {
                    view! { <MonthButtonsWithoutYears search=search /> }.into_any()
                } else {
                    view! { <MonthYearSelects search=search years=years /> }.into_any()
                }
            }}
        </PeriodOptionWrap>
    }
}

#[component]
fn MonthButtonsWithoutYears(search: RwSignal<ReceiptSearch>) -> impl IntoView {
    let months = (1..=12).map(|m| format!("{m:02}")).collect::<Vec<_>>();
    let selected_month = Memo::new(move |_| search.with(|s| s.date_inputs.month_value.clone()));
    view! {
        <PeriodFlatOptions>
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
        </PeriodFlatOptions>
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
        <PeriodYearMonthRow>
            <PeriodNativeSelect
                aria_label=labels::PERIOD_YEAR.to_string()
                value=selected_year
                on_change=move |value: String| search.update(|s| s.select_year(value))
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
            </PeriodNativeSelect>
            <PeriodNativeSelect
                aria_label=labels::MONTH.to_string()
                value=selected_month
                on_change=move |month: String| {
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
            </PeriodNativeSelect>
        </PeriodYearMonthRow>
    }
}

#[component]
fn DatePicker(search: RwSignal<ReceiptSearch>) -> impl IntoView {
    let selected_date = Memo::new(move |_| search.with(|s| s.date_inputs.date_value.clone()));

    view! {
        <PeriodDateField
            label=labels::DATE
            value=selected_date
            on_input=move |value: String| {
                search.update(|s| {
                    s.date_inputs.date_value = value.clone();
                    s.set_date_query(value);
                });
            }
        />
    }
}

#[component]
fn RangePicker(search: RwSignal<ReceiptSearch>) -> impl IntoView {
    let range_start = Memo::new(move |_| search.with(|s| s.date_inputs.range_start.clone()));
    let range_end = Memo::new(move |_| search.with(|s| s.date_inputs.range_end.clone()));

    view! {
        <PeriodRangeWrap>
            <PeriodRangeField
                label=labels::START_DATE
                value=range_start
                on_input=move |value: String| {
                    search.update(|s| {
                        s.date_inputs.range_start = value;
                        s.update_range_query();
                    });
                }
            />
            <span class="text-text-muted">{labels::RANGE_SEPARATOR}</span>
            <PeriodRangeField
                label=labels::END_DATE
                value=range_end
                on_input=move |value: String| {
                    search.update(|s| {
                        s.date_inputs.range_end = value;
                        s.update_range_query();
                    });
                }
            />
        </PeriodRangeWrap>
    }
}
