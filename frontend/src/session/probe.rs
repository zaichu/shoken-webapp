use crate::api::dto::SessionUser;
use serde::Deserialize;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

const GLOBAL_KEY: &str = "__shokenSessionProbe";

pub enum Probe {
    /// プローブ未起動(古い index.html・テスト環境など)。通常経路で確認する
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

fn probe_promise() -> Option<js_sys::Promise> {
    let window = web_sys::window()?;
    js_sys::Reflect::get(&window, &JsValue::from_str(GLOBAL_KEY))
        .ok()
        .and_then(|value| value.dyn_into::<js_sys::Promise>().ok())
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
