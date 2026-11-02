use crate::ui::focus::trap_focus;
use leptos::prelude::*;
use wasm_bindgen::JsCast;

pub(crate) fn use_page_scroll_lock() {
    // iOS Safari は overflow:hidden だけでは後ろが動くため body を固定して位置を保つ
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
        let body_position = body_style
            .as_ref()
            .and_then(|style| style.get_property_value("position").ok())
            .unwrap_or_default();
        let body_top = body_style
            .as_ref()
            .and_then(|style| style.get_property_value("top").ok())
            .unwrap_or_default();
        let body_width = body_style
            .as_ref()
            .and_then(|style| style.get_property_value("width").ok())
            .unwrap_or_default();
        let page_y = window.scroll_y().unwrap_or_default();
        let app_top = app_scroll.as_ref().map(|scroller| scroller.scroll_top());
        if let Some(style) = body_style.as_ref() {
            let _ = style.set_property("overflow", "hidden");
            let _ = style.set_property("position", "fixed");
            let _ = style.set_property("top", &format!("-{page_y}px"));
            let _ = style.set_property("width", "100%");
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
                    if body_position.is_empty() {
                        let _ = style.remove_property("position");
                    } else {
                        let _ = style.set_property("position", &body_position);
                    }
                    if body_top.is_empty() {
                        let _ = style.remove_property("top");
                    } else {
                        let _ = style.set_property("top", &body_top);
                    }
                    if body_width.is_empty() {
                        let _ = style.remove_property("width");
                    } else {
                        let _ = style.set_property("width", &body_width);
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
    let overlay_close = on_close.clone();
    let key_close = on_close.clone();
    let dialog_title = title.clone();
    let panel_testid = testid.unwrap_or("search-dialog");
    use_page_scroll_lock();
    Effect::new(move |_| {
        let previous_focus = web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.active_element());
        if let Some(panel) = panel_ref.get() {
            let _ = panel.focus();
        }
        on_cleanup(move || {
            if let Some(element) = previous_focus
                && let Ok(element) = element.dyn_into::<web_sys::HtmlElement>()
            {
                let _ = element.focus();
            }
        });
    });
    view! {
        <div class="search-dialog-root">
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
