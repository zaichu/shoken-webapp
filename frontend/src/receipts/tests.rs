use super::store::should_apply_fetch_result;
use super::*;
use crate::api::ApiError;
use crate::receipts_filter::ReceiptSearch;
use crate::session::SessionStore;
use leptos::prelude::*;
use std::collections::{HashMap, HashSet};

fn user(id: &str) -> crate::dto::SessionUser {
    crate::dto::SessionUser {
        id: id.to_string(),
        email: format!("{id}@example.com"),
        name: None,
        picture_url: None,
    }
}

#[test]
fn stale_receipts_result_is_rejected_after_same_or_different_user_login() {
    let owner = Owner::new();
    owner.with(|| {
        for next_user in [user("alice"), user("bob")] {
            let session = SessionStore::new();
            session.user.set(Some(user("alice")));
            let fetch_generation = session.generation.get_untracked();

            session.mark_unauthenticated();
            session.user.set(Some(next_user));

            assert!(!should_apply_fetch_result(&session, fetch_generation));
        }
    });
}

#[test]
fn unauthorized_receipts_error_uses_react_message() {
    assert_eq!(ApiError::http(401).message(), "認証が必要です");
}

#[test]
fn header_summary_uses_api_value_without_preview_or_search() {
    assert_eq!(
        select_header_summary(Some(&"api"), false, "", "client"),
        "api"
    );
}

#[test]
fn header_summary_uses_client_value_during_search() {
    assert_eq!(
        select_header_summary(Some(&"api"), false, "7203", "client"),
        "client"
    );
}

#[test]
fn header_summary_uses_client_value_during_preview_or_without_api_summary() {
    assert_eq!(
        select_header_summary(Some(&"api"), true, "", "client"),
        "client"
    );
    assert_eq!(select_header_summary(None, false, "", "client"), "client");
}

#[test]
fn failed_tabs_are_not_fetched_again_in_the_same_generation() {
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        session.user.set(Some(user("alice")));
        let generation = session.generation.get_untracked();
        let fetch = Action::new_unsync(|_: &(u64, ReceiptsTab)| async {});

        for tab in ReceiptsTab::ALL {
            let cache = RwSignal::new(HashMap::from([(
                (generation, tab),
                TabState::Failed("データ取得に失敗しました".to_string()),
            )]));
            let store = ReceiptsStore {
                session,
                active_tab: RwSignal::new(tab),
                search: RwSignal::new(ReceiptSearch::default()),
                expanded: RwSignal::new(HashSet::new()),
                mobile_summary_expanded: RwSignal::new(false),
                expanded_epoch: RwSignal::new(None),
                visited: RwSignal::new(HashSet::from([tab])),
                cache,
                fetch,
                csv: RwSignal::new(HashMap::new()),
                csv_files: RwSignal::new(HashMap::new()),
            };

            let ensure_result =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| store.ensure(tab)));

            assert!(ensure_result.is_ok(), "失敗済みタブを再取得しようとした");
            assert!(matches!(
                cache.with_untracked(|map| map.get(&(generation, tab)).cloned()),
                Some(TabState::Failed(_))
            ));
        }
    });
}

#[test]
fn expanded_state_is_cleared_on_generation_change() {
    let _ = any_spawner::Executor::init_futures_executor();
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        session.user.set(Some(user("alice")));
        let tab = ReceiptsTab::Dividend;
        let store = ReceiptsStore {
            session,
            active_tab: RwSignal::new(tab),
            search: RwSignal::new(ReceiptSearch::default()),
            expanded: RwSignal::new(HashSet::new()),
            mobile_summary_expanded: RwSignal::new(false),
            expanded_epoch: RwSignal::new(None),
            visited: RwSignal::new(HashSet::from([tab])),
            cache: RwSignal::new(HashMap::new()),
            fetch: Action::new_unsync(|_: &(u64, ReceiptsTab)| async {}),
            csv: RwSignal::new(HashMap::new()),
            csv_files: RwSignal::new(HashMap::new()),
        };

        store.ensure(tab);
        store.expanded.update(|set| {
            set.insert("g0".to_string());
        });
        store.mobile_summary_expanded.set(true);

        session.mark_unauthenticated();
        session.user.set(Some(user("bob")));
        store.ensure(tab);

        assert!(store.expanded.with_untracked(|set| set.is_empty()));
        assert!(!store.mobile_summary_expanded.get_untracked());
    });
}

#[test]
fn expanded_state_survives_ensure_in_same_generation() {
    let _ = any_spawner::Executor::init_futures_executor();
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        session.user.set(Some(user("alice")));
        let tab = ReceiptsTab::Dividend;
        let store = ReceiptsStore {
            session,
            active_tab: RwSignal::new(tab),
            search: RwSignal::new(ReceiptSearch::default()),
            expanded: RwSignal::new(HashSet::new()),
            mobile_summary_expanded: RwSignal::new(false),
            expanded_epoch: RwSignal::new(None),
            visited: RwSignal::new(HashSet::from([tab])),
            cache: RwSignal::new(HashMap::new()),
            fetch: Action::new_unsync(|_: &(u64, ReceiptsTab)| async {}),
            csv: RwSignal::new(HashMap::new()),
            csv_files: RwSignal::new(HashMap::new()),
        };

        store.ensure(tab);
        store.expanded.update(|set| {
            set.insert("g0".to_string());
        });
        store.mobile_summary_expanded.set(true);
        store.ensure(tab);

        assert!(store.expanded.with_untracked(|set| set.contains("g0")));
        assert!(store.mobile_summary_expanded.get_untracked());
    });
}

#[test]
fn dividend_cells_match_react_columns_and_formatting() {
    let row: crate::dto::Dividend = serde_json::from_value(serde_json::json!({
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "settlement_date": "2024-03-01",
        "product": "特定口座",
        "account": "SBI証券",
        "security_code": "7203",
        "security_name": "トヨタ自動車",
        "unit_price": 30.0,
        "shares": 100,
        "dividends_before_tax": 3000,
        "taxes": 609,
        "net_amount_received": 2391,
        "created_at": "2024-03-01T00:00:00Z",
        "updated_at": "2024-03-01T00:00:00Z"
    }))
    .expect("deserialize");
    let cells = ReceiptItem::Dividend(row).cells();
    assert_eq!(
        cells.iter().map(ReceiptCell::text).collect::<Vec<_>>(),
        vec![
            "2024/03/01",
            "特定口座",
            "SBI証券",
            "7203",
            "トヨタ自動車",
            "¥ 30",
            "100",
            "¥ 3,000",
            "¥ 609",
            "¥ 2,391",
        ]
    );
    assert_eq!(cells[3], ReceiptCell::SecurityCode("7203".to_string()));
    assert_eq!(
        cells[4],
        ReceiptCell::InstrumentName {
            name: "トヨタ自動車".to_string(),
            code: Some("7203".to_string()),
        }
    );
}
