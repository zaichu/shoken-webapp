use super::condition::{ConditionControls, ConditionKind, SearchCategories, search_category_memos};
use super::period::PeriodDraftContent;
use super::search_row::{ClearAllButton, ConditionTags, ReceiptSearchCount, SecuritiesInput};
use crate::features::receipts::filter::{ReceiptSearch, SearchKey};
use crate::features::receipts::{ReceiptsStore, ReceiptsTab};
use crate::support::list_search::SearchOption;
use crate::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::ui::labels;
use crate::ui::search_controls::{
    AddConditionButton, AddConditionRow, ConditionOptionList, ConditionPanel, ConditionRowEnd,
    SearchBox,
};
use leptos::prelude::*;

pub(crate) fn apply_period_draft(
    search: RwSignal<ReceiptSearch>,
    draft: &ReceiptSearch,
    has_years: bool,
) {
    search.update(|s| {
        if draft.date_all {
            s.clear_key(SearchKey::Date, has_years);
        } else {
            s.date_segment = draft.date_segment;
            s.date_inputs = draft.date_inputs.clone();
            s.date_all = false;
            s.selected_queries.date = draft.selected_queries.date.clone();
            s.rebuild_query();
        }
    });
}

#[component]
fn ReceiptAddConditionButton(
    kind: ConditionKind,
    open: RwSignal<Option<ConditionKind>>,
    node_ref: NodeRef<leptos::html::Button>,
) -> impl IntoView {
    let is_open = move || open.get() == Some(kind);
    view! {
        <AddConditionButton
            label=kind.label()
            expanded=move || open.get() == Some(kind)
            node_ref=node_ref
            on_click=move || {
                if is_open() {
                    open.set(None);
                } else {
                    open.set(Some(kind));
                }
            }
        />
    }
}

#[component]
fn ConditionPopover(
    kind: ConditionKind,
    search: RwSignal<ReceiptSearch>,
    years: Memo<Vec<SearchOption>>,
    products: Memo<Vec<SearchOption>>,
    accounts: Memo<Vec<SearchOption>>,
    has_years: impl Fn() -> bool + Copy + 'static + Send + Sync,
    on_close: impl Fn(ConditionKind) + Copy + 'static + Send + Sync,
) -> impl IntoView {
    let draft = RwSignal::new(search.get_untracked());

    let apply_period = move |_| {
        apply_period_draft(search, &draft.get_untracked(), has_years());
        on_close(kind);
    };
    let select_option = move |key: SearchKey, value: String| {
        search.update(|s| s.select_quick(key, value));
        on_close(kind);
    };
    let selected_accounts = single_selected(search, |s| s.selected_queries.accounts.clone());
    let selected_products = single_selected(search, |s| s.selected_queries.products.clone());

    view! {
        <ConditionPanel title=kind.title() on_close=move || on_close(kind)>
            {match kind {
                ConditionKind::Period => view! {
                    <PeriodDraftContent draft=draft years=years has_years=has_years />
                    <div class="condition-panel-actions">
                        <Button variant=ButtonVariant::Primary(ButtonSize::Md) on_click=apply_period>
                            {labels::RECEIPT_APPLY_PERIOD}
                        </Button>
                    </div>
                }
                    .into_any(),
                ConditionKind::Account => view! {
                    <ConditionOptionList
                        options=accounts
                        selected=selected_accounts
                        on_toggle=move |value| select_option(SearchKey::Accounts, value)
                    />
                }
                    .into_any(),
                ConditionKind::Product => view! {
                    <ConditionOptionList
                        options=products
                        selected=selected_products
                        on_toggle=move |value| select_option(SearchKey::Products, value)
                    />
                }
                    .into_any(),
            }}
        </ConditionPanel>
    }
}

fn single_selected(
    search: RwSignal<ReceiptSearch>,
    get: fn(&ReceiptSearch) -> String,
) -> Memo<Vec<String>> {
    Memo::new(move |_| {
        let value = search.with(get);
        if value.is_empty() {
            Vec::new()
        } else {
            vec![value]
        }
    })
}

#[component]
fn ConditionButtons(
    accounts: Memo<Vec<SearchOption>>,
    products: Memo<Vec<SearchOption>>,
    controls: ConditionControls,
) -> impl IntoView {
    view! {
        <ReceiptAddConditionButton kind=ConditionKind::Period open=controls.open node_ref=controls.period_ref />
        {move || {
            (!accounts.get().is_empty()).then(|| {
                view! {
                    <ReceiptAddConditionButton kind=ConditionKind::Account open=controls.open node_ref=controls.account_ref />
                }
            })
        }}
        {move || {
            (!products.get().is_empty()).then(|| {
                view! {
                    <ReceiptAddConditionButton kind=ConditionKind::Product open=controls.open node_ref=controls.product_ref />
                }
            })
        }}
    }
}

#[component]
pub fn ReceiptsSearchBar(
    store: ReceiptsStore,
    #[prop(into)] tab: Signal<ReceiptsTab>,
    #[prop(optional, default = "h-4 w-4")] icon_size: &'static str,
) -> impl IntoView {
    let search = store.search;
    let SearchCategories {
        years,
        products,
        accounts,
    } = search_category_memos(store, tab);
    let has_years = move || !years.get().is_empty();

    let controls = ConditionControls::new();
    let on_close = move |kind: ConditionKind| controls.finish(kind);

    view! {
        <div class="pc-search-area no-print">
            <SearchBox icon_size=icon_size>
                <ConditionTags search=search has_years=has_years />
                <SecuritiesInput search=search />
            </SearchBox>
            <AddConditionRow show_label=true>
                <ConditionButtons accounts=accounts products=products controls=controls />
                {move || {
                    controls.open.get().map(|kind| {
                        view! {
                            <ConditionPopover
                                kind=kind
                                search=search
                                years=years
                                products=products
                                accounts=accounts
                                has_years=has_years
                                on_close=on_close
                            />
                        }
                    })
                }}
                <ConditionRowEnd>
                    <ReceiptSearchCount store=store tab=tab />
                    <ClearAllButton search=search has_years=has_years />
                </ConditionRowEnd>
            </AddConditionRow>
        </div>
    }
}
