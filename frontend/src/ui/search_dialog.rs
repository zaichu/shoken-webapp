use crate::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::ui::focus::trap_focus;
use crate::ui::labels::{DIALOG_CLOSE, SEARCH, SHEET_CLEAR};
use leptos::prelude::*;
use wasm_bindgen::JsCast;

#[component]
pub fn SearchDialog(
    #[prop(into)] title: String,
    on_close: impl Fn() + Clone + 'static,
    on_search: impl Fn() + Clone + 'static,
    on_clear: impl Fn() + Clone + 'static,
    #[prop(optional)] testid: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    let panel_ref = NodeRef::<leptos::html::Div>::new();
    let overlay_close = on_close.clone();
    let key_close = on_close.clone();
    let footer_close = on_close.clone();
    let dialog_title = title.clone();
    let panel_testid = testid.unwrap_or("search-dialog");
    Effect::new(move |_| {
        let (body_overflow, previous_focus) = web_sys::window()
            .and_then(|window| window.document())
            .map(|document| {
                let previous = document.active_element();
                let overflow = document
                    .body()
                    .and_then(|body| body.style().get_property_value("overflow").ok());
                if let Some(body) = document.body() {
                    let _ = body.style().set_property("overflow", "hidden");
                }
                (overflow.unwrap_or_default(), previous)
            })
            .unwrap_or_default();
        if let Some(panel) = panel_ref.get() {
            let _ = panel.focus();
        }
        on_cleanup(move || {
            if let Some(document) = web_sys::window().and_then(|window| window.document()) {
                if let Some(body) = document.body() {
                    if body_overflow.is_empty() {
                        let _ = body.style().remove_property("overflow");
                    } else {
                        let _ = body.style().set_property("overflow", &body_overflow);
                    }
                }
                if let Some(element) = previous_focus
                    && let Ok(element) = element.dyn_into::<web_sys::HtmlElement>()
                {
                    let _ = element.focus();
                }
            }
        });
    });
    view! {
        <div class="search-dialog-root">
            <div class="search-dialog-backdrop" data-testid="search-dialog-backdrop" on:click=move |_| overlay_close() />
            <div
                class="search-dialog-panel"
                role="dialog"
                aria-modal="true"
                aria-label=dialog_title
                tabindex="-1"
                node_ref=panel_ref
                data-testid=panel_testid
                on:keydown=move |event| {
                    if event.key() == "Escape" {
                        event.prevent_default();
                        key_close();
                        return;
                    }
                    if event.key() == "Tab" {
                        trap_focus(&event);
                    }
                }
            >
                <div class="search-dialog-head">
                    <h2 class="search-dialog-title">{title.clone()}</h2>
                </div>
                <div class="search-dialog-body">{children()}</div>
                <div class="search-dialog-footer">
                    <Button
                        variant=ButtonVariant::Ghost
                        testid="search-dialog-clear"
                        on_click=move |_| on_clear()
                    >
                        {SHEET_CLEAR}
                    </Button>
                    <span class="search-dialog-footer-actions">
                        <Button
                            variant=ButtonVariant::Ghost
                            testid="search-dialog-close"
                            on_click=move |_| footer_close()
                        >
                            {DIALOG_CLOSE}
                        </Button>
                        <Button
                            variant=ButtonVariant::Primary(ButtonSize::Md)
                            testid="search-dialog-search"
                            on_click=move |_| on_search()
                        >
                            {SEARCH}
                        </Button>
                    </span>
                </div>
            </div>
        </div>
    }
}
