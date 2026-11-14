use leptos::prelude::*;
use send_wrapper::SendWrapper;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;

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
/// 指に追従し始める移動量。小さめにして指への吸着感を出す
pub const SWIPE_LOCK_PX: f64 = 10.0;
/// 離したときに切り替わる移動量の割合(面の幅に対する比)
pub const SWIPE_COMMIT_FRACTION: f64 = 0.5;
/// 速い払いとみなす速度(px/ms)。最後の数フレームから求める
pub const SWIPE_FLING_PX_PER_MS: f64 = 0.5;
/// 端のタブでそれ以上進めないときの抵抗の掛け方
pub const SWIPE_RUBBER_FACTOR: f64 = 0.35;
pub const SWIPE_RUBBER_MAX_PX: f64 = 96.0;
/// 離したあと隣へ吸い付く・元へ戻る動きの長さ(ms)
pub const SWIPE_SETTLE_MS: f64 = 250.0;
/// 動きの終わりの合図が来なくても受付を再開するまでの余裕(ms)
pub const SWIPE_SETTLE_GUARD_EXTRA_MS: f64 = 150.0;
/// 動きの終わりの合図が来なくても再開するまでの時間(ms)。動きの長さより少し長い
pub const SWIPE_SETTLE_GUARD_MS: f64 = SWIPE_SETTLE_MS + SWIPE_SETTLE_GUARD_EXTRA_MS;
/// 速度の計算に使う直近の時間幅(ms)
pub const SWIPE_VELOCITY_WINDOW_MS: f64 = 100.0;
const MOBILE_MAX_WIDTH_PX: f64 = 1024.0;

/// 動いている間の入力無視が必要か。動きを減らす設定では動きが無いため今のまま受け付ける
pub const fn swipe_should_ignore_input(settling: bool, reduced_motion: bool) -> bool {
    settling && !reduced_motion
}

/// 新しい接触の開始を無視するか。動いている間のタップ・新しいスワイプと2本目の指を無視する
pub const fn swipe_should_ignore_touchstart(
    settling: bool,
    reduced_motion: bool,
    touch_count: usize,
) -> bool {
    if reduced_motion {
        return false;
    }
    settling || touch_count > 1
}

/// スワイプの向き。左へ払うと次のタブ、右へ払うと前のタブに進む
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SwipeDirection {
    Prev,
    Next,
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

/// 最後の数フレームの移動から速度(px/ms)を求める。符号は移動方向を表す
pub fn swipe_velocity(samples: &[(f64, f64)]) -> f64 {
    if samples.len() < 2 {
        return 0.0;
    }
    let (last_time, last_x) = samples[samples.len() - 1];
    let cutoff = last_time - SWIPE_VELOCITY_WINDOW_MS;
    let (first_time, first_x) = samples
        .iter()
        .copied()
        .find(|(time, _)| *time >= cutoff)
        .unwrap_or((last_time, last_x));
    let elapsed = last_time - first_time;
    if elapsed <= 0.0 {
        0.0
    } else {
        (last_x - first_x) / elapsed
    }
}

/// 進む先に隣のタブがあるか。端のタブでは外向きに進めない
pub fn swipe_can_advance(count: usize, current: usize, dx: f64) -> bool {
    if count == 0 {
        return false;
    }
    (dx < 0.0 && current + 1 < count) || (dx > 0.0 && current > 0)
}

/// 追従を始めてよい接触か。切り替わらない接触(開閉ボタン起点・端末端・ダイアログ中)では面も動かさない
pub fn swipe_lock_allowed(
    suppressed: bool,
    dialog_open: bool,
    start_x: f64,
    viewport: Option<f64>,
) -> bool {
    if suppressed || dialog_open {
        return false;
    }
    match viewport {
        Some(width) => !is_edge_start(start_x, width),
        None => false,
    }
}

/// 指の移動量を面の動きに変える。端では抵抗をつけて少しだけ伸びる
pub fn clamp_drag_offset(dx: f64, count: usize, current: usize, width: f64) -> f64 {
    if dx == 0.0 || count == 0 || width <= 0.0 {
        return 0.0;
    }
    if swipe_can_advance(count, current, dx) {
        dx.clamp(-width, width)
    } else {
        (dx * SWIPE_RUBBER_FACTOR).clamp(-SWIPE_RUBBER_MAX_PX, SWIPE_RUBBER_MAX_PX)
    }
}

/// 離したときの行き先。面の幅の約半分以上か速い払いなら隣へ、足りなければ `None`(元へ戻る)
pub fn swipe_commit_index(
    count: usize,
    current: usize,
    dx: f64,
    dy: f64,
    width: f64,
    velocity: f64,
) -> Option<usize> {
    if count == 0 || width <= 0.0 || dx == 0.0 {
        return None;
    }
    if dx.abs() < SWIPE_AXIS_RATIO * dy.abs() {
        return None;
    }
    let flung = velocity.abs() >= SWIPE_FLING_PX_PER_MS && velocity.signum() == dx.signum();
    let far_enough = dx.abs() >= width * SWIPE_COMMIT_FRACTION;
    if !flung && !far_enough {
        return None;
    }
    if dx < 0.0 {
        (current + 1 < count).then_some(current + 1)
    } else {
        current.checked_sub(1)
    }
}

/// 下線の滑り量(タブ何個分)。指の進み具合そのまま
pub fn tab_ink_offset(current: usize, progress: f64) -> f64 {
    current as f64 + progress.clamp(-1.0, 1.0)
}

/// タブ番号方向の進み具合。左スワイプ(指は負)を正(次へ)で表す
pub fn swipe_progress(drag: f64, width: f64) -> f64 {
    if width <= 0.0 {
        0.0
    } else {
        (-drag / width).clamp(-1.0, 1.0)
    }
}

/// 確定後に面が止まる位置。離した側の外へ流すため指の移動とは逆符号になる
pub fn swipe_settle_offset(current: usize, target: usize, width: f64) -> f64 {
    (current as f64 - target as f64) * width
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

pub fn prefers_reduced_motion() -> bool {
    web_sys::window()
        .and_then(|window| {
            window
                .match_media("(prefers-reduced-motion: reduce)")
                .ok()
                .flatten()
        })
        .is_some_and(|query| query.matches())
}

/// スワイプ中の共有状態。面の追従と下線の滑りを同じ値で動かす
#[derive(Clone, Copy)]
pub struct SwipeMotion {
    pub progress: RwSignal<f64>,
    pub dragging: RwSignal<bool>,
    pub settling: RwSignal<bool>,
}

impl SwipeMotion {
    pub fn new() -> Self {
        Self {
            progress: RwSignal::new(0.0),
            dragging: RwSignal::new(false),
            settling: RwSignal::new(false),
        }
    }
}

impl Default for SwipeMotion {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Default)]
struct SwipeSettle {
    from: f64,
    to: f64,
    target: usize,
    started: f64,
}

#[derive(Default)]
struct SwipeGesture {
    start_x: f64,
    start_y: f64,
    suppressed: bool,
    locked: bool,
    vertical: bool,
    reduced: bool,
    width: f64,
    latest_dx: f64,
    latest_dy: f64,
    samples: Vec<(f64, f64)>,
    frame: Option<i32>,
    settle: Option<SwipeSettle>,
}

fn slide_elements(track: &web_sys::HtmlDivElement) -> Vec<web_sys::HtmlElement> {
    let Ok(list) = track.query_selector_all(".tab-swipe-slide") else {
        return Vec::new();
    };
    (0..list.length())
        .filter_map(|index| list.item(index))
        .filter_map(|element| element.dyn_into::<web_sys::HtmlElement>().ok())
        .collect()
}

fn slide_position(element: &web_sys::HtmlElement) -> f64 {
    element
        .get_attribute("data-index")
        .and_then(|value| value.parse::<f64>().ok())
        .unwrap_or(0.0)
}

fn place_slides(track: &web_sys::HtmlDivElement, current: usize, drag: f64, width: f64) {
    for element in slide_elements(track) {
        let base = (slide_position(&element) - current as f64) * width;
        let _ = element
            .style()
            .set_property("transform", &format!("translateX({}px)", base + drag));
    }
}

fn clear_slides(track: &web_sys::HtmlDivElement) {
    for element in slide_elements(track) {
        let _ = element.style().remove_property("transform");
    }
}

fn request_frame(
    gesture: &Rc<RefCell<SwipeGesture>>,
    mounted: &Arc<AtomicBool>,
    track: NodeRef<leptos::html::Div>,
    count: Signal<usize>,
    current: Signal<usize>,
    on_select: Callback<usize>,
    motion: SwipeMotion,
) {
    if gesture.borrow().frame.is_some() {
        return;
    }
    let Some(window) = web_sys::window() else {
        return;
    };
    let tick_gesture = gesture.clone();
    let tick_mounted = mounted.clone();
    let tick = Closure::once(move || {
        tick_gesture.borrow_mut().frame = None;
        if !tick_mounted.load(Ordering::Relaxed) {
            return;
        }
        let action = {
            let mut gesture = tick_gesture.borrow_mut();
            if let Some(settle) = gesture.settle.take() {
                let rate =
                    ((js_sys::Date::now() - settle.started) / SWIPE_SETTLE_MS).clamp(0.0, 1.0);
                let eased = 1.0 - (1.0 - rate).powi(3);
                let drag = settle.from + (settle.to - settle.from) * eased;
                if rate < 1.0 {
                    gesture.settle = Some(settle);
                    TickAction::Apply { drag, again: true }
                } else {
                    TickAction::Finish {
                        target: settle.target,
                    }
                }
            } else if gesture.locked && !gesture.vertical {
                TickAction::Apply {
                    drag: clamp_drag_offset(
                        gesture.latest_dx,
                        count.get_untracked(),
                        current.get_untracked(),
                        gesture.width,
                    ),
                    again: false,
                }
            } else {
                TickAction::None
            }
        };
        match action {
            TickAction::None => {}
            TickAction::Apply { drag, again } => {
                if let Some(track) = track.get() {
                    let width = tick_gesture.borrow().width;
                    place_slides(&track, current.get_untracked(), drag, width);
                    motion.progress.set(swipe_progress(drag, width));
                }
                if again {
                    request_frame(
                        &tick_gesture,
                        &tick_mounted,
                        track,
                        count,
                        current,
                        on_select,
                        motion,
                    );
                }
            }
            TickAction::Finish { target } => {
                if let Some(track) = track.get() {
                    clear_slides(&track);
                }
                motion.progress.set(0.0);
                motion.dragging.set(false);
                motion.settling.set(false);
                tick_gesture.borrow_mut().locked = false;
                if target != current.get_untracked() {
                    on_select.run(target);
                }
            }
        }
    });
    let id = window
        .request_animation_frame(tick.as_ref().unchecked_ref())
        .ok();
    gesture.borrow_mut().frame = id;
    tick.forget();
}

fn cancel_frame(gesture: &Rc<RefCell<SwipeGesture>>) {
    if let Some(id) = gesture.borrow_mut().frame.take()
        && let Some(window) = web_sys::window()
    {
        let _ = window.cancel_animation_frame(id);
    }
    gesture.borrow_mut().settle = None;
}

fn arm_settle_guard(
    gesture: &Rc<RefCell<SwipeGesture>>,
    mounted: &Arc<AtomicBool>,
    track: NodeRef<leptos::html::Div>,
    current: Signal<usize>,
    on_select: Callback<usize>,
    motion: SwipeMotion,
) {
    let guard_gesture = gesture.clone();
    let guard_mounted = mounted.clone();
    gloo_timers::callback::Timeout::new(SWIPE_SETTLE_GUARD_MS as u32, move || {
        if !guard_mounted.load(Ordering::Relaxed) {
            return;
        }
        if !motion.settling.get_untracked() {
            return;
        }
        let target = {
            let mut gesture = guard_gesture.borrow_mut();
            let Some(settle) = gesture.settle.take() else {
                return;
            };
            if let Some(id) = gesture.frame.take()
                && let Some(window) = web_sys::window()
            {
                let _ = window.cancel_animation_frame(id);
            }
            gesture.locked = false;
            settle.target
        };
        if let Some(track) = track.get() {
            clear_slides(&track);
        }
        motion.progress.set(0.0);
        motion.dragging.set(false);
        motion.settling.set(false);
        if target != current.get_untracked() {
            on_select.run(target);
        }
    })
    .forget();
}

enum TickAction {
    None,
    Apply { drag: f64, again: bool },
    Finish { target: usize },
}

#[expect(clippy::too_many_arguments)]
fn finish_swipe(
    gesture: &Rc<RefCell<SwipeGesture>>,
    mounted: &Arc<AtomicBool>,
    track: NodeRef<leptos::html::Div>,
    count: Signal<usize>,
    current: Signal<usize>,
    on_select: Callback<usize>,
    motion: SwipeMotion,
    event: &web_sys::TouchEvent,
    cancelled: bool,
) {
    cancel_frame(gesture);
    let (locked, start_x, suppressed, width, latest, samples) = {
        let mut gesture = gesture.borrow_mut();
        let state = (
            gesture.locked,
            gesture.start_x,
            gesture.suppressed,
            gesture.width,
            (gesture.latest_dx, gesture.latest_dy),
            std::mem::take(&mut gesture.samples),
        );
        gesture.locked = false;
        gesture.vertical = false;
        state
    };
    let reset_view = || {
        if let Some(track) = track.get() {
            clear_slides(&track);
        }
        motion.progress.set(0.0);
        motion.dragging.set(false);
        motion.settling.set(false);
    };
    if cancelled || suppressed || has_open_dialog() || !is_mobile_width() {
        reset_view();
        return;
    }
    let count_value = count.get_untracked();
    let current_value = current.get_untracked();
    let Some(viewport) = viewport_width() else {
        reset_view();
        return;
    };
    if is_edge_start(start_x, viewport) {
        reset_view();
        return;
    }
    let (mut dx, mut dy) = latest;
    if !cancelled && let Some(touch) = event.changed_touches().get(0) {
        dx = f64::from(touch.client_x()) - start_x;
        dy = f64::from(touch.client_y()) - gesture.borrow().start_y;
    }
    if gesture.borrow().reduced {
        reset_view();
        if !cancelled && let Some(next) = next_swipe_index(count_value, current_value, dx, dy) {
            on_select.run(next);
        }
        return;
    }
    if !locked && cancelled {
        reset_view();
        return;
    }
    let mut width = width;
    if !locked {
        width = track
            .get()
            .map(|track| f64::from(track.client_width()))
            .unwrap_or(0.0);
    }
    let from = if locked {
        clamp_drag_offset(dx, count_value, current_value, width)
    } else {
        0.0
    };
    let velocity = swipe_velocity(&samples);
    let target = swipe_commit_index(count_value, current_value, dx, dy, width, velocity)
        .unwrap_or(current_value);
    let to = swipe_settle_offset(current_value, target, width);
    if from == 0.0 && to == 0.0 {
        reset_view();
        return;
    }
    gesture.borrow_mut().settle = Some(SwipeSettle {
        from,
        to,
        target,
        started: js_sys::Date::now(),
    });
    motion.settling.set(true);
    arm_settle_guard(gesture, mounted, track, current, on_select, motion);
    request_frame(gesture, mounted, track, count, current, on_select, motion);
}

/// タブ1枚分の器。表示中のものだけ描き、指で引いている間だけ隣を足す
#[component]
pub fn SwipeSlide(
    index: usize,
    #[prop(into)] current: Signal<usize>,
    motion: SwipeMotion,
    children: ChildrenFn,
) -> impl IntoView {
    let mounted = Memo::new(move |_| {
        let value = current.get();
        index == value || (motion.dragging.get() && value.abs_diff(index) == 1)
    });
    view! {
        {move || {
            mounted.get().then(|| view! {
                <div
                    class="tab-swipe-slide"
                    data-index=index.to_string()
                    data-active=move || (current.get() == index).to_string()
                >
                    {children()}
                </div>
            })
        }}
    }
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
    motion: SwipeMotion,
    children: Children,
) -> impl IntoView {
    let track = NodeRef::<leptos::html::Div>::new();
    let area = NodeRef::<leptos::html::Div>::new();
    let gesture: Rc<RefCell<SwipeGesture>> = Rc::new(RefCell::new(SwipeGesture::default()));
    let mounted = Arc::new(AtomicBool::new(true));
    {
        let mounted = mounted.clone();
        on_cleanup(move || {
            mounted.store(false, Ordering::Relaxed);
        });
    }
    {
        Effect::new(move || {
            let Some(element) = area.get() else {
                return;
            };
            let guard =
                SendWrapper::new(Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
                    if event.detail() == 0 {
                        return;
                    }
                    if motion.settling.get_untracked() {
                        event.prevent_default();
                        event.stop_propagation();
                    }
                }) as Box<dyn FnMut(_)>));
            let _ = element.add_event_listener_with_callback_and_bool(
                "click",
                guard.as_ref().unchecked_ref(),
                true,
            );
            let pair = SendWrapper::new((element, guard));
            on_cleanup(move || {
                let (target, guard) = &*pair;
                let _ = target.remove_event_listener_with_callback_and_bool(
                    "click",
                    guard.as_ref().unchecked_ref(),
                    true,
                );
            });
        });
    }
    let on_touchstart = {
        let gesture = gesture.clone();
        move |event: web_sys::TouchEvent| {
            if swipe_should_ignore_touchstart(
                motion.settling.get_untracked(),
                prefers_reduced_motion(),
                event.touches().length() as usize,
            ) {
                return;
            }
            cancel_frame(&gesture);
            if let Some(track) = track.get() {
                clear_slides(&track);
            }
            motion.progress.set(0.0);
            motion.dragging.set(false);
            motion.settling.set(false);
            {
                let mut gesture = gesture.borrow_mut();
                *gesture = SwipeGesture::default();
                gesture.reduced = prefers_reduced_motion();
            }
            let Some(touch) = event.touches().get(0) else {
                return;
            };
            let suppressed = event
                .target()
                .is_some_and(|target| swipe_start_suppressed(&target));
            let mut gesture = gesture.borrow_mut();
            gesture.start_x = f64::from(touch.client_x());
            gesture.start_y = f64::from(touch.client_y());
            gesture.suppressed = suppressed;
        }
    };
    let on_touchmove = {
        let gesture = gesture.clone();
        let mounted = mounted.clone();
        move |event: web_sys::TouchEvent| {
            if !is_mobile_width() {
                return;
            }
            if gesture.borrow().settle.is_some() {
                return;
            }
            if event.touches().length() > 1 {
                return;
            }
            let Some(touch) = event.touches().get(0) else {
                return;
            };
            let x = f64::from(touch.client_x());
            let y = f64::from(touch.client_y());
            let mut locked = gesture.borrow_mut();
            if locked.reduced || locked.vertical {
                return;
            }
            let dx = x - locked.start_x;
            let dy = y - locked.start_y;
            if !locked.locked {
                if dx.abs() < SWIPE_AXIS_RATIO * dy.abs() {
                    if dy.abs() >= SWIPE_LOCK_PX {
                        locked.vertical = true;
                    }
                    return;
                }
                if dx.abs() < SWIPE_LOCK_PX {
                    return;
                }
                if !swipe_lock_allowed(
                    locked.suppressed,
                    has_open_dialog(),
                    locked.start_x,
                    viewport_width(),
                ) {
                    locked.vertical = true;
                    return;
                }
                locked.locked = true;
                locked.width = track
                    .get()
                    .map(|track| f64::from(track.client_width()))
                    .unwrap_or(0.0);
                if locked.width <= 0.0 {
                    locked.locked = false;
                    locked.vertical = true;
                    return;
                }
                motion.dragging.set(true);
                motion.settling.set(true);
            }
            locked.latest_dx = dx;
            locked.latest_dy = dy;
            locked.samples.push((event.time_stamp(), x));
            if locked.samples.len() > 8 {
                locked.samples.remove(0);
            }
            drop(locked);
            request_frame(&gesture, &mounted, track, count, current, on_select, motion);
        }
    };
    let on_touchend = {
        let gesture = gesture.clone();
        let mounted = mounted.clone();
        move |event: web_sys::TouchEvent| {
            if gesture.borrow().settle.is_some() {
                return;
            }
            finish_swipe(
                &gesture, &mounted, track, count, current, on_select, motion, &event, false,
            );
        }
    };
    let on_touchcancel = {
        let gesture = gesture.clone();
        let mounted = mounted.clone();
        move |event: web_sys::TouchEvent| {
            if gesture.borrow().settle.is_some() {
                return;
            }
            finish_swipe(
                &gesture, &mounted, track, count, current, on_select, motion, &event, true,
            );
        }
    };
    view! {
        <div
            class="content-gutter tab-swipe-area flex-1 min-h-0 overflow-y-auto lg:flex-none lg:overflow-visible"
            data-testid=testid
            data-settling=move || motion.settling.get().to_string()
            node_ref=area
            on:touchstart=on_touchstart
            on:touchmove=on_touchmove
            on:touchend=on_touchend
            on:touchcancel=on_touchcancel
        >
            <div class="tab-swipe-track" node_ref=track>
                {children()}
            </div>
        </div>
    }
}
