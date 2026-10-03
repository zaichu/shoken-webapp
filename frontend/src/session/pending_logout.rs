use crate::api::{ApiClient, ApiError};
use crate::session::local_storage;
use gloo_timers::future::TimeoutFuture;
use std::cell::Cell;

const STORAGE_KEY: &str = "pending_logout";
const RETRY_DELAYS_MS: [u32; 3] = [10_000, 30_000, 60_000];

fn retry_delay_ms(attempt: u32) -> u32 {
    RETRY_DELAYS_MS[(attempt as usize).min(RETRY_DELAYS_MS.len() - 1)]
}

// 401(サーバー側でセッションが既に無い)も成功扱いにして再試行を止める
pub fn is_finished(result: &Result<(), ApiError>) -> bool {
    match result {
        Ok(()) => true,
        Err(error) => error.is_unauthorized(),
    }
}

trait FlagStore {
    fn is_set(&self) -> bool;
    fn set(&self);
    fn clear(&self);
}

// localStorage が使えない環境では読み書きを捨て、保留なしとして扱う
struct LocalStore;

impl FlagStore for LocalStore {
    fn is_set(&self) -> bool {
        local_storage()
            .and_then(|storage| storage.get_item(STORAGE_KEY).ok().flatten())
            .is_some()
    }

    fn set(&self) {
        if let Some(storage) = local_storage() {
            let _ = storage.set_item(STORAGE_KEY, "1");
        }
    }

    fn clear(&self) {
        if let Some(storage) = local_storage() {
            let _ = storage.remove_item(STORAGE_KEY);
        }
    }
}

fn is_pending_in(store: &impl FlagStore) -> bool {
    store.is_set()
}

fn mark_in(store: &impl FlagStore) {
    store.set();
}

fn clear_in(store: &impl FlagStore) {
    store.clear();
}

pub fn is_pending() -> bool {
    is_pending_in(&LocalStore)
}

pub fn mark() {
    mark_in(&LocalStore);
}

pub fn clear() {
    clear_in(&LocalStore);
}

thread_local! {
    static RETRY_RUNNING: Cell<bool> = const { Cell::new(false) };
}

pub fn start_retry_loop() {
    if RETRY_RUNNING.with(|running| running.replace(true)) {
        return;
    }
    leptos::task::spawn_local(async move {
        let client = ApiClient::default_client();
        let mut attempt = 0;
        while is_pending() {
            TimeoutFuture::new(retry_delay_ms(attempt)).await;
            // 待機中に login 側で再送・解除済みなら、新しいセッションを消し得る余計な DELETE を撃たない
            if !is_pending() {
                break;
            }
            if is_finished(&client.delete_empty("/api/v1/session").await) {
                clear();
            } else {
                attempt += 1;
            }
        }
        RETRY_RUNNING.with(|running| running.set(false));
    });
}

#[cfg(test)]
mod tests;
