use crate::features::receipts::filter;
use crate::features::receipts::filter::{ReceiptSearch, SearchKey};
use crate::features::receipts::{ReceiptsStore, ReceiptsTab, TabState};
use crate::support::list_search::SearchOption;
use crate::ui::labels;
use leptos::prelude::*;

// 銘柄名・コードは別の入力欄で検索するため、条件数に含めない
pub(crate) const CONDITION_KEYS: [SearchKey; 3] =
    [SearchKey::Date, SearchKey::Products, SearchKey::Accounts];

pub(crate) fn condition_count(search: &ReceiptSearch) -> usize {
    CONDITION_KEYS
        .iter()
        .filter(|key| !search.selected_queries.get(**key).is_empty())
        .count()
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum ConditionKind {
    Period,
    Account,
    Product,
}

impl ConditionKind {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Period => labels::PERIOD,
            Self::Account => labels::ACCOUNT,
            Self::Product => labels::PRODUCT,
        }
    }

    pub(crate) fn title(self) -> String {
        labels::pick_title(self.label())
    }
}

#[derive(Clone, Copy)]
pub(crate) struct ConditionControls {
    pub open: RwSignal<Option<ConditionKind>>,
    pub period_ref: NodeRef<leptos::html::Button>,
    pub account_ref: NodeRef<leptos::html::Button>,
    pub product_ref: NodeRef<leptos::html::Button>,
}

impl ConditionControls {
    pub(crate) fn new() -> Self {
        Self {
            open: RwSignal::new(None),
            period_ref: NodeRef::new(),
            account_ref: NodeRef::new(),
            product_ref: NodeRef::new(),
        }
    }

    pub(crate) fn finish(&self, kind: ConditionKind) {
        self.open.set(None);
        let reference = match kind {
            ConditionKind::Period => self.period_ref,
            ConditionKind::Account => self.account_ref,
            ConditionKind::Product => self.product_ref,
        };
        if let Some(button) = reference.get() {
            let _ = button.focus();
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct SearchCategories {
    pub years: Memo<Vec<SearchOption>>,
    pub products: Memo<Vec<SearchOption>>,
    pub accounts: Memo<Vec<SearchOption>>,
}

impl SearchCategories {
    pub(crate) fn has_years(&self) -> bool {
        !self.years.get().is_empty()
    }
}

pub(crate) fn search_category_memos(
    store: ReceiptsStore,
    tab: Signal<ReceiptsTab>,
) -> SearchCategories {
    let display_rows = Memo::new(move |_| match store.tab_state(tab.get()) {
        TabState::Ready(data) => data.rows.clone(),
        _ => Vec::new(),
    });
    let categories = Memo::new(move |_| filter::search_categories(tab.get(), &display_rows.get()));
    SearchCategories {
        years: Memo::new(move |_| categories.with(|c| c.years.clone())),
        products: Memo::new(move |_| categories.with(|c| c.products.clone())),
        accounts: Memo::new(move |_| categories.with(|c| c.accounts.clone())),
    }
}
