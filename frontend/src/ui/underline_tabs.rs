use leptos::prelude::*;
use wasm_bindgen::JsCast;

#[cfg(test)]
mod tests;

pub fn next_tab_index(count: usize, current: usize, key: &str) -> Option<usize> {
    if count == 0 {
        return None;
    }
    match key {
        "ArrowRight" => Some((current + 1) % count),
        "ArrowLeft" => Some((current + count - 1) % count),
        "Home" => Some(0),
        "End" => Some(count - 1),
        _ => None,
    }
}

pub fn focus_tab_by_id(id: &str) {
    let Some(element) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id(id))
    else {
        return;
    };
    if let Some(element) = element.dyn_ref::<web_sys::HtmlElement>() {
        let _ = element.focus();
    }
}

/// 横スワイプでタブを切り替える判定のしきい値。縦スクロールとの区別と端末差を吸収するため少し余裕を持つ
pub const SWIPE_MIN_DISTANCE_PX: f64 = 60.0;
pub const SWIPE_AXIS_RATIO: f64 = 2.0;
pub const SWIPE_EDGE_GUARD_PX: f64 = 20.0;
const MOBILE_MAX_WIDTH_PX: f64 = 1024.0;

/// スワイプの向き。左へ払うと次のタブ、右へ払うと前のタブに進む
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SwipeDirection {
    Prev,
    Next,
}

impl SwipeDirection {
    fn attr(self) -> &'static str {
        match self {
            Self::Prev => "prev",
            Self::Next => "next",
        }
    }
}

/// 横の移動が縦の2倍以上かつ約60px以上あるときだけ向きを返す。縦スクロールは無視する
pub fn swipe_step(dx: f64, dy: f64) -> Option<SwipeDirection> {
    if dx.abs() < SWIPE_MIN_DISTANCE_PX {
        return None;
    }
    if dx.abs() < SWIPE_AXIS_RATIO * dy.abs() {
        return None;
    }
    if dx < 0.0 {
        Some(SwipeDirection::Next)
    } else {
        Some(SwipeDirection::Prev)
    }
}

/// スワイプ先のタブ位置。端のタブではそれ以上動かず `None` を返す(キー操作の循環とは別扱い)
pub fn next_swipe_index(count: usize, current: usize, dx: f64, dy: f64) -> Option<usize> {
    let step = swipe_step(dx, dy)?;
    match step {
        SwipeDirection::Next if current + 1 < count => Some(current + 1),
        SwipeDirection::Prev if current > 0 => Some(current - 1),
        _ => None,
    }
}

/// 画面の左右の端から始めた接触か。ブラウザの戻る操作とぶつかるため無視する
pub fn is_edge_start(client_x: f64, viewport_width: f64) -> bool {
    client_x < SWIPE_EDGE_GUARD_PX || client_x > viewport_width - SWIPE_EDGE_GUARD_PX
}

/// ダイアログの内側や開閉ボタンの上から始めた接触か。操作中のタブ切り替えを防ぐ
pub fn swipe_start_suppressed(target: &web_sys::EventTarget) -> bool {
    let Some(element) = target.dyn_ref::<web_sys::Element>() else {
        return false;
    };
    element
        .closest("[role=\"dialog\"], button[aria-expanded]")
        .ok()
        .flatten()
        .is_some()
}

fn viewport_width() -> Option<f64> {
    web_sys::window()
        .and_then(|window| window.inner_width().ok())
        .and_then(|width| width.as_f64())
}

/// ダイアログが開いている間は切り替えない
pub fn has_open_dialog() -> bool {
    web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.query_selector("[role=\"dialog\"]").ok().flatten())
        .is_some()
}

fn is_mobile_width() -> bool {
    viewport_width().is_some_and(|width| width < MOBILE_MAX_WIDTH_PX)
}

/// 下線タブの中身を包む器。スマホ幅で横スワイプだけ拾い、縦スクロール・端からの接触・
/// ダイアログや開閉ボタンの操作中は切り替えない。PC幅では何もしない
#[component]
pub fn SwipeArea(
    /// 中身のスクロール面の data-testid(資産管理・取引明細で既存の値をそのまま渡す)
    #[prop(into)]
    testid: String,
    #[prop(into)] count: Signal<usize>,
    #[prop(into)] current: Signal<usize>,
    on_select: Callback<usize>,
    children: Children,
) -> impl IntoView {
    let start = RwSignal::new(None::<(f64, f64, bool)>);
    let direction = RwSignal::new(None::<SwipeDirection>);
    let dir_attr = move || direction.get().map(SwipeDirection::attr).unwrap_or("");
    let on_touchstart = move |event: web_sys::TouchEvent| {
        let Some(touch) = event.touches().get(0) else {
            start.set(None);
            return;
        };
        let suppressed = event
            .target()
            .is_some_and(|target| swipe_start_suppressed(&target));
        start.set(Some((
            f64::from(touch.client_x()),
            f64::from(touch.client_y()),
            suppressed,
        )));
    };
    let finish = move |event: web_sys::TouchEvent, cancelled: bool| {
        let Some((start_x, start_y, suppressed)) = start.get() else {
            return;
        };
        start.set(None);
        if cancelled || suppressed || has_open_dialog() || !is_mobile_width() {
            return;
        }
        let Some(touch) = event.changed_touches().get(0) else {
            return;
        };
        let Some(width) = viewport_width() else {
            return;
        };
        if is_edge_start(start_x, width) {
            return;
        }
        let dx = f64::from(touch.client_x()) - start_x;
        let dy = f64::from(touch.client_y()) - start_y;
        if let Some(next) = next_swipe_index(count.get_untracked(), current.get_untracked(), dx, dy)
        {
            direction.set(swipe_step(dx, dy));
            on_select.run(next);
            leptos::task::spawn_local(async move {
                gloo_timers::future::TimeoutFuture::new(250).await;
                direction.set(None);
            });
        }
    };
    let on_touchend = move |event: web_sys::TouchEvent| finish(event, false);
    let on_touchcancel = move |event: web_sys::TouchEvent| finish(event, true);
    view! {
        <div
            class="content-gutter tab-swipe-area flex-1 min-h-0 overflow-y-auto lg:flex-none lg:overflow-visible"
            data-testid=testid
            data-swipe-dir=dir_attr
            on:touchstart=on_touchstart
            on:touchend=on_touchend
            on:touchcancel=on_touchcancel
        >
            <div class="tab-swipe-panel" data-swipe-dir=dir_attr>
                {children()}
            </div>
        </div>
    }
}
