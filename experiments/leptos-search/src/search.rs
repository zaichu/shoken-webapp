use crate::api::{Stock, fetch_stock};
use leptos::prelude::*;

pub struct StockSearch {
    pub stock_code: RwSignal<String>,
    pub search: Action<String, Result<Stock, String>>,
    pub has_invalid_code_param: bool,
}

impl StockSearch {
    pub fn stock_data(&self) -> Option<Stock> {
        self.search.value().get().and_then(|result| result.ok())
    }

    pub fn error_message(&self) -> Option<String> {
        self.search.value().get().and_then(|result| result.err())
    }

    pub fn search_by_code(&self, code: String) {
        self.stock_code.set(code.clone());
        self.search.dispatch(code);
    }
}

pub fn use_stock_search() -> StockSearch {
    let (initial, has_invalid_code_param) = read_code_param();
    let stock_code = RwSignal::new(initial.clone());
    let search = Action::new_unsync(|query: &String| {
        let query = query.clone();
        async move { fetch_stock(&query).await }
    });
    let stock_search = StockSearch {
        stock_code,
        search,
        has_invalid_code_param,
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
    if valid_code(&trimmed) {
        (trimmed, false)
    } else {
        (String::new(), true)
    }
}

fn valid_code(code: &str) -> bool {
    !code.is_empty() && code.chars().all(|c| c.is_ascii_alphanumeric() || c == '.')
}

fn read_query_value(key: &str) -> Option<String> {
    let window = web_sys::window()?;
    let search = window.location().search().ok()?;
    let query = search.strip_prefix('?')?;
    for pair in query.split('&') {
        let mut parts = pair.splitn(2, '=');
        if parts.next() == Some(key) {
            let raw = parts.next().unwrap_or("").replace('+', " ");
            return urlencoding::decode(&raw).ok().map(|s| s.into_owned());
        }
    }
    None
}
