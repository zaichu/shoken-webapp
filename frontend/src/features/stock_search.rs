mod view;

pub(crate) use view::StockSearchPage;

use crate::api::{fetch_stock, ApiError, Stock};
use crate::session::{use_session, Generation};
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
        self.search.value().get().and_then(|result| result.ok())
    }

    pub fn error_message(&self) -> Option<String> {
        if !self.session.is_current(self.fetch_generation.get()) {
            return None;
        }
        self.search
            .value()
            .get()
            .and_then(|result| result.err())
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
            .and_then(|result| result.err())
            .is_some_and(|error| is_not_found(&error))
    }

    pub fn search_by_code(&self, code: String) {
        self.stock_code.set(code.clone());
        self.fetch_generation
            .set(self.session.generation.get_untracked());
        self.search.dispatch(code);
    }
}

fn is_not_found(error: &ApiError) -> bool {
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
            return urlencoding::decode(&raw).ok().map(|s| s.into_owned());
        }
    }
    None
}

#[cfg(test)]
mod tests;
