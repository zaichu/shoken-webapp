use super::workspace::ReceiptWorkspace;
use super::TAB_IDS;
use crate::components::ui::Loading;
use crate::receipts::{ReceiptsStore, ReceiptsTab};
use leptos::prelude::*;
use wasm_bindgen::JsCast;

const TAB_BUTTON_BASE: &str = "inline-flex items-center gap-2 rounded-lg border px-4 py-2.5 text-sm font-bold max-sm:min-h-[44px] max-sm:shrink-0 max-sm:px-3";
const TAB_BUTTON_ACTIVE: &str =
    "border-slate-950 bg-slate-950 text-white shadow-[inset_0_-2px_0_#f59e0b]";
const TAB_BUTTON_INACTIVE: &str =
    "border-slate-300 bg-white text-slate-800 hover:border-slate-400 hover:bg-white hover:text-slate-950";
const TAB_COUNT_BASE: &str =
    "inline-flex min-w-6 items-center justify-center rounded-full px-2 py-0.5 text-xs font-semibold";
const TAB_COUNT_ACTIVE: &str = "border border-white/20 bg-white text-slate-950";
const TAB_COUNT_INACTIVE: &str = "border border-slate-200 bg-white text-slate-700";

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
pub(crate) fn TabButton(store: ReceiptsStore, tab: ReceiptsTab) -> impl IntoView {
    let slug = TAB_IDS[tab as usize];
    let label = tab.label();
    let selected = store.clone();
    let keyed = store.clone();
    let clicked = store.clone();
    let counted = store.clone();
    let counted_store = store.clone();
    view! {
        <button
            id={format!("tab-{slug}")}
            type="button"
            role="tab"
            class=move || {
                format!(
                    "{TAB_BUTTON_BASE} {}",
                    if selected.active_tab.get() == tab {
                        TAB_BUTTON_ACTIVE
                    } else {
                        TAB_BUTTON_INACTIVE
                    }
                )
            }
            aria-selected=move || {
                if selected.active_tab.get() == tab {
                    "true"
                } else {
                    "false"
                }
            }
            aria-controls={format!("tabpanel-{slug}")}
            tabindex=move || if selected.active_tab.get() == tab { "0" } else { "-1" }
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
                {move || counted_store.count(tab).to_string()}
            </span>
        </button>
    }
}

#[component]
pub(crate) fn TabPanel(store: ReceiptsStore, tab: ReceiptsTab) -> impl IntoView {
    let slug = TAB_IDS[tab as usize];
    let hidden = store.clone();
    let rendered = store.clone();
    view! {
        <div
            id={format!("tabpanel-{slug}")}
            role="tabpanel"
            aria-labelledby={format!("tab-{slug}")}
            hidden=move || hidden.active_tab.get() != tab
        >
            {move || {
                let store = rendered.clone();
                if store.active_tab.get() != tab {
                    return ().into_any();
                }
                if !store.is_authenticated() {
                    return view! { <Loading /> }.into_any();
                }
                view! { <ReceiptWorkspace store=store tab=tab /> }.into_any()
            }}
        </div>
    }
}
