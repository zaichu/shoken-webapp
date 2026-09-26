use crate::session::pending_logout;
use crate::session::SessionStore;
use leptos::ev;
use leptos::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};

const CHANNEL_NAME: &str = "shoken_session";
const LOGOUT_MESSAGE: &str = "logout";
const LOGOUT_NOTIFY_KEY: &str = "logout_at";
pub const ACTIVITY_KEY: &str = "last_activity_at";

// 最終操作時刻の書き込みを間引く幅。取りこぼしは発火時の共有時刻確認で吸収する
pub const ACTIVITY_THROTTLE_MS: u64 = 10_000;

fn storage() -> Option<web_sys::Storage> {
    web_sys::window().and_then(|window| window.local_storage().ok().flatten())
}

thread_local! {
    // None=未試行、Some(None)=構築不可、Some(Some)=利用中
    static CHANNEL: RefCell<Option<Option<web_sys::BroadcastChannel>>> =
        const { RefCell::new(None) };
}

fn broadcast_channel() -> Option<web_sys::BroadcastChannel> {
    CHANNEL.with(|slot| {
        slot.borrow_mut()
            .get_or_insert_with(|| web_sys::BroadcastChannel::new(CHANNEL_NAME).ok())
            .clone()
    })
}

// APIを呼ぶのは実行したタブだけ。通知を受けたタブはローカルの認証状態だけを捨てる
pub fn notify_logout() {
    if let Some(channel) = broadcast_channel() {
        let _ = channel.post_message(&JsValue::from_str(LOGOUT_MESSAGE));
    } else if let Some(storage) = storage() {
        let _ = storage.set_item(LOGOUT_NOTIFY_KEY, &(js_sys::Date::now() as u64).to_string());
    }
}

thread_local! {
    // 通知の購読はアプリの生存期間中ずっと有効にするため、ドロップされない場所に保持する
    static KEEP_ALIVE: RefCell<Vec<Box<dyn std::any::Any>>> = const { RefCell::new(Vec::new()) };
}

pub fn watch_logout_notifications(session: SessionStore) {
    if let Some(channel) = broadcast_channel() {
        let onmessage = Closure::wrap(Box::new(move |event: web_sys::MessageEvent| {
            if event.data().as_string().as_deref() == Some(LOGOUT_MESSAGE) {
                session.mark_unauthenticated();
                session.loaded.set(true);
            }
        }) as Box<dyn FnMut(_)>);
        channel.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
        KEEP_ALIVE.with(|keep| {
            keep.borrow_mut()
                .push(Box::new((channel, onmessage)) as Box<dyn std::any::Any>);
        });
    } else {
        let handle = window_event_listener(ev::storage, move |event| {
            if event.key().as_deref() == Some(LOGOUT_NOTIFY_KEY) {
                session.mark_unauthenticated();
                session.loaded.set(true);
            }
        });
        on_cleanup(move || handle.remove());
    }
}

thread_local! {
    static LAST_WRITTEN_MS: Cell<u64> = const { Cell::new(0) };
}

fn should_record(now_ms: u64, last_written_ms: u64) -> bool {
    now_ms.saturating_sub(last_written_ms) >= ACTIVITY_THROTTLE_MS
}

pub fn record_activity() {
    let now = js_sys::Date::now() as u64;
    if !LAST_WRITTEN_MS.with(|last| should_record(now, last.get())) {
        return;
    }
    LAST_WRITTEN_MS.with(|last| last.set(now));
    if let Some(storage) = storage() {
        let _ = storage.set_item(ACTIVITY_KEY, &now.to_string());
    }
}

pub fn last_activity_ms() -> Option<u64> {
    storage()?
        .get_item(ACTIVITY_KEY)
        .ok()
        .flatten()
        .and_then(|value| value.parse::<u64>().ok())
}

const LOGOUT_CLAIM_LOCK: &str = "logout_claim";

fn claim_logout_once_local() -> bool {
    if pending_logout::is_pending() {
        return false;
    }
    pending_logout::mark();
    true
}

// 別タブと同時に期限が切れても DELETE を送るのは1つだけにする。
// navigator.locks が無い環境では読み書きの競合が残るが、DELETE は冪等なのでベストエフォートに落とす
pub async fn claim_logout_once() -> bool {
    let Some(request) = web_sys::window()
        .and_then(|window| js_sys::Reflect::get(&window.navigator(), &"locks".into()).ok())
        .and_then(|locks| {
            js_sys::Reflect::get(&locks, &"request".into())
                .ok()
                .and_then(|request| request.dyn_into::<js_sys::Function>().ok())
                .map(|request| (locks, request))
        })
    else {
        return claim_logout_once_local();
    };
    let (locks, request) = request;
    let claimed = Rc::new(Cell::new(false));
    let callback = Closure::wrap(Box::new({
        let claimed = Rc::clone(&claimed);
        move |_lock: JsValue| {
            if !pending_logout::is_pending() {
                pending_logout::mark();
                claimed.set(true);
            }
        }
    }) as Box<dyn FnMut(JsValue)>);
    let promise = request
        .call2(
            &locks,
            &JsValue::from_str(LOGOUT_CLAIM_LOCK),
            callback.as_ref().unchecked_ref(),
        )
        .ok()
        .and_then(|value| value.dyn_into::<js_sys::Promise>().ok());
    let Some(promise) = promise else {
        return claim_logout_once_local();
    };
    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
    claimed.get()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn activity_write_is_throttled() {
        assert!(should_record(u64::MAX, 0));
        assert!(should_record(ACTIVITY_THROTTLE_MS, 0));
        assert!(!should_record(ACTIVITY_THROTTLE_MS - 1, 0));
        assert!(should_record(30_000, 10_000));
        assert!(!should_record(19_999, 10_000));
    }
}
