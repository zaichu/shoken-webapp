mod view;

pub(crate) use view::StockSearchPage;

use crate::api::{ApiError, Stock, fetch_stock};
use crate::session::{Generation, use_session};
use crate::support::list_search::is_searchable_code;
use leptos::prelude::*;

#[derive(Clone, Copy)]
pub struct StockSearch {
    pub stock_code: RwSignal<String>,
    pub search: Action<String, Result<Stock, ApiError>>,
    pub has_invalid_code_param: bool,
    fetch_generation: RwSignal<Generation>,
    session: crate::session::SessionStore,
}

impl StockSearch {
    pub fn stock_data(&self) -> Option<Stock> {
        if !self.session.is_current(self.fetch_generation.get()) {
            return None;
        }
        self.search.value().get().and_then(std::result::Result::ok)
    }

    pub fn error_message(&self) -> Option<String> {
        if !self.session.is_current(self.fetch_generation.get()) {
            return None;
        }
        self.search
            .value()
            .get()
            .and_then(std::result::Result::err)
            .filter(|error| !is_not_found(error))
            .map(|error| error.message())
    }

    pub fn is_not_found(&self) -> bool {
        if !self.session.is_current(self.fetch_generation.get()) {
            return false;
        }
        self.search
            .value()
            .get()
            .and_then(std::result::Result::err)
            .is_some_and(|error| is_not_found(&error))
    }

    pub fn search_by_code(&self, code: String) {
        self.stock_code.set(code.clone());
        self.fetch_generation
            .set(self.session.generation.get_untracked());
        self.search.dispatch(code);
    }
}

const fn is_not_found(error: &ApiError) -> bool {
    matches!(error, ApiError::Http { status: 404, .. })
}

pub fn use_stock_search() -> StockSearch {
    let (initial, has_invalid_code_param) = read_code_param();
    let session = use_session();
    let stock_code = RwSignal::new(initial.clone());
    let fetch_generation = RwSignal::new(session.generation.get_untracked());
    let search = Action::new_unsync(move |query: &String| {
        let query = query.clone();
        async move { fetch_stock(&query).await }
    });
    let stock_search = StockSearch {
        stock_code,
        search,
        has_invalid_code_param,
        fetch_generation,
        session,
    };
    if !initial.is_empty() {
        stock_search.search_by_code(initial);
    }
    stock_search
}

fn read_code_param() -> (String, bool) {
    let raw = read_query_value("code").unwrap_or_default();
    if raw.is_empty() {
        return (String::new(), false);
    }
    let trimmed = raw.trim().to_string();
    if trimmed.is_empty() {
        return (String::new(), false);
    }
    if is_searchable_code(&trimmed) {
        (trimmed, false)
    } else {
        (String::new(), true)
    }
}

fn read_query_value(key: &str) -> Option<String> {
    let window = web_sys::window()?;
    let search = window.location().search().ok()?;
    let query = search.strip_prefix('?')?;
    for pair in query.split('&') {
        let mut parts = pair.splitn(2, '=');
        if parts.next() == Some(key) {
            let raw = parts.next().unwrap_or("").replace('+', " ");
            return urlencoding::decode(&raw)
                .ok()
                .map(std::borrow::Cow::into_owned);
        }
    }
    None
}

/// URL の `?code=` から銘柄コードを読む。更新・戻る・リンクで開いた画面の
/// 保有解決と、名前検索の成功同期の比較に使う
pub(crate) fn url_stock_code() -> String {
    read_code_param().0
}

/// 検索した銘柄コードを URL(`?code=`)に載せる。更新・戻る・リンクで同じ画面に
/// 戻れるよう履歴へ積み、アプリの現在パスを進めて再マウントさせる。
/// 名前検索などコードにならない入力は URL に載せない。
/// 遷移できたとき true(呼び出し側の直接 dispatch は要らない)を返す
pub(crate) fn sync_code_param(code: &str) -> bool {
    if !is_searchable_code(code) {
        return false;
    }
    let Some(current) = use_context::<crate::app::CurrentPath>() else {
        return false;
    };
    let path = format!("/search?code={}", urlencoding::encode(code));
    if let Some(window) = web_sys::window()
        && let Ok(history) = window.history()
    {
        let _ = history.push_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(&path));
    }
    current.0.set(path);
    true
}

#[cfg(test)]
mod tests;
