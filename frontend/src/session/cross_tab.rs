use crate::session::pending_logout;
use crate::session::SessionStore;
use leptos::ev;
use leptos::prelude::*;
use std::cell::{Cell, RefCell};
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
const LOGOUT_CLAIMED_KEY: &str = "logout_claimed_at";
// 次の無操作ログアウトは最短でも30分後なので、この幅なら次回と取り違えない
const RECENT_CLAIM_MS: u64 = 60_000;

fn is_recent_claim(now_ms: u64, claimed_at_ms: u64) -> bool {
    now_ms.saturating_sub(claimed_at_ms) < RECENT_CLAIM_MS
}

fn try_claim() -> bool {
    let now = js_sys::Date::now() as u64;
    let recently_claimed = storage()
        .and_then(|storage| storage.get_item(LOGOUT_CLAIMED_KEY).ok().flatten())
        .and_then(|value| value.parse::<u64>().ok())
        .is_some_and(|claimed_at| is_recent_claim(now, claimed_at));
    if pending_logout::is_pending() || recently_claimed {
        return false;
    }
    pending_logout::mark();
    if let Some(storage) = storage() {
        let _ = storage.set_item(LOGOUT_CLAIMED_KEY, &now.to_string());
    }
    true
}

// localStorage はタブ間で即時に同期されないので、取ったタブがロックを保持して他タブを ifAvailable で諦めさせる
pub async fn claim_logout_once() -> bool {
    let Some((locks, request)) = web_sys::window()
        .and_then(|window| js_sys::Reflect::get(&window.navigator(), &"locks".into()).ok())
        .and_then(|locks| {
            js_sys::Reflect::get(&locks, &"request".into())
                .ok()
                .and_then(|request| request.dyn_into::<js_sys::Function>().ok())
                .map(|request| (locks, request))
        })
    else {
        return try_claim();
    };
    let mut resolve_slot = None;
    let decision = js_sys::Promise::new(&mut |resolve, _reject| resolve_slot = Some(resolve));
    let Some(resolve) = resolve_slot else {
        return try_claim();
    };
    let on_rejected_resolve = resolve.clone();
    let callback = Closure::wrap(Box::new(move |lock: JsValue| -> JsValue {
        let claimed = !lock.is_null() && try_claim();
        let _ = resolve.call1(&JsValue::NULL, &JsValue::from_bool(claimed));
        if !claimed {
            return JsValue::UNDEFINED;
        }
        js_sys::Promise::new(&mut |release, _reject| {
            if let Some(window) = web_sys::window() {
                let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                    &release,
                    RECENT_CLAIM_MS as i32,
                );
            }
        })
        .into()
    }) as Box<dyn FnMut(JsValue) -> JsValue>);
    let options = js_sys::Object::new();
    let _ = js_sys::Reflect::set(&options, &"ifAvailable".into(), &JsValue::TRUE);
    let Some(requested) = request
        .call3(
            &locks,
            &JsValue::from_str(LOGOUT_CLAIM_LOCK),
            &options,
            callback.as_ref().unchecked_ref(),
        )
        .ok()
        .and_then(|value| value.dyn_into::<js_sys::Promise>().ok())
    else {
        return try_claim();
    };
    // callback を呼ばずに reject されると decision が決まらないため
    let on_rejected = Closure::once(move |_error: JsValue| {
        let _ = on_rejected_resolve.call1(&JsValue::NULL, &JsValue::from_bool(try_claim()));
    });
    let settled = requested.catch(&on_rejected);
    let claimed = wasm_bindgen_futures::JsFuture::from(decision)
        .await
        .ok()
        .and_then(|value| value.as_bool())
        .unwrap_or(false);
    // ロックの保持が終わって要求が決着するまで、JS から呼ばれうるクロージャを生かしておく
    leptos::task::spawn_local(async move {
        let _ = wasm_bindgen_futures::JsFuture::from(settled).await;
        drop(callback);
        drop(on_rejected);
    });
    claimed
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
