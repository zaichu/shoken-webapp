use crate::session::SessionStore;
use gloo_timers::callback::Timeout;
use leptos::ev;
use leptos::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

// React版 AuthContext.tsx の IDLE_TIMEOUT = 30 * 60 * 1000 に相当
const IDLE_TIMEOUT_MS: u32 = 30 * 60 * 1000;

type IdleTimer = Rc<RefCell<Option<Timeout>>>;

fn reset_timer(timer: &IdleTimer, session: SessionStore) {
    if session.user.get_untracked().is_none() {
        timer.borrow_mut().take();
        return;
    }
    timer
        .borrow_mut()
        .replace(Timeout::new(IDLE_TIMEOUT_MS, move || {
            if session.user.get_untracked().is_some() {
                leptos::task::spawn_local(async move {
                    session.logout().await;
                });
            }
        }));
}

// React版 useIdleTimer と同じ操作イベントでタイマーを張り直す。
// イベントリスナー自体は常駐させ、未ログイン時は reset 側で無効化する
pub fn watch_idle_logout(session: SessionStore) {
    let timer: IdleTimer = Rc::new(RefCell::new(None));

    // ログイン/ログアウトでタイマーを開始・停止する
    let timer_effect = Rc::clone(&timer);
    Effect::new(move |_| {
        if session.user.get().is_some() {
            reset_timer(&timer_effect, session);
        } else {
            timer_effect.borrow_mut().take();
        }
    });

    let handles = [
        window_event_listener(ev::mousedown, {
            let timer = Rc::clone(&timer);
            move |_| reset_timer(&timer, session)
        }),
        window_event_listener(ev::mousemove, {
            let timer = Rc::clone(&timer);
            move |_| reset_timer(&timer, session)
        }),
        window_event_listener(ev::keydown, {
            let timer = Rc::clone(&timer);
            move |_| reset_timer(&timer, session)
        }),
        window_event_listener(ev::scroll, {
            let timer = Rc::clone(&timer);
            move |_| reset_timer(&timer, session)
        }),
        window_event_listener(ev::touchstart, {
            let timer = Rc::clone(&timer);
            move |_| reset_timer(&timer, session)
        }),
        window_event_listener(ev::click, {
            let timer = Rc::clone(&timer);
            move |_| reset_timer(&timer, session)
        }),
    ];
    on_cleanup(move || {
        for handle in handles {
            handle.remove();
        }
    });
}
