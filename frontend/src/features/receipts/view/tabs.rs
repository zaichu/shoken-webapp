use super::TAB_IDS;
use crate::features::receipts::filter::filter_receipts;
use crate::features::receipts::view::ReceiptsMainContent;
use crate::features::receipts::{ReceiptsStore, ReceiptsTab, TabState};
use crate::ui::state::{ListSkeleton, ListSkeletonVariant, Loading};
use crate::ui::underline_tabs::{UnderlineTabButton, focus_tab_by_id, next_tab_index};
use leptos::prelude::*;

pub fn next_receipt_tab_index(current: usize, key: &str) -> Option<usize> {
    next_tab_index(TAB_IDS.len(), current, key)
}

fn focus_tab(index: usize) {
    focus_tab_by_id(&format!("tab-{}", TAB_IDS[index]));
}

#[component]
pub fn ReceiptsTabButton(store: ReceiptsStore, tab: ReceiptsTab) -> impl IntoView {
    let slug = TAB_IDS[tab as usize];
    let label = tab.label();
    let selected = Signal::derive(move || store.active_tab.get() == tab);
    let count = Signal::derive(move || match store.tab_state(tab) {
        TabState::Ready(data) => {
            let query = store.search.with(|s| s.query.clone());
            filter_receipts(tab, &data.rows, &query).len().to_string()
        }
        TabState::Loading | TabState::Failed(_) => "—".to_string(),
    });
    let keyed = store;
    let clicked = store;
    view! {
        <UnderlineTabButton
            id=format!("tab-{slug}")
            controls=format!("tabpanel-{slug}")
            label=label.to_string()
            count=count
            selected=selected
            count_testid=format!("tab-count-{slug}")
            on_select=move || clicked.select_tab(tab)
            on_keydown=move |event: web_sys::KeyboardEvent| {
                let current = ReceiptsTab::ALL
                    .iter()
                    .position(|item| *item == keyed.active_tab.get_untracked())
                    .unwrap_or(0);
                if let Some(next) = next_receipt_tab_index(current, &event.key()) {
                    event.prevent_default();
                    keyed.select_tab(ReceiptsTab::ALL[next]);
                    focus_tab(next);
                }
            }
        />
    }
}

#[component]
pub fn TabPanel(
    store: ReceiptsStore,
    tab: ReceiptsTab,
    #[prop(optional)] loading: bool,
) -> impl IntoView {
    let slug = TAB_IDS[tab as usize];
    let hidden = store;
    let rendered = store;
    view! {
        <div
            id={format!("tabpanel-{slug}")}
            role="tabpanel"
            aria-labelledby={format!("tab-{slug}")}
            hidden=move || hidden.active_tab.get() != tab
        >
            {move || {
                let store = rendered;
                if store.active_tab.get() != tab {
                    return ().into_any();
                }
                if loading {
                    return view! { <ListSkeleton variant=ListSkeletonVariant::Table /> }.into_any();
                }
                if !store.is_authenticated() {
                    return view! { <Loading /> }.into_any();
                }
                view! { <ReceiptsMainContent store=store tab=tab /> }.into_any()
            }}
        </div>
    }
}
