//! 拡張機能との `chrome.runtime.connect` 通信。
//! wasm から直接 Reflect で触り、JS の橋渡しファイルは置かない。
//! CSV の中身(個人情報)をログに出さないため、受信メッセージの中身はここで型に載せるだけ

use super::{CsvKind, FetchFailure, ReceivedCsv};
use serde::Deserialize;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use wasm_bindgen::{JsCast, JsValue, closure::Closure};

/// 順に試す拡張機能 ID(公開版 → 手元のテスト版)。公開用の値で秘密ではない
const EXTENSION_IDS: [&str; 2] = [
    "hneplimfghlcckeielonpeklmjagmceh",
    "kmmhlbafephbeepealpilibeollofeip",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ProgressState {
    LoginWait,
    Fetching,
    Completed,
    Failed,
}

/// ポート経由の出来事。ストアの状態遷移はここに写す
pub(crate) enum Event {
    Progress {
        state: ProgressState,
        kind: Option<CsvKind>,
        reason: Option<String>,
    },
    Result {
        csvs: Vec<ReceivedCsv>,
        failures: Vec<FetchFailure>,
    },
    /// 拡張機能が {type:'error'} を返した(形式不正・別実行中など)
    ErrorMessage(String),
    /// 結果を受け取る前にポートが切れた
    Disconnected,
    /// どの拡張機能 ID にも接続できなかった
    Unavailable,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum ExtensionMessage {
    #[serde(rename = "progress")]
    Progress {
        state: String,
        kind: Option<String>,
        reason: Option<String>,
    },
    #[serde(rename = "result")]
    Result {
        csvs: Vec<FetchedCsvMessage>,
        failures: Vec<FailureMessage>,
    },
    #[serde(rename = "error")]
    Error { error: String },
}

#[derive(Deserialize)]
struct FetchedCsvMessage {
    kind: String,
    content: String,
    #[serde(rename = "fetchedAt")]
    fetched_at: String,
}

#[derive(Deserialize)]
struct FailureMessage {
    kind: String,
    reason: String,
    code: Option<String>,
}

fn parse_message(raw: &JsValue) -> Option<ExtensionMessage> {
    let text = js_sys::JSON::stringify(raw).ok()?.as_string()?;
    serde_json::from_str(&text).ok()
}

type PortListener = Closure<dyn FnMut(JsValue)>;

struct FetchContext {
    on_event: Box<dyn Fn(Event)>,
    /// result・error を受けて自分で切ったあとは、切断を失敗として扱わない
    settled: Cell<bool>,
    /// メッセージが1通でも届いたポートか(届く前に切れた ID は「拡張機能なし」として次を試す)
    saw_message: Cell<bool>,
    next_id: Cell<usize>,
    port: RefCell<JsValue>,
    /// 実行中のリスナーを保持する。FetchContext を Rc で持つ限りコールバックは生きる
    _listeners: RefCell<Vec<PortListener>>,
}

/// 実行中の接続。捨てる前に disconnect() で切る
pub(crate) struct ActiveFetch {
    ctx: Rc<FetchContext>,
}

impl ActiveFetch {
    pub(crate) fn disconnect(&self) {
        self.ctx.settled.set(true);
        let port = self.ctx.port.borrow().clone();
        call0(&port, "disconnect");
        // リスナーの Closure が ctx の Rc を持つため、クリアして循環を切る
        self.ctx._listeners.borrow_mut().clear();
    }
}

/// chrome.runtime.connect で拡張機能に依頼を出す。
/// chrome が無い・connect が無い環境では None を返し、呼び出し側が Unavailable に倒す
pub(crate) fn start(on_event: impl Fn(Event) + 'static) -> Option<ActiveFetch> {
    let runtime = chrome_runtime()?;
    let ctx = Rc::new(FetchContext {
        on_event: Box::new(on_event),
        settled: Cell::new(false),
        saw_message: Cell::new(false),
        next_id: Cell::new(0),
        port: RefCell::new(JsValue::UNDEFINED),
        _listeners: RefCell::new(Vec::new()),
    });
    try_connect(&ctx, &runtime)?;
    Some(ActiveFetch { ctx })
}

fn chrome_runtime() -> Option<JsValue> {
    let window = web_sys::window()?;
    let chrome = js_sys::Reflect::get(&window, &JsValue::from_str("chrome")).ok()?;
    if chrome.is_null() || chrome.is_undefined() {
        return None;
    }
    let runtime = js_sys::Reflect::get(&chrome, &JsValue::from_str("runtime")).ok()?;
    if runtime.is_null() || runtime.is_undefined() {
        return None;
    }
    js_sys::Reflect::get(&runtime, &JsValue::from_str("connect"))
        .ok()?
        .dyn_into::<js_sys::Function>()
        .ok()?;
    Some(runtime)
}

/// extension_id へ接続し、受信・切断のリスナーを付けて依頼を投げる。
/// 接続そのものが成立しなかった(全 ID 失敗)ときだけ None
fn try_connect(ctx: &Rc<FetchContext>, runtime: &JsValue) -> Option<JsValue> {
    for (index, extension_id) in EXTENSION_IDS.iter().enumerate().skip(ctx.next_id.get()) {
        ctx.next_id.set(index + 1);
        let Some(port) = connect(runtime, extension_id) else {
            continue;
        };
        if listen(ctx, runtime, &port) {
            *ctx.port.borrow_mut() = port.clone();
            post(&port, &request_message());
            return Some(port);
        }
    }
    None
}

fn connect(runtime: &JsValue, extension_id: &str) -> Option<JsValue> {
    let connect = js_sys::Reflect::get(runtime, &JsValue::from_str("connect"))
        .ok()?
        .dyn_into::<js_sys::Function>()
        .ok()?;
    let port = connect
        .call1(runtime, &JsValue::from_str(extension_id))
        .ok()?;
    (!port.is_null() && !port.is_undefined()).then_some(port)
}

fn request_message() -> JsValue {
    let kinds = js_sys::Array::new();
    for kind in CsvKind::ALL {
        kinds.push(&JsValue::from_str(kind.request_value()));
    }
    let request = js_sys::Object::new();
    let _ = js_sys::Reflect::set(
        &request,
        &JsValue::from_str("type"),
        &JsValue::from_str("fetchLatestCsv"),
    );
    let _ = js_sys::Reflect::set(&request, &JsValue::from_str("kinds"), &kinds);
    request.into()
}

/// onMessage / onDisconnect のリスナーを付ける。付けられなければ false(=この ID は使えない)
fn listen(ctx: &Rc<FetchContext>, runtime: &JsValue, port: &JsValue) -> bool {
    let on_message = {
        let ctx = Rc::clone(ctx);
        Closure::new(move |raw: JsValue| {
            ctx.saw_message.set(true);
            handle_message(&ctx, &raw);
        })
    };
    if !add_listener(port, "onMessage", &on_message) {
        return false;
    }
    let on_disconnect = {
        let ctx = Rc::clone(ctx);
        let runtime = runtime.clone();
        Closure::new(move |_: JsValue| {
            if ctx.settled.get() {
                return;
            }
            // 1通も届かず切れた ID は「その拡張機能が無い」として次を試す
            if !ctx.saw_message.get() && try_connect(&ctx, &runtime).is_some() {
                return;
            }
            ctx.settled.set(true);
            let event = if ctx.saw_message.get() {
                Event::Disconnected
            } else {
                Event::Unavailable
            };
            (ctx.on_event)(event);
        })
    };
    if !add_listener(port, "onDisconnect", &on_disconnect) {
        return false;
    }
    ctx._listeners
        .borrow_mut()
        .extend([on_message, on_disconnect]);
    true
}

fn handle_message(ctx: &Rc<FetchContext>, raw: &JsValue) {
    let Some(message) = parse_message(raw) else {
        return;
    };
    match message {
        ExtensionMessage::Progress {
            state,
            kind,
            reason,
        } => {
            let Some(state) = parse_progress_state(&state) else {
                return;
            };
            (ctx.on_event)(Event::Progress {
                state,
                kind: kind.as_deref().and_then(CsvKind::from_request_value),
                reason,
            });
        }
        ExtensionMessage::Result { csvs, failures } => {
            ctx.settled.set(true);
            (ctx.on_event)(Event::Result {
                csvs: csvs
                    .into_iter()
                    .filter_map(|csv| {
                        Some(ReceivedCsv {
                            kind: CsvKind::from_request_value(&csv.kind)?,
                            content: csv.content,
                            fetched_at: csv.fetched_at,
                        })
                    })
                    .collect(),
                failures: failures
                    .into_iter()
                    .filter_map(|failure| {
                        Some(FetchFailure {
                            kind: CsvKind::from_request_value(&failure.kind)?,
                            reason: failure.reason,
                            code: failure.code,
                        })
                    })
                    .collect(),
            });
        }
        ExtensionMessage::Error { error } => {
            ctx.settled.set(true);
            (ctx.on_event)(Event::ErrorMessage(error));
        }
    }
}

fn parse_progress_state(state: &str) -> Option<ProgressState> {
    match state {
        "login-wait" => Some(ProgressState::LoginWait),
        "fetching" => Some(ProgressState::Fetching),
        "completed" => Some(ProgressState::Completed),
        "failed" => Some(ProgressState::Failed),
        _ => None,
    }
}

fn add_listener(target: &JsValue, name: &str, listener: &Closure<dyn FnMut(JsValue)>) -> bool {
    let Ok(emitter) = js_sys::Reflect::get(target, &JsValue::from_str(name)) else {
        return false;
    };
    let Ok(add) = js_sys::Reflect::get(&emitter, &JsValue::from_str("addListener"))
        .and_then(|add| add.dyn_into::<js_sys::Function>())
    else {
        return false;
    };
    add.call1(&emitter, listener.as_ref().unchecked_ref())
        .is_ok()
}

fn post(port: &JsValue, message: &JsValue) {
    let Ok(post) = js_sys::Reflect::get(port, &JsValue::from_str("postMessage"))
        .and_then(|post| post.dyn_into::<js_sys::Function>())
    else {
        return;
    };
    let _ = post.call1(port, message);
}

fn call0(target: &JsValue, name: &str) {
    let Ok(call) = js_sys::Reflect::get(target, &JsValue::from_str(name))
        .and_then(|call| call.dyn_into::<js_sys::Function>())
    else {
        return;
    };
    let _ = call.call0(target);
}
