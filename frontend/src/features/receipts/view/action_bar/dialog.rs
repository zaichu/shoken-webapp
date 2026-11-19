use super::condition::search_category_memos;
use super::search_row::{ConditionTags, ReceiptSearchCount};
use crate::features::receipts::filter::{DateSegment, ReceiptSearch, ReceiptSortOrder, SearchKey};
use crate::features::receipts::{ReceiptsStore, ReceiptsTab};
use crate::support::list_search::SearchOption;
use crate::ui::labels;
use crate::ui::more_link::MoreLink;
use crate::ui::receipts::{ReceiptDialogFooter, ReceiptDialogLabel, ReceiptDialogStack};
use crate::ui::search_controls::{SearchBox, SearchTextInput};
use crate::ui::search_dialog::SearchDialog;
use crate::ui::search_entry::MobileSearchEntryShell;
use crate::ui::select::Select;
use leptos::prelude::*;

pub(crate) fn apply_receipt_sheet_draft(search: RwSignal<ReceiptSearch>, draft: &ReceiptSearch) {
    search.set(draft.clone());
}

pub(crate) fn clear_receipt_dialog_draft(draft: RwSignal<ReceiptSearch>, has_years: bool) {
    draft.update(|s| s.clear(has_years));
}

pub(crate) const fn receipt_date_label(tab: ReceiptsTab) -> &'static str {
    match tab {
        ReceiptsTab::Dividend => labels::PAYMENT_DATE,
        ReceiptsTab::DomesticStock | ReceiptsTab::MutualFund => labels::TRADE_DATE,
    }
}

#[component]
fn DialogFieldSelect(
    label: &'static str,
    testid: &'static str,
    #[prop(into)] value: Signal<String>,
    #[prop(into)] options: Signal<Vec<SearchOption>>,
    on_select: impl Fn(String) + Copy + Send + Sync + 'static,
) -> impl IntoView {
    view! {
        <div class="search-dialog-field">
            <ReceiptDialogLabel>
                {label}
                <Select
                    value=value
                    aria_label=label.to_string()
                    testid=testid
                    class="search-dialog-select".to_string()
                    on_change=on_select
                >
                    <option value="">{labels::ALL}</option>
                    {move || {
                        options
                            .get()
                            .into_iter()
                            .map(|item| {
                                view! { <option value={item.value}>{item.label}</option> }
                            })
                            .collect_view()
                    }}
                </Select>
            </ReceiptDialogLabel>
        </div>
    }
}

#[component]
fn DialogSearchSection(
    draft: RwSignal<ReceiptSearch>,
    on_search: impl Fn() + Copy + 'static + Send + Sync,
) -> impl IntoView {
    let draft_text = move || draft.with(|s| s.selected_queries.securities.clone());
    let set_draft_text = move |value: String| draft.update(|s| s.set_securities_query(value));
    let search_callback = Callback::new(move |()| on_search());
    view! {
        <section aria-label=labels::SEARCH>
            <p class="search-dialog-section-title">{labels::SEARCH}</p>
            <div class="search-dialog-field">
                <SearchBox>
                    <SearchTextInput
                        placeholder=labels::search_placeholder(labels::RECEIPT_SEARCH_TARGET)
                        testid="receipts-search-input"
                        value=draft_text
                        on_input=set_draft_text
                        on_enter=search_callback
                    />
                </SearchBox>
                <p class="search-dialog-hint">{labels::search_placeholder(labels::RECEIPT_SEARCH_TARGET)}</p>
            </div>
        </section>
    }
}

#[component]
fn DialogConditionSection(
    draft: RwSignal<ReceiptSearch>,
    years: Memo<Vec<SearchOption>>,
    products: Memo<Vec<SearchOption>>,
    accounts: Memo<Vec<SearchOption>>,
    has_years: impl Fn() -> bool + Copy + 'static + Send + Sync,
) -> impl IntoView {
    let period_value = Memo::new(move |_| {
        let date = draft.with(|s| {
            if s.date_all {
                String::new()
            } else {
                s.selected_queries.date.clone()
            }
        });
        if date.is_empty() {
            return String::new();
        }
        let is_year = years.get().into_iter().any(|item| item.value == date);
        if is_year { date } else { String::new() }
    });
    let account_value = Memo::new(move |_| draft.with(|s| s.selected_queries.accounts.clone()));
    let product_value = Memo::new(move |_| draft.with(|s| s.selected_queries.products.clone()));
    view! {
        <section aria-label=labels::CONDITION>
            <p class="search-dialog-section-title">{labels::CONDITION}</p>
            <DialogFieldSelect
                label=labels::PERIOD
                testid="receipts-dialog-period"
                value=period_value
                options=years
                on_select=move |next: String| {
                    draft.update(|s| {
                        if next.is_empty() {
                            s.clear_key(SearchKey::Date, has_years());
                        } else {
                            s.date_segment = DateSegment::Year;
                            s.date_all = false;
                            s.select_year(next);
                        }
                    });
                }
            />
            {move || {
                (!accounts.get().is_empty())
                    .then(|| {
                        view! {
                            <DialogFieldSelect
                                label=labels::ACCOUNT
                                testid="receipts-dialog-account"
                                value=account_value
                                options=accounts
                                on_select=move |next: String| {
                                    draft.update(|s| {
                                        if next.is_empty() {
                                            s.clear_key(SearchKey::Accounts, has_years());
                                        } else {
                                            s.selected_queries.accounts = next;
                                            s.rebuild_query();
                                        }
                                    });
                                }
                            />
                        }
                    })
            }}
            {move || {
                (!products.get().is_empty())
                    .then(|| {
                        view! {
                            <DialogFieldSelect
                                label=labels::PRODUCT
                                testid="receipts-dialog-product"
                                value=product_value
                                options=products
                                on_select=move |next: String| {
                                    draft.update(|s| {
                                        if next.is_empty() {
                                            s.clear_key(SearchKey::Products, has_years());
                                        } else {
                                            s.selected_queries.products = next;
                                            s.rebuild_query();
                                        }
                                    });
                                }
                            />
                        }
                    })
            }}
        </section>
    }
}

#[component]
fn DialogSortSection(
    draft_order: RwSignal<ReceiptSortOrder>,
    #[prop(into)] tab: Signal<ReceiptsTab>,
) -> impl IntoView {
    let date_label = move || receipt_date_label(tab.get());
    view! {
        <section aria-label=labels::SORT_BY.to_string()>
            <p class="search-dialog-section-title">{labels::SORT_BY}</p>
            <div class="search-dialog-sort-row">
                <div>
                    <ReceiptDialogLabel>
                        {labels::SORT_TARGET}
                        <Select
                            value=Signal::derive(move || "date".to_string())
                            aria_label=labels::SORT_TARGET.to_string()
                            testid="receipts-dialog-sort-target"
                            class="search-dialog-select".to_string()
                            on_change=move |_| {}
                        >
                            <option value="date">{date_label}</option>
                        </Select>
                    </ReceiptDialogLabel>
                </div>
                <div>
                    <ReceiptDialogLabel>
                        {labels::SORT_ORDER_LABEL}
                        <Select
                            value=Signal::derive(move || draft_order.get().value().to_string())
                            aria_label=labels::SORT_ORDER_LABEL.to_string()
                            testid="receipts-dialog-sort-order"
                            class="search-dialog-select".to_string()
                            on_change=move |next: String| {
                                if let Some(order) = ReceiptSortOrder::from_value(&next) {
                                    draft_order.set(order);
                                }
                            }
                        >
                            <option value="newest">{labels::SORT_NEWEST_FIRST}</option>
                            <option value="oldest">{labels::SORT_OLDEST_FIRST}</option>
                        </Select>
                    </ReceiptDialogLabel>
                </div>
            </div>
        </section>
    }
}

#[component]
fn ReceiptDialogFields(
    draft: RwSignal<ReceiptSearch>,
    draft_order: RwSignal<ReceiptSortOrder>,
    #[prop(into)] tab: Signal<ReceiptsTab>,
    years: Memo<Vec<SearchOption>>,
    products: Memo<Vec<SearchOption>>,
    accounts: Memo<Vec<SearchOption>>,
    has_years: impl Fn() -> bool + Copy + 'static + Send + Sync,
    on_search: impl Fn() + Copy + 'static + Send + Sync,
) -> impl IntoView {
    view! {
        <ReceiptDialogStack>
            <DialogSearchSection draft=draft on_search=on_search />
            <DialogConditionSection
                draft=draft
                years=years
                products=products
                accounts=accounts
                has_years=has_years
            />
            <DialogSortSection draft_order=draft_order tab=tab />
        </ReceiptDialogStack>
    }
}

#[component]
fn ReceiptSearchDialog(
    search: RwSignal<ReceiptSearch>,
    sort_order: RwSignal<ReceiptSortOrder>,
    #[prop(into)] tab: Signal<ReceiptsTab>,
    years: Memo<Vec<SearchOption>>,
    products: Memo<Vec<SearchOption>>,
    accounts: Memo<Vec<SearchOption>>,
    has_years: impl Fn() -> bool + Copy + 'static + Send + Sync,
    on_close: impl Fn() + Copy + 'static + Send + Sync,
) -> impl IntoView {
    let draft = RwSignal::new(search.get_untracked());
    let draft_order = RwSignal::new(sort_order.get_untracked());
    let do_search = move || {
        apply_receipt_sheet_draft(search, &draft.get_untracked());
        sort_order.set(draft_order.get_untracked());
        on_close();
    };
    let years_flag = has_years();
    let do_clear = move || clear_receipt_dialog_draft(draft, years_flag);
    view! {
        <SearchDialog
            title=labels::SEARCH_AND_SORT.to_string()
            on_close=on_close
            on_search=do_search
            on_clear=do_clear
            testid="receipts-search-sheet"
        >
            <ReceiptDialogFields
                draft=draft
                draft_order=draft_order
                tab=tab
                years=years
                products=products
                accounts=accounts
                has_years=has_years
                on_search=do_search
            />
        </SearchDialog>
    }
}

#[component]
pub fn ReceiptsMobileSearchEntry(store: ReceiptsStore) -> impl IntoView {
    let tab = store.active_tab;
    let search = store.search;
    let sort_order = store.sort_order;
    let open = RwSignal::new(false);

    let categories = search_category_memos(store, tab.into());
    let years = categories.years;
    let products = categories.products;
    let accounts = categories.accounts;
    let has_years = move || categories.has_years();
    let sort_summary = move || {
        let date = receipt_date_label(tab.get());
        match sort_order.get() {
            ReceiptSortOrder::Newest => labels::sort_summary(date, labels::SORT_NEWEST_FIRST),
            ReceiptSortOrder::Oldest => format!("{date} {}", labels::SORT_OLDEST_FIRST),
        }
    };

    let sort_summary = Signal::derive(sort_summary);
    let placeholder = labels::search_placeholder(labels::RECEIPT_SEARCH_TARGET);
    let close = move || open.set(false);
    let sheet_placeholder = placeholder.clone();
    view! {
        <MobileSearchEntryShell
            entry_testid="receipts-search-sheet-entry"
            search_label=labels::SEARCH.to_string()
            placeholder=sheet_placeholder.clone()
            value=Signal::derive(move || search.with(|s| s.selected_queries.securities.clone()))
            on_input=Callback::new(move |value| search.update(|s| s.set_securities_query(value)))
            search_input_testid="receipts-search-input-inline"
            sort_summary=sort_summary
            open=open
        >
            <ConditionTags search=search has_years=has_years />
            <ReceiptDialogFooter>
                <ReceiptSearchCount store=store tab=tab tax_note=true />
                <MoreLink
                    href="/csv".to_string()
                    label=labels::CSV_MANAGE_TITLE.to_string()
                    testid="receipts-csv-link".to_string()
                />
            </ReceiptDialogFooter>
        </MobileSearchEntryShell>
        {move || {
            open.get()
                .then(|| {
                    view! {
                        <ReceiptSearchDialog
                            search=search
                            sort_order=sort_order
                            tab=tab
                            years=years
                            products=products
                            accounts=accounts
                            has_years=has_years
                            on_close=close
                        />
                    }
                })
        }}
    }
}
