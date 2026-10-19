use crate::session::SessionStore;
use crate::session::cross_tab;
use gloo_timers::callback::Timeout;
use leptos::ev;
use leptos::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

const MS_PER_MINUTE: u32 = 60_000;

type IdleTimer = Rc<RefCell<Option<Timeout>>>;

fn now_ms() -> u64 {
    js_sys::Date::now() as u64
}

fn timeout_ms(minutes: RwSignal<u32>) -> u32 {
    minutes.get_untracked().saturating_mul(MS_PER_MINUTE)
}

// 時計ずれで共有時刻が未来でも u32 変換で壊れないよう、タイムアウトで頭打ちにする
fn idle_remaining_ms(last_activity_ms: u64, now_ms: u64, timeout_ms: u64) -> i64 {
    last_activity_ms
        .saturating_add(timeout_ms)
        .saturating_sub(now_ms)
        .min(timeout_ms) as i64
}

fn arm_timer(timer: &IdleTimer, session: SessionStore, minutes: RwSignal<u32>, delay_ms: u32) {
    if session.user.get_untracked().is_none() {
        timer.borrow_mut().take();
        return;
    }
    let timer2 = Rc::clone(timer);
    timer
        .borrow_mut()
        .replace(Timeout::new(delay_ms.max(1), move || {
            on_idle_timer(&timer2, session, minutes);
        }));
}

// 満了までの残り時間で張り直す。設定値の変更途中でも、共有された最終操作時刻から残りを引く
fn arm_idle_timer(timer: &IdleTimer, session: SessionStore, minutes: RwSignal<u32>) {
    let last = cross_tab::last_activity_ms().unwrap_or_else(now_ms);
    let delay = idle_remaining_ms(last, now_ms(), timeout_ms(minutes) as u64);
    arm_timer(timer, session, minutes, delay.max(0) as u32);
}

fn on_idle_timer(timer: &IdleTimer, session: SessionStore, minutes: RwSignal<u32>) {
    if session.user.get_untracked().is_none() {
        return;
    }
    // 書き込みの間引きや取りこぼしで満了が前倒しになり得るので、共有された最終操作時刻で判定し直す
    let remaining = cross_tab::last_activity_ms()
        .map(|last| idle_remaining_ms(last, now_ms(), timeout_ms(minutes) as u64));
    match remaining {
        Some(remaining) if remaining > 0 => {
            arm_timer(timer, session, minutes, remaining as u32);
        }
        _ => {
            leptos::task::spawn_local(async move {
                if cross_tab::claim_logout_once().await {
                    session.logout().await;
                } else {
                    session.mark_unauthenticated();
                    session.loaded.set(true);
                }
            });
        }
    }
}

fn on_user_activity(timer: &IdleTimer, session: SessionStore, minutes: RwSignal<u32>) {
    // 未認証タブの操作で認証済みタブのログアウトを延ばさない
    if session.user.get_untracked().is_some() {
        cross_tab::record_activity();
    }
    arm_idle_timer(timer, session, minutes);
}

// イベントリスナー自体は常駐させ、未ログイン時は arm 側で無効化する
pub fn watch_idle_logout(session: SessionStore, minutes: RwSignal<u32>) {
    let timer: IdleTimer = Rc::new(RefCell::new(None));

    let timer_effect = Rc::clone(&timer);
    let was_authed = std::cell::Cell::new(false);
    Effect::new(move |_| {
        // 設定が変わったら残り時間でタイマーを張り直す
        minutes.get();
        if session.user.get().is_some() {
            // 期限切れの最終操作時刻が残ったまま再ログインすると即ログアウトするため、
            // 認証の開始を新しい操作として記録し直す。設定変更では上書きしない
            if !was_authed.replace(true) {
                cross_tab::record_activity();
            }
            arm_idle_timer(&timer_effect, session, minutes);
        } else {
            was_authed.set(false);
            timer_effect.borrow_mut().take();
        }
    });

    // 他タブの操作でもこのタブのタイマーを延ばす
    let storage_handle = window_event_listener(ev::storage, {
        let timer = Rc::clone(&timer);
        move |event| {
            if event.key().as_deref() == Some(cross_tab::ACTIVITY_KEY) {
                arm_idle_timer(&timer, session, minutes);
            }
        }
    });

    let handles = [
        window_event_listener(ev::mousedown, {
            let timer = Rc::clone(&timer);
            move |_| on_user_activity(&timer, session, minutes)
        }),
        window_event_listener(ev::mousemove, {
            let timer = Rc::clone(&timer);
            move |_| on_user_activity(&timer, session, minutes)
        }),
        window_event_listener(ev::keydown, {
            let timer = Rc::clone(&timer);
            move |_| on_user_activity(&timer, session, minutes)
        }),
        window_event_listener(ev::scroll, {
            let timer = Rc::clone(&timer);
            move |_| on_user_activity(&timer, session, minutes)
        }),
        window_event_listener(ev::touchstart, {
            let timer = Rc::clone(&timer);
            move |_| on_user_activity(&timer, session, minutes)
        }),
        window_event_listener(ev::click, {
            let timer = Rc::clone(&timer);
            move |_| on_user_activity(&timer, session, minutes)
        }),
    ];
    on_cleanup(move || {
        storage_handle.remove();
        for handle in handles {
            handle.remove();
        }
    });
}

#[cfg(test)]
mod tests;
