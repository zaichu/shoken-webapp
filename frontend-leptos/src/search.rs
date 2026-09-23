use crate::api::{fetch_stock, ApiError, Stock};
use crate::session::use_session;
use leptos::prelude::*;

fn should_apply_search_result(session: &crate::session::SessionStore, generation: u64) -> bool {
    session.is_current(generation)
}

pub struct StockSearch {
    pub stock_code: RwSignal<String>,
    pub search: Action<String, Result<Stock, String>>,
    pub has_invalid_code_param: bool,
    fetch_generation: RwSignal<u64>,
    session: crate::session::SessionStore,
}

impl StockSearch {
    pub fn stock_data(&self) -> Option<Stock> {
        if !should_apply_search_result(&self.session, self.fetch_generation.get()) {
            return None;
        }
        self.search.value().get().and_then(|result| result.ok())
    }

    pub fn error_message(&self) -> Option<String> {
        if !should_apply_search_result(&self.session, self.fetch_generation.get()) {
            return None;
        }
        self.search.value().get().and_then(|result| result.err())
    }

    pub fn search_by_code(&self, code: String) {
        self.stock_code.set(code.clone());
        self.fetch_generation
            .set(self.session.generation.get_untracked());
        self.search.dispatch(code);
    }
}

fn stock_error_message(error: &ApiError) -> String {
    match error.status() {
        Some(404) => "指定された銘柄が見つかりませんでした。".to_string(),
        Some(401) => error.user_message(),
        Some(_) => "銘柄情報の取得に失敗しました。".to_string(),
        None => match error {
            ApiError::Parse => {
                "銘柄データの読み込みに失敗しました。データ形式が変更された可能性があります。"
                    .to_string()
            }
            _ => "銘柄情報の取得に失敗しました。".to_string(),
        },
    }
}

pub fn use_stock_search() -> StockSearch {
    let (initial, has_invalid_code_param) = read_code_param();
    let session = use_session();
    let stock_code = RwSignal::new(initial.clone());
    let fetch_generation = RwSignal::new(session.generation.get_untracked());
    let search = Action::new_unsync(move |query: &String| {
        let query = query.clone();
        async move {
            match fetch_stock(&query).await {
                Ok(stock) => Ok(stock),
                Err(error) => Err(stock_error_message(&error)),
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::SessionUser;
    use crate::session::SessionStore;

    fn user(id: &str) -> SessionUser {
        SessionUser {
            id: id.to_string(),
            email: format!("{id}@example.com"),
            name: None,
            picture_url: None,
        }
    }

    #[test]
    fn stale_search_result_is_rejected_after_same_or_different_user_login() {
        let owner = Owner::new();
        owner.with(|| {
            for next_user in [user("alice"), user("bob")] {
                let session = SessionStore::new();
                session.user.set(Some(user("alice")));
                let fetch_generation = session.generation.get_untracked();

                session.mark_unauthenticated();
                session.user.set(Some(next_user));

                assert!(!should_apply_search_result(&session, fetch_generation));
            }
        });
    }

    #[test]
    fn unauthorized_search_error_uses_react_message() {
        assert_eq!(
            stock_error_message(&ApiError::Http { status: 401 }),
            "認証が必要です"
        );
    }
}
