use crate::api::dto::SessionUser;
use serde::Deserialize;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

const GLOBAL_KEY: &str = "__shokenSessionProbe";

pub enum Probe {
    /// プローブ未起動・または消費済み(起動時の1回しか使わない)。通常経路で確認する
    Missing,
    Authenticated(SessionUser),
    /// 401 が確定したものだけここに入る。ネットワーク失敗は Failed で通常経路へ逃がす
    Anonymous,
    Failed,
}

// session-probe.js が Promise<{state, user?}> で解決する形に合わせる
#[derive(Deserialize)]
struct ProbeResult {
    state: String,
    #[serde(default)]
    user: Option<SessionUser>,
}

// 取り出しと同時に窓から消す。応答済みの Promise は値を保持するため、
// 消さないと2回目以降の check() が古い結果を再利用してしまう
fn probe_promise() -> Option<js_sys::Promise> {
    let window = web_sys::window()?;
    let key = JsValue::from_str(GLOBAL_KEY);
    let value = js_sys::Reflect::get(&window, &key).ok();
    let _ = js_sys::Reflect::delete_property(&window, &key);
    value.and_then(|value| value.dyn_into::<js_sys::Promise>().ok())
}

fn parse_result(value: JsValue) -> Probe {
    let Ok(text) = js_sys::JSON::stringify(&value) else {
        return Probe::Failed;
    };
    let Some(text) = text.as_string() else {
        return Probe::Failed;
    };
    let Ok(result) = serde_json::from_str::<ProbeResult>(&text) else {
        return Probe::Failed;
    };
    match (result.state.as_str(), result.user) {
        ("authenticated", Some(user)) => Probe::Authenticated(user),
        ("anonymous", _) => Probe::Anonymous,
        _ => Probe::Failed,
    }
}

pub async fn take() -> Probe {
    let Some(promise) = probe_promise() else {
        return Probe::Missing;
    };
    match JsFuture::from(promise).await {
        Ok(value) => parse_result(value),
        Err(_) => Probe::Failed,
    }
}
