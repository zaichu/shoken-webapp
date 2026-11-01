use crate::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::ui::focus::trap_focus;
use leptos::prelude::*;
use wasm_bindgen::JsCast;

fn swipe_should_close(start_y: f64, end_y: f64) -> bool {
    end_y - start_y > 60.0
}

/// 下から出る共通シート。行の詳細シートでも使い回す前提で中身は呼び出し側が渡す。
#[component]
pub fn BottomSheet(
    #[prop(into)] title: String,
    on_close: impl Fn() + Clone + 'static,
    #[prop(optional)] testid: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    let panel_ref = NodeRef::<leptos::html::Div>::new();
    let touch_start = RwSignal::new(None::<f64>);
    let overlay_close = on_close.clone();
    let key_close = on_close.clone();
    let button_close = on_close.clone();
    let swipe_close = on_close.clone();
    let dialog_title = title.clone();
    let panel_testid = testid.unwrap_or("bottom-sheet");
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
        <div class="bottom-sheet-root">
            <div class="bottom-sheet-backdrop" on:click=move |_| overlay_close() />
            <div
                class="bottom-sheet-panel"
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
                <div
                    class="bottom-sheet-handle"
                    aria-hidden="true"
                    on:touchstart=move |event: web_sys::TouchEvent| {
                        if let Some(touch) = event.touches().get(0) {
                            touch_start.set(Some(f64::from(touch.client_y())));
                        }
                    }
                    on:touchend=move |event: web_sys::TouchEvent| {
                        let start = touch_start.get_untracked();
                        touch_start.set(None);
                        if let Some(start) = start
                            && let Some(touch) = event.changed_touches().get(0)
                            && swipe_should_close(start, f64::from(touch.client_y()))
                        {
                            swipe_close();
                        }
                    }
                >
                    <span class="bottom-sheet-grip" />
                </div>
                <div class="bottom-sheet-head">
                    <h2 class="bottom-sheet-title">{title.clone()}</h2>
                    <Button
                        variant=ButtonVariant::Secondary(ButtonSize::Xs)
                        class="bottom-sheet-close".to_string()
                        aria_label="閉じる".to_string()
                        testid="bottom-sheet-close"
                        on_click=move |_| button_close()
                    >
                        <span aria-hidden="true">"✕"</span>
                    </Button>
                </div>
                <div class="bottom-sheet-body">{children()}</div>
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::swipe_should_close;

    #[test]
    fn swipe_down_over_threshold_closes() {
        assert!(swipe_should_close(100.0, 170.0));
        assert!(!swipe_should_close(100.0, 160.0));
    }

    #[test]
    fn small_or_upward_move_keeps_sheet_open() {
        assert!(!swipe_should_close(100.0, 130.0));
        assert!(!swipe_should_close(170.0, 100.0));
        assert!(!swipe_should_close(100.0, 100.0));
    }
}
