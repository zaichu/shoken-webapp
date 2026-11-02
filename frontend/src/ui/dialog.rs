use crate::ui::focus::trap_focus;
use leptos::prelude::*;
use wasm_bindgen::JsCast;

pub(crate) fn move_root_to_body(root: NodeRef<leptos::html::Div>) {
    Effect::new(move |_| {
        if let Some(root) = root.get()
            && let Some(body) = web_sys::window()
                .and_then(|window| window.document())
                .and_then(|document| document.body())
        {
            let _ = body.append_child(root.unchecked_ref());
        }
    });
}

pub(crate) fn focus_without_scroll(element: &web_sys::HtmlElement) {
    let options = web_sys::FocusOptions::new();
    options.set_prevent_scroll(true);
    let _ = element.focus_with_options(&options);
}

pub(crate) fn use_page_scroll_lock() {
    Effect::new(move |_| {
        let Some(window) = web_sys::window() else {
            return;
        };
        let Some(document) = window.document() else {
            return;
        };
        let body = document.body();
        let html = document
            .document_element()
            .and_then(|element| element.dyn_into::<web_sys::HtmlElement>().ok());
        let app_scroll = document
            .query_selector(".app-scroll")
            .ok()
            .flatten()
            .and_then(|element| element.dyn_into::<web_sys::HtmlElement>().ok());
        let body_style = body.as_ref().map(|body| body.style());
        let html_style = html.as_ref().map(|html| html.style());
        let app_style = app_scroll.as_ref().map(|scroller| scroller.style());
        let body_overflow = body_style
            .as_ref()
            .and_then(|style| style.get_property_value("overflow").ok())
            .unwrap_or_default();
        let html_overflow = html_style
            .as_ref()
            .and_then(|style| style.get_property_value("overflow").ok())
            .unwrap_or_default();
        let app_overflow = app_style
            .as_ref()
            .and_then(|style| style.get_property_value("overflow").ok())
            .unwrap_or_default();
        let page_y = window.scroll_y().unwrap_or_default();
        let app_top = app_scroll.as_ref().map(|scroller| scroller.scroll_top());
        if let Some(style) = body_style.as_ref() {
            let _ = style.set_property("overflow", "hidden");
        }
        if let Some(style) = html_style.as_ref() {
            let _ = style.set_property("overflow", "hidden");
        }
        if let Some(style) = app_style.as_ref() {
            let _ = style.set_property("overflow", "hidden");
        }
        on_cleanup(move || {
            if let Some(window) = web_sys::window()
                && let Some(document) = window.document()
            {
                if let Some(body) = document.body() {
                    let style = body.style();
                    if body_overflow.is_empty() {
                        let _ = style.remove_property("overflow");
                    } else {
                        let _ = style.set_property("overflow", &body_overflow);
                    }
                }
                if let Some(html) = document
                    .document_element()
                    .and_then(|element| element.dyn_into::<web_sys::HtmlElement>().ok())
                {
                    let style = html.style();
                    if html_overflow.is_empty() {
                        let _ = style.remove_property("overflow");
                    } else {
                        let _ = style.set_property("overflow", &html_overflow);
                    }
                }
                if let Ok(Some(scroller)) = document.query_selector(".app-scroll")
                    && let Ok(scroller) = scroller.dyn_into::<web_sys::HtmlElement>()
                {
                    if app_overflow.is_empty() {
                        let _ = scroller.style().remove_property("overflow");
                    } else {
                        let _ = scroller.style().set_property("overflow", &app_overflow);
                    }
                    if let Some(top) = app_top {
                        scroller.set_scroll_top(top);
                    }
                }
                window.scroll_to_with_x_and_y(0.0, page_y);
            }
        });
    });
}

#[component]
pub fn DialogShell(
    #[prop(into)] title: String,
    on_close: impl Fn() + Clone + Send + 'static,
    #[prop(optional)] testid: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    let panel_ref = NodeRef::<leptos::html::Div>::new();
    let root_ref = NodeRef::<leptos::html::Div>::new();
    let overlay_close = on_close.clone();
    let key_close = on_close.clone();
    let dialog_title = title.clone();
    let panel_testid = testid.unwrap_or("search-dialog");
    use_page_scroll_lock();
    move_root_to_body(root_ref);
    Effect::new(move |_| {
        let previous_focus = web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.active_element());
        if let Some(panel) = panel_ref.get() {
            focus_without_scroll(&panel);
        }
        on_cleanup(move || {
            if let Some(element) = previous_focus
                && let Ok(element) = element.dyn_into::<web_sys::HtmlElement>()
            {
                focus_without_scroll(&element);
            }
        });
    });
    view! {
        <div class="search-dialog-root" node_ref=root_ref>
            <div
                class="search-dialog-backdrop"
                data-testid="search-dialog-backdrop"
                on:click=move |_| overlay_close()
            />
            <div
                class="dialog-panel search-dialog-panel"
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
                {children()}
            </div>
        </div>
    }
}
