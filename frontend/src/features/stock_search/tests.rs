use super::*;
use crate::api::dto::SessionUser;
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
fn search_result_applies_within_the_same_generation() {
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        session.user.set(Some(user("alice")));
        let generation = session.generation.get_untracked();
        assert!(should_apply_search_result(&session, generation));
    });
}

#[test]
fn valid_code_matches_react_regex() {
    assert!(valid_code("7203"));
    assert!(valid_code("BRK.B"));
    assert!(!valid_code(""));
    assert!(!valid_code("7203: トヨタ"));
    assert!(!valid_code("７２０３"));
    assert!(!valid_code("A B"));
}

fn stock(code: &str) -> Stock {
    Stock {
        code: code.to_string(),
        name: "テスト".to_string(),
        date: String::new(),
        market_category: String::new(),
        industry_code_33: None,
        industry_category_33: None,
        industry_code_17: None,
        industry_category_17: None,
        size_code: None,
        size_category: None,
    }
}

fn search_with(value: Option<Result<Stock, ApiError>>) -> Action<String, Result<Stock, ApiError>> {
    Action::new_local_with_value(value, |query: &String| {
        let query = query.clone();
        async move { Ok(stock(&query)) }
    })
}

#[test]
fn stock_data_and_error_message_follow_result_and_generation() {
    let _ = any_spawner::Executor::init_futures_executor();
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        session.user.set(Some(user("alice")));
        let generation = session.generation.get_untracked();

        let ok = StockSearch {
            stock_code: RwSignal::new(String::new()),
            search: search_with(Some(Ok(stock("7203")))),
            has_invalid_code_param: false,
            fetch_generation: RwSignal::new(generation),
            session,
        };
        ok.search_by_code("7203".to_string());
        assert_eq!(ok.stock_code.get_untracked(), "7203");
        assert_eq!(
            ok.stock_data().map(|stock| stock.code),
            Some("7203".to_string())
        );
        assert!(ok.error_message().is_none());
        ok.fetch_generation.set(generation.next());
        assert!(ok.stock_data().is_none());
        assert!(ok.error_message().is_none());

        let err = StockSearch {
            stock_code: RwSignal::new(String::new()),
            search: search_with(Some(Err(ApiError::Http {
                status: 400,
                server_message: Some("取得失敗".to_string()),
            }))),
            has_invalid_code_param: false,
            fetch_generation: RwSignal::new(generation),
            session,
        };
        assert!(err.stock_data().is_none());
        assert_eq!(err.error_message().as_deref(), Some("取得失敗"));
        assert!(!err.is_not_found());
        err.fetch_generation.set(generation.next());
        assert!(err.error_message().is_none());

        let not_found = StockSearch {
            stock_code: RwSignal::new(String::new()),
            search: search_with(Some(Err(ApiError::http(404)))),
            has_invalid_code_param: false,
            fetch_generation: RwSignal::new(generation),
            session,
        };
        assert!(not_found.stock_data().is_none());
        assert!(not_found.error_message().is_none());
        assert!(not_found.is_not_found());
        not_found.fetch_generation.set(generation.next());
        assert!(!not_found.is_not_found());
    });
}
