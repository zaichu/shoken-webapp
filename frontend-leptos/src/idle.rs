use crate::cross_tab;
use crate::session::SessionStore;
use gloo_timers::callback::Timeout;
use leptos::ev;
use leptos::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

const IDLE_TIMEOUT_MS: u32 = 30 * 60 * 1000;

type IdleTimer = Rc<RefCell<Option<Timeout>>>;

fn now_ms() -> u64 {
    js_sys::Date::now() as u64
}

// 時計ずれで共有時刻が未来でも u32 変換で壊れないよう、タイムアウトで頭打ちにする
fn idle_remaining_ms(last_activity_ms: u64, now_ms: u64) -> i64 {
    last_activity_ms
        .saturating_add(IDLE_TIMEOUT_MS as u64)
        .saturating_sub(now_ms)
        .min(IDLE_TIMEOUT_MS as u64) as i64
}

fn arm_timer(timer: &IdleTimer, session: SessionStore, delay_ms: u32) {
    if session.user.get_untracked().is_none() {
        timer.borrow_mut().take();
        return;
    }
    let timer2 = Rc::clone(timer);
    timer
        .borrow_mut()
        .replace(Timeout::new(delay_ms.max(1), move || {
            on_idle_timer(&timer2, session);
        }));
}

fn on_idle_timer(timer: &IdleTimer, session: SessionStore) {
    if session.user.get_untracked().is_none() {
        return;
    }
    // 書き込みの間引きや取りこぼしで満了が前倒しになり得るので、共有された最終操作時刻で判定し直す
    match cross_tab::last_activity_ms().map(|last| idle_remaining_ms(last, now_ms())) {
        Some(remaining) if remaining > 0 => {
            arm_timer(timer, session, remaining as u32);
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

fn on_user_activity(timer: &IdleTimer, session: SessionStore) {
    // 未認証タブの操作で認証済みタブのログアウトを延ばさない
    if session.user.get_untracked().is_some() {
        cross_tab::record_activity();
    }
    arm_timer(timer, session, IDLE_TIMEOUT_MS);
}

// イベントリスナー自体は常駐させ、未ログイン時は arm 側で無効化する
pub fn watch_idle_logout(session: SessionStore) {
    let timer: IdleTimer = Rc::new(RefCell::new(None));

    let timer_effect = Rc::clone(&timer);
    Effect::new(move |_| {
        if session.user.get().is_some() {
            arm_timer(&timer_effect, session, IDLE_TIMEOUT_MS);
        } else {
            timer_effect.borrow_mut().take();
        }
    });

    // 他タブの操作でもこのタブのタイマーを延ばす
    let storage_handle = window_event_listener(ev::storage, {
        let timer = Rc::clone(&timer);
        move |event| {
            if event.key().as_deref() == Some(cross_tab::ACTIVITY_KEY) {
                arm_timer(&timer, session, IDLE_TIMEOUT_MS);
            }
        }
    });

    let handles = [
        window_event_listener(ev::mousedown, {
            let timer = Rc::clone(&timer);
            move |_| on_user_activity(&timer, session)
        }),
        window_event_listener(ev::mousemove, {
            let timer = Rc::clone(&timer);
            move |_| on_user_activity(&timer, session)
        }),
        window_event_listener(ev::keydown, {
            let timer = Rc::clone(&timer);
            move |_| on_user_activity(&timer, session)
        }),
        window_event_listener(ev::scroll, {
            let timer = Rc::clone(&timer);
            move |_| on_user_activity(&timer, session)
        }),
        window_event_listener(ev::touchstart, {
            let timer = Rc::clone(&timer);
            move |_| on_user_activity(&timer, session)
        }),
        window_event_listener(ev::click, {
            let timer = Rc::clone(&timer);
            move |_| on_user_activity(&timer, session)
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
mod tests {
    use super::*;

    #[test]
    fn idle_remaining_hits_zero_at_deadline() {
        let timeout = IDLE_TIMEOUT_MS as u64;
        assert_eq!(idle_remaining_ms(0, 0), timeout as i64);
        assert_eq!(idle_remaining_ms(0, timeout - 1), 1);
        assert_eq!(idle_remaining_ms(0, timeout), 0);
        assert_eq!(idle_remaining_ms(0, timeout + 1_000), 0);
        assert_eq!(idle_remaining_ms(1_000, 1_000), timeout as i64);
    }

    #[test]
    fn idle_remaining_is_capped_for_future_timestamps() {
        let timeout = IDLE_TIMEOUT_MS as u64;
        assert_eq!(idle_remaining_ms(timeout + 60_000, 0), timeout as i64);
        assert_eq!(idle_remaining_ms(u64::MAX - 1, 0), timeout as i64);
    }
}
