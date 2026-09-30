use super::pickers::{DatePeriod, SecurityDropdown, ToggleCategory, YearDropdown};
use crate::features::receipts::filter::{search_categories, DateSegment, ReceiptSearch, SearchKey};
use crate::features::receipts::{ReceiptTabData, ReceiptsStore, ReceiptsTab};
use crate::support::row::Row;
use crate::ui::collapsible_search_card::{CollapsibleSearchCard, SearchCardLayout};
use leptos::prelude::*;

#[component]
pub(crate) fn ReceiptsSearchCard(
    store: ReceiptsStore,
    tab: ReceiptsTab,
    data: ReceiptTabData,
    expanded: RwSignal<bool>,
) -> impl IntoView {
    let search = store.search;
    let display_store = store;
    let display_rows = Memo::new(move |_| match display_store.csv_state(tab).preview {
        Some(preview) if !preview.rows.is_empty() => {
            preview.rows.into_iter().map(Row::Preview).collect()
        }
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
            class="min-w-0"
            role="search"
            aria-label="取引明細の検索"
            data-testid="search-card"
        >
            <CollapsibleSearchCard
                expanded=expanded
                layout=SearchCardLayout::Toolbar
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
                on_expanded_change=Callback::new(move |(open,): (bool,)| {
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
