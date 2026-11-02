use super::TAB_IDS;
use crate::features::receipts::filter::filter_receipts;
use crate::features::receipts::view::ReceiptsMainContent;
use crate::features::receipts::{ReceiptsStore, ReceiptsTab, TabState};
use crate::ui::state::{ListSkeleton, ListSkeletonVariant};
use crate::ui::underline_tabs::{
    SwipeMotion, focus_tab_by_id, next_tab_index as shared_next_tab_index, prefers_reduced_motion,
    swipe_should_ignore_input,
};
use leptos::prelude::*;

pub fn next_tab_index(current: usize, key: &str) -> Option<usize> {
    shared_next_tab_index(TAB_IDS.len(), current, key)
}

fn focus_tab(index: usize) {
    focus_tab_by_id(&format!("tab-{}", TAB_IDS[index]));
}

#[component]
pub fn ReceiptsTabButton(
    store: ReceiptsStore,
    tab: ReceiptsTab,
    motion: SwipeMotion,
) -> impl IntoView {
    let slug = TAB_IDS[tab as usize];
    let label = tab.label();
    let selected = store;
    let keyed = store;
    let clicked = store;
    let guard = motion;
    let counted_store = store;
    view! {
        <button
            type="button"
            role="tab"
            id=format!("tab-{slug}")
            aria-controls=format!("tabpanel-{slug}")
            aria-selected=move || (selected.active_tab.get() == tab).to_string()
            tabindex=move || if selected.active_tab.get() == tab { "0" } else { "-1" }
            class="page-tab"
            on:click=move |event| {
                if event.detail() != 0
                    && swipe_should_ignore_input(
                        guard.settling.get_untracked(),
                        prefers_reduced_motion(),
                    )
                {
                    return;
                }
                clicked.select_tab(tab);
            }
            on:keydown=move |event| {
                let current = ReceiptsTab::ALL
                    .iter()
                    .position(|item| *item == keyed.active_tab.get_untracked())
                    .unwrap_or(0);
                if let Some(next) = next_tab_index(current, &event.key()) {
                    event.prevent_default();
                    keyed.select_tab(ReceiptsTab::ALL[next]);
                    focus_tab(next);
                }
            }
        >
            {label}
            <span
                data-testid={format!("tab-count-{slug}")}
                class="page-tab-count"
            >
                {move || match counted_store.tab_state(tab) {
                    TabState::Ready(data) => {
                        let query = counted_store.search.with(|s| s.query.clone());
                        filter_receipts(tab, &data.rows, &query).len().to_string()
                    }
                    TabState::Loading | TabState::Failed(_) => "—".to_string(),
                }}
            </span>
        </button>
    }
}

#[component]
pub fn TabPanel(
    store: ReceiptsStore,
    tab: ReceiptsTab,
    #[prop(optional)] loading: bool,
    motion: SwipeMotion,
) -> impl IntoView {
    let slug = TAB_IDS[tab as usize];
    let index = ReceiptsTab::ALL
        .iter()
        .position(|item| *item == tab)
        .unwrap_or(0);
    let shown = Memo::new(move |_| {
        let current = store.active_tab.get();
        let position = ReceiptsTab::ALL
            .iter()
            .position(|item| *item == current)
            .unwrap_or(0);
        current == tab || (motion.dragging.get() && position.abs_diff(index) == 1)
    });
    view! {
        <div
            id={format!("tabpanel-{slug}")}
            role="tabpanel"
            aria-labelledby={format!("tab-{slug}")}
            class="tab-swipe-slide"
            data-index=index.to_string()
            data-active=move || (store.active_tab.get() == tab).to_string()
            hidden=move || !shown.get()
        >
            {move || {
                if !shown.get() {
                    return ().into_any();
                }
                let store = store;
                if loading {
                    return view! { <ListSkeleton variant=ListSkeletonVariant::Table /> }.into_any();
                }
                if !store.is_authenticated() {
                    return view! { <ListSkeleton variant=ListSkeletonVariant::Table /> }.into_any();
                }
                view! { <ReceiptsMainContent store=store tab=tab /> }.into_any()
            }}
        </div>
    }
}
