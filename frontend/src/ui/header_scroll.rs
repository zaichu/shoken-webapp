use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;

#[cfg(test)]
mod tests;

/// 向きを切り替えるのに要る最小の移動量(px)。指の微小な揺れで出入りさせない
pub(crate) const MOVE_THRESHOLD: f64 = 8.0;
/// この位置より上ではヘッダーを隠さない(px)。ヘッダーの高さ以上の余裕
pub(crate) const HIDE_AFTER: f64 = 64.0;

/// スクロール領域の測定値
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ScrollMetrics {
    pub top: f64,
    pub max: f64,
    pub focus_within: bool,
}

/// スマホのヘッダーを隠すか。下へ動けば隠し、上へ動けば戻す。
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct HeaderScroll {
    pub hidden: bool,
    anchor: f64,
}

impl HeaderScroll {
    pub(crate) fn update(self, metrics: ScrollMetrics) -> Self {
        // 上端・末端のバウンドで範囲外になる値を範囲内へ戻してから比べる
        let top = metrics.top.clamp(0.0, metrics.max.max(0.0));
        if metrics.focus_within || metrics.max <= 0.0 || top <= HIDE_AFTER {
            return Self {
                hidden: false,
                anchor: top,
            };
        }
        let delta = top - self.anchor;
        if delta.abs() < MOVE_THRESHOLD {
            return self;
        }
        Self {
            hidden: delta > 0.0,
            anchor: top,
        }
    }
}

fn measure(scroller: &web_sys::Element, header: &web_sys::Element) -> ScrollMetrics {
    ScrollMetrics {
        top: f64::from(scroller.scroll_top()),
        max: f64::from(scroller.scroll_height() - scroller.client_height()),
        focus_within: header.matches(":focus-within").unwrap_or(false),
    }
}

/// `.app-scroll` のスクロール方向に合わせてヘッダーを隠す。ヘッダー内にフォーカスがある間は隠さない
pub(crate) fn use_header_hidden(header: NodeRef<leptos::html::Header>) -> RwSignal<bool> {
    let hidden = RwSignal::new(false);
    Effect::new(move |_| {
        let Some(header) = header.get() else {
            return;
        };
        let Some(scroller) = web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.query_selector(".app-scroll").ok().flatten())
        else {
            return;
        };
        let header: web_sys::Element = header.unchecked_into();
        let state = std::rc::Rc::new(std::cell::Cell::new(HeaderScroll {
            hidden: false,
            anchor: f64::from(scroller.scroll_top()),
        }));
        let apply = {
            let state = state.clone();
            let scroller = scroller.clone();
            let header = header.clone();
            move || {
                let next = state.get().update(measure(&scroller, &header));
                state.set(next);
                if hidden.get_untracked() != next.hidden {
                    hidden.set(next.hidden);
                }
            }
        };
        let on_scroll: Closure<dyn FnMut()> = Closure::wrap(Box::new(apply.clone()));
        let on_focus: Closure<dyn FnMut()> = Closure::wrap(Box::new(apply));
        let _ =
            scroller.add_event_listener_with_callback("scroll", on_scroll.as_ref().unchecked_ref());
        let _ =
            header.add_event_listener_with_callback("focusin", on_focus.as_ref().unchecked_ref());
        let handles = send_wrapper::SendWrapper::new((scroller, header, on_scroll, on_focus));
        on_cleanup(move || {
            let (scroller, header, on_scroll, on_focus) = &*handles;
            let _ = scroller
                .remove_event_listener_with_callback("scroll", on_scroll.as_ref().unchecked_ref());
            let _ = header
                .remove_event_listener_with_callback("focusin", on_focus.as_ref().unchecked_ref());
        });
    });
    hidden
}
