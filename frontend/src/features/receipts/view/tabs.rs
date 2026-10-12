use super::TAB_IDS;
use crate::features::receipts::view::ReceiptsMainContent;
use crate::features::receipts::{ReceiptsStore, ReceiptsTab, TabState};
use crate::ui::state::{ListSkeleton, ListSkeletonVariant, Loading};
use leptos::prelude::*;
use wasm_bindgen::JsCast;

pub fn next_tab_index(current: usize, key: &str) -> Option<usize> {
    let count = TAB_IDS.len();
    match key {
        "ArrowRight" => Some((current + 1) % count),
        "ArrowLeft" => Some((current + count - 1) % count),
        "Home" => Some(0),
        "End" => Some(count - 1),
        _ => None,
    }
}

fn focus_tab(index: usize) {
    let Some(element) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id(&format!("tab-{}", TAB_IDS[index])))
    else {
        return;
    };
    if let Some(element) = element.dyn_ref::<web_sys::HtmlElement>() {
        let _ = element.focus();
    }
}

#[component]
pub fn ReceiptsTabButton(store: ReceiptsStore, tab: ReceiptsTab) -> impl IntoView {
    let slug = TAB_IDS[tab as usize];
    let label = tab.label();
    let selected = store;
    let keyed = store;
    let clicked = store;
    let counted_store = store;
    view! {
        <button
            type="button"
            role="tab"
            id=format!("tab-{slug}")
            aria-controls=format!("tabpanel-{slug}")
            attr:aria-selected=move || (selected.active_tab.get() == tab).to_string()
            tabindex=move || if selected.active_tab.get() == tab { "0" } else { "-1" }
            class="receipts-tab"
            on:click=move |_| clicked.select_tab(tab)
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
                class="receipts-tab-count"
            >
                {move || match counted_store.tab_state(tab) {
                    TabState::Ready(data) => data.rows.len().to_string(),
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
