use super::condition::condition_count;
use crate::features::receipts::filter;
use crate::features::receipts::filter::{ReceiptSearch, SearchKey};
use crate::features::receipts::kind;
use crate::features::receipts::{ReceiptsStore, ReceiptsTab, TabState};
use crate::ui::badge;
use crate::ui::labels;
use crate::ui::search_controls::{SearchClearButton, SearchCount, SearchTag, SearchTextInput};
use leptos::prelude::*;

pub(crate) fn date_tag_text(value: &str) -> String {
    if let Some((start, end)) = value.split_once("..") {
        let (start, end) = (date_part_text(start), date_part_text(end));
        return match (start.is_empty(), end.is_empty()) {
            (true, _) => labels::receipt_range_until(&end),
            (_, true) => labels::receipt_range_from(&start),
            (false, false) => labels::receipt_range_between(&start, &end),
        };
    }
    date_part_text(value)
}

fn date_part_text(value: &str) -> String {
    if value.len() == 4 {
        labels::receipt_year_label(value)
    } else {
        kind::group_label(value)
    }
}

#[component]
pub(crate) fn ConditionTags(
    search: RwSignal<ReceiptSearch>,
    has_years: impl Fn() -> bool + Copy + 'static + Send + Sync,
) -> impl IntoView {
    let date_value =
        Memo::new(move |_| date_tag_text(&search.with(|s| s.selected_queries.date.clone())));
    let products_value = Memo::new(move |_| search.with(|s| s.selected_queries.products.clone()));
    let accounts_value = Memo::new(move |_| {
        search.with(|s| {
            badge::account_display_name(s.selected_queries.accounts.trim_matches('"')).to_string()
        })
    });
    view! {
        <SearchTag
            label=labels::PERIOD
            value=date_value
            on_clear=move || search.update(|s| s.clear_key(SearchKey::Date, has_years()))
        />
        <SearchTag
            label=labels::PRODUCT
            value=products_value
            on_clear=move || search.update(|s| s.clear_key(SearchKey::Products, has_years()))
        />
        <SearchTag
            label=labels::ACCOUNT
            value=accounts_value
            on_clear=move || search.update(|s| s.clear_key(SearchKey::Accounts, has_years()))
        />
    }
}

#[component]
pub(crate) fn SecuritiesInput(search: RwSignal<ReceiptSearch>) -> impl IntoView {
    view! {
        <SearchTextInput
            placeholder=labels::search_placeholder(labels::RECEIPT_SEARCH_TARGET)
            value=move || search.with(|s| s.selected_queries.get(SearchKey::Securities).to_string())
            on_input=move |value| search.update(|s| s.set_securities_query(value))
        />
    }
}

#[component]
pub(crate) fn ReceiptSearchCount(
    store: ReceiptsStore,
    #[prop(into)] tab: Signal<ReceiptsTab>,
) -> impl IntoView {
    let count = Memo::new(move |_| match store.tab_state(tab.get()) {
        TabState::Ready(data) => {
            let query = store.search.with(|s| s.query.clone());
            let total = data.total.unwrap_or(data.rows.len());
            Some((
                filter::filter_receipts(tab.get(), &data.rows, &query).len(),
                total,
            ))
        }
        _ => None,
    });
    move || {
        count.get().map(|(filtered, total)| {
            view! {
                <SearchCount filtered=move || filtered total=move || total />
            }
        })
    }
}

#[component]
pub(crate) fn ClearAllButton(
    search: RwSignal<ReceiptSearch>,
    has_years: impl Fn() -> bool + Copy + 'static + Send + Sync,
) -> impl IntoView {
    let nothing_to_clear = move || {
        search.with(|s| condition_count(s) == 0 && s.selected_queries.securities.is_empty())
    };
    view! {
        <SearchClearButton
            disabled=nothing_to_clear
            on_clear=move || search.update(|s| s.clear(has_years()))
        />
    }
}
