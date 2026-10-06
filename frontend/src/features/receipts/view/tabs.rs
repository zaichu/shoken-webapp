use super::TAB_IDS;
use super::workspace::ReceiptMainColumn;
use crate::features::receipts::{ReceiptsStore, ReceiptsTab, TabState};
use crate::ui::state::{ListSkeleton, ListSkeletonVariant, Loading};
use crate::ui::tabs::TabButton;
use leptos::prelude::*;
use wasm_bindgen::JsCast;

const TAB_COUNT_BASE: &str = "inline-flex min-w-6 items-center justify-center rounded-full px-2 py-0.5 text-xs font-semibold tabular-nums";
const TAB_COUNT_ACTIVE: &str = "border border-text-inverse/20 bg-surface text-ink";
const TAB_COUNT_INACTIVE: &str = "border border-border-subtle bg-surface text-text-soft";

pub(crate) fn next_tab_index(current: usize, key: &str) -> Option<usize> {
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

pub(crate) fn scroll_tab_into_view(tab: ReceiptsTab) {
    let Some(element) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id(&format!("tab-{}", TAB_IDS[tab as usize])))
    else {
        return;
    };
    let options = web_sys::ScrollIntoViewOptions::new();
    options.set_block(web_sys::ScrollLogicalPosition::Nearest);
    options.set_inline(web_sys::ScrollLogicalPosition::Nearest);
    element.scroll_into_view_with_scroll_into_view_options(&options);
}

#[component]
pub(crate) fn ReceiptsTabButton(store: ReceiptsStore, tab: ReceiptsTab) -> impl IntoView {
    let slug = TAB_IDS[tab as usize];
    let label = tab.label();
    let selected = store;
    let keyed = store;
    let clicked = store;
    let counted = store;
    let counted_store = store;
    view! {
        <TabButton
            id=format!("tab-{slug}")
            controls=format!("tabpanel-{slug}")
            selected=Signal::derive(move || selected.active_tab.get() == tab)
            on_select=move || clicked.select_tab(tab)
            on_keydown=move |event| {
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
                class=move || {
                    format!(
                        "{TAB_COUNT_BASE} {}",
                        if counted.active_tab.get() == tab {
                            TAB_COUNT_ACTIVE
                        } else {
                            TAB_COUNT_INACTIVE
                        }
                    )
                }
            >
                {move || match counted_store.tab_state(tab) {
                    TabState::Ready(data) => data.rows.len().to_string(),
                    TabState::Loading | TabState::Failed(_) => "—".to_string(),
                }}
            </span>
        </TabButton>
    }
}

#[component]
pub(crate) fn TabPanel(
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
                view! { <ReceiptMainColumn store=store tab=tab /> }.into_any()
            }}
        </div>
    }
}
