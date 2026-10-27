use crate::features::receipts::filter::ReceiptSearch;
use crate::features::receipts::store::{TabCacheEntry, mark_tab_for_refresh, tab_settled};
use crate::features::receipts::*;
use crate::session::{Generation, SessionStore};
use crate::support::csv_flow::{CsvTabMeta, CsvTabState};
use crate::testing::receipts::dividends;
use leptos::prelude::*;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

fn user(id: &str) -> crate::api::dto::SessionUser {
    crate::api::dto::SessionUser {
        id: id.to_string(),
        email: format!("{id}@example.com"),
        name: None,
    }
}

fn test_store(
    session: &SessionStore,
    cache: HashMap<(Generation, ReceiptsTab), TabState>,
    csv: HashMap<(Generation, ReceiptsTab), CsvTabState>,
) -> ReceiptsStore {
    let mut tabs: HashMap<(Generation, ReceiptsTab), TabCacheEntry> = HashMap::new();
    for (key, list) in cache {
        tabs.entry(key).or_default().list = Some(list);
    }
    for (key, csv) in csv {
        tabs.entry(key).or_default().csv = csv;
    }
    ReceiptsStore {
        session: *session,
        active_tab: RwSignal::new(ReceiptsTab::Dividend),
        search: RwSignal::new(ReceiptSearch::default()),
        expanded: RwSignal::new(HashSet::new()),
        mobile_summary_expanded: RwSignal::new(false),
        expanded_epoch: RwSignal::new(None),
        visited: RwSignal::new(HashSet::new()),
        tabs: RwSignal::new(tabs),
        fetch: Action::new_unsync(|_: &(Generation, ReceiptsTab)| async {}),
    }
}

#[test]
fn count_uses_api_total_over_loaded_rows() {
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        session.user.set(Some(user("alice")));
        let generation = session.generation.get_untracked();
        let tab = ReceiptsTab::Dividend;
        let ready = |rows: Vec<ReceiptRow>, total| {
            TabState::Ready(ReceiptTabData {
                rows,
                total,
                summary: None,
                truncated: false,
            })
        };
        let store = test_store(
            &session,
            HashMap::from([((generation, tab), ready(Vec::new(), Some(100_001)))]),
            HashMap::new(),
        );

        // 取得上限で行が欠けても削除確認は保存済み全件数を示す
        assert_eq!(store.count(tab), 100_001);

        let rows = dividends();
        let loaded = rows.len();
        store.tabs.set(HashMap::from([(
            (generation, tab),
            ready(rows, None).into(),
        )]));
        assert_eq!(
            store.count(tab),
            loaded,
            "API total が無いときだけ読み込み済み行数に戻る"
        );
    });
}

#[test]
fn refresh_for_delete_refetches_only_settled_tab() {
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        session.user.set(Some(user("alice")));
        let generation = session.generation.get_untracked();
        let calls = Rc::new(RefCell::new(Vec::new()));
        let record = Rc::clone(&calls);
        let fetch = Action::new_unsync(move |(generation, tab): &(Generation, ReceiptsTab)| {
            record.borrow_mut().push((*generation, *tab));
            async {}
        });
        let mut store = test_store(
            &session,
            HashMap::from([(
                (generation, ReceiptsTab::Dividend),
                TabState::Ready(ReceiptTabData {
                    rows: Vec::new(),
                    total: None,
                    summary: None,
                    truncated: false,
                }),
            )]),
            HashMap::new(),
        );
        store.fetch = fetch;

        store.refresh_for_delete(ReceiptsTab::Dividend);
        assert_eq!(
            calls.borrow().as_slice(),
            &[(generation, ReceiptsTab::Dividend)]
        );

        calls.borrow_mut().clear();
        // Ready を Loading に戻した直後は進行中の取得に任せて重複 dispatch しない
        store.refresh_for_delete(ReceiptsTab::Dividend);
        assert!(calls.borrow().is_empty());
        // 未取得タブは取り直さない
        store.refresh_for_delete(ReceiptsTab::DomesticStock);
        assert!(calls.borrow().is_empty());
    });
}

#[test]
fn csv_meta_is_scoped_to_generation_and_cleared_by_logout() {
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        let tab = ReceiptsTab::Dividend;
        let store = test_store(&session, HashMap::new(), HashMap::new());
        assert_eq!(store.csv_meta(tab), CsvTabMeta::default());

        session.user.set(Some(user("alice")));
        store.open_delete_confirm(tab);
        assert!(store.csv_meta(tab).show_delete_confirm);

        session.mark_unauthenticated();
        assert_eq!(
            store.csv_meta(tab),
            CsvTabMeta::default(),
            "世代が進むとCSV状態は見えなくなる"
        );

        session.user.set(Some(user("bob")));
        assert_eq!(
            store.csv_meta(tab),
            CsvTabMeta::default(),
            "別ユーザーでも見えない"
        );
    });
}

#[test]
fn delete_success_empties_cached_list_and_clears_state() {
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        session.user.set(Some(user("alice")));
        let generation = session.generation.get_untracked();
        let tab = ReceiptsTab::Dividend;
        let store = test_store(
            &session,
            HashMap::from([(
                (generation, tab),
                TabState::Ready(ReceiptTabData {
                    rows: vec![dividends()[0].clone()],
                    total: None,
                    summary: None,
                    truncated: false,
                }),
            )]),
            HashMap::from([(
                (generation, tab),
                CsvTabState {
                    deleting: true,
                    ..Default::default()
                },
            )]),
        );

        store.apply_delete_result(generation, tab, Ok(()));

        assert!(!store.csv_meta(tab).deleting);
        match store.tab_state(tab) {
            TabState::Ready(data) => assert!(data.rows.is_empty()),
            _ => panic!("キャッシュは空の Ready に置き換わる"),
        }
    });
}

#[test]
fn delete_success_without_cached_list_creates_nothing() {
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        session.user.set(Some(user("alice")));
        let generation = session.generation.get_untracked();
        let tab = ReceiptsTab::MutualFund;
        let store = test_store(&session, HashMap::new(), HashMap::new());

        store.apply_delete_result(generation, tab, Ok(()));

        assert!(
            store
                .tabs
                .with_untracked(|map| map.get(&(generation, tab)).is_none_or(|e| e.list.is_none()))
        );
    });
}

#[test]
fn stale_generation_delete_result_is_dropped() {
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        session.user.set(Some(user("alice")));
        let generation = session.generation.get_untracked();
        let tab = ReceiptsTab::Dividend;
        let store = test_store(
            &session,
            HashMap::new(),
            HashMap::from([(
                (generation, tab),
                CsvTabState {
                    show_delete_confirm: true,
                    ..Default::default()
                },
            )]),
        );

        session.mark_unauthenticated();
        store.apply_delete_result(generation, tab, Ok(()));

        let untouched = store.tabs.with_untracked(|map| {
            map.get(&(generation, tab)).is_some_and(|entry| {
                entry.csv.show_delete_confirm && !entry.csv.deleting && entry.csv.error.is_none()
            })
        });
        assert!(untouched, "古い世代の遅れ結果は現在世代に影響しない");
    });
}

#[test]
fn delete_confirm_opens_and_guards_double_confirm() {
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        session.user.set(Some(user("alice")));
        let tab = ReceiptsTab::Dividend;
        let store = test_store(&session, HashMap::new(), HashMap::new());

        store.open_delete_confirm(tab);
        assert!(store.csv_meta(tab).show_delete_confirm);

        assert!(store.try_begin_delete(tab).is_some());
        let meta = store.csv_meta(tab);
        assert!(!meta.show_delete_confirm);
        assert!(meta.deleting);
        assert!(
            store.try_begin_delete(tab).is_none(),
            "deleting 中の確定は開始しない"
        );
    });
}

#[test]
fn csv_operations_require_authentication() {
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        let tab = ReceiptsTab::Dividend;
        let store = test_store(&session, HashMap::new(), HashMap::new());

        assert!(store.try_begin_delete(tab).is_none());
        store.open_delete_confirm(tab);
        assert!(!store.csv_meta(tab).show_delete_confirm);
    });
}

#[test]
fn refresh_tab_list_marks_cached_tab_loading_only() {
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        session.user.set(Some(user("alice")));
        let generation = session.generation.get_untracked();
        let store = test_store(
            &session,
            HashMap::from([(
                (generation, ReceiptsTab::Dividend),
                TabState::Ready(ReceiptTabData {
                    rows: Vec::new(),
                    total: None,
                    summary: None,
                    truncated: false,
                }),
            )]),
            HashMap::new(),
        );

        assert!(store.refresh_tab_list(generation, ReceiptsTab::Dividend));
        assert!(matches!(
            store.tab_state(ReceiptsTab::Dividend),
            TabState::Loading
        ));
        assert!(!store.refresh_tab_list(generation, ReceiptsTab::MutualFund));
        assert!(store.tabs.with_untracked(|map| {
            map.get(&(generation, ReceiptsTab::MutualFund))
                .is_none_or(|e| e.list.is_none())
        }));
    });
}

#[test]
fn mark_tab_for_refresh_marks_only_existing_entry() {
    let mut map: HashMap<(Generation, ReceiptsTab), TabCacheEntry> = HashMap::new();
    assert!(!mark_tab_for_refresh(
        &mut map,
        Generation::new(0),
        ReceiptsTab::Dividend
    ));
    map.entry((Generation::new(0), ReceiptsTab::Dividend))
        .or_default()
        .list = Some(TabState::Ready(ReceiptTabData {
        rows: Vec::new(),
        total: None,
        summary: None,
        truncated: false,
    }));
    assert!(mark_tab_for_refresh(
        &mut map,
        Generation::new(0),
        ReceiptsTab::Dividend
    ));
    assert!(matches!(
        map.get(&(Generation::new(0), ReceiptsTab::Dividend))
            .and_then(|e| e.list.as_ref()),
        Some(TabState::Loading)
    ));
    assert!(!mark_tab_for_refresh(
        &mut map,
        Generation::new(1),
        ReceiptsTab::Dividend
    ));
    assert!(!mark_tab_for_refresh(
        &mut map,
        Generation::new(0),
        ReceiptsTab::MutualFund
    ));
    // CSV 側の操作だけで作られたエントリ(一覧未取得)も再取得対象にしない
    map.entry((Generation::new(0), ReceiptsTab::MutualFund))
        .or_default();
    assert!(!mark_tab_for_refresh(
        &mut map,
        Generation::new(0),
        ReceiptsTab::MutualFund
    ));
}

#[test]
fn confirm_delete_all_requires_open_confirmation() {
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        session.user.set(Some(user("alice")));
        let tab = ReceiptsTab::Dividend;
        let store = test_store(&session, HashMap::new(), HashMap::new());

        store.confirm_delete_all(tab);
        let meta = store.csv_meta(tab);
        assert!(!meta.deleting);
        assert!(meta.error.is_none());
        assert!(store.try_begin_delete(tab).is_none());

        store.open_delete_confirm(tab);
        assert!(store.try_begin_delete(tab).is_some());
    });
}

#[test]
fn confirm_delete_all_marks_state_deleting() {
    let _ = any_spawner::Executor::init_futures_executor();
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        session.user.set(Some(user("alice")));
        let tab = ReceiptsTab::Dividend;
        let store = test_store(&session, HashMap::new(), HashMap::new());

        store.open_delete_confirm(tab);
        store.confirm_delete_all(tab);
        assert!(store.csv_meta(tab).deleting);
    });
}

#[test]
fn tab_labels_and_api_paths_are_defined_per_tab() {
    for (tab, expected) in [
        (
            ReceiptsTab::Dividend,
            (
                "配当金",
                "/api/v1/dividends",
                "/api/v1/dividend-import-validations",
                "/api/v1/dividend-imports",
            ),
        ),
        (
            ReceiptsTab::DomesticStock,
            (
                "国内株式",
                "/api/v1/domestic-stock-transactions",
                "/api/v1/domestic-stock-import-validations",
                "/api/v1/domestic-stock-imports",
            ),
        ),
        (
            ReceiptsTab::MutualFund,
            (
                "投資信託",
                "/api/v1/mutual-fund-transactions",
                "/api/v1/mutual-fund-import-validations",
                "/api/v1/mutual-fund-imports",
            ),
        ),
    ] {
        assert_eq!(
            (
                tab.label(),
                tab.list_path(),
                tab.preview_path(),
                tab.import_path()
            ),
            expected
        );
    }
}

#[test]
fn tab_settled_only_for_ready_or_failed() {
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        session.user.set(Some(user("alice")));
        let store = test_store(&session, HashMap::new(), HashMap::new());
        let tab = ReceiptsTab::Dividend;
        assert!(!tab_settled(&store, Generation::new(0), tab));
        store.tabs.update(|map| {
            map.entry((Generation::new(0), tab)).or_default().list = Some(TabState::Loading);
        });
        assert!(!tab_settled(&store, Generation::new(0), tab));
        store.tabs.update(|map| {
            map.entry((Generation::new(0), tab)).or_default().list =
                Some(TabState::Failed("x".to_string()));
        });
        assert!(tab_settled(&store, Generation::new(0), tab));
        store.tabs.update(|map| {
            map.entry((Generation::new(0), tab)).or_default().list =
                Some(TabState::Ready(ReceiptTabData {
                    rows: Vec::new(),
                    total: None,
                    summary: None,
                    truncated: false,
                }));
        });
        assert!(tab_settled(&store, Generation::new(0), tab));
    });
}

#[test]
fn ensure_prunes_only_stale_generation_entries() {
    let _ = any_spawner::Executor::init_futures_executor();
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        session.user.set(Some(user("alice")));
        let store = test_store(
            &session,
            HashMap::from([(
                (session.generation.get_untracked(), ReceiptsTab::Dividend),
                TabState::Loading,
            )]),
            HashMap::new(),
        );
        let tab = ReceiptsTab::Dividend;
        let stale = session.generation.get_untracked().next();

        store.tabs.update(|map| {
            map.insert((stale, tab), TabCacheEntry::default());
        });
        let generation = session.generation.get_untracked();
        store.tabs.update(|map| {
            map.entry((generation, tab)).or_default().fetch_rev = 1;
        });

        store.ensure(tab);

        assert!(
            store
                .tabs
                .with_untracked(|map| map.contains_key(&(generation, tab)))
        );
        assert!(
            !store
                .tabs
                .with_untracked(|map| map.keys().any(|(cached, _)| *cached == stale))
        );
    });
}

#[test]
fn ensure_prunes_csv_state_when_only_csv_has_stale_entries() {
    let _ = any_spawner::Executor::init_futures_executor();
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        session.user.set(Some(user("alice")));
        let generation = session.generation.get_untracked();
        let stale = session.generation.get_untracked().next();
        let tab = ReceiptsTab::Dividend;
        let store = test_store(
            &session,
            HashMap::from([(
                (generation, tab),
                TabState::Ready(ReceiptTabData {
                    rows: Vec::new(),
                    total: None,
                    summary: None,
                    truncated: false,
                }),
            )]),
            HashMap::from([
                ((generation, tab), CsvTabState::default()),
                ((stale, tab), CsvTabState::default()),
            ]),
        );

        store.ensure(tab);

        assert!(
            store
                .tabs
                .with_untracked(|map| map.contains_key(&(generation, tab)))
        );
        assert!(
            !store
                .tabs
                .with_untracked(|map| map.keys().any(|(cached, _)| *cached == stale))
        );
    });
}

#[test]
fn ensure_keeps_entries_when_nothing_is_stale() {
    let _ = any_spawner::Executor::init_futures_executor();
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        session.user.set(Some(user("alice")));
        let generation = session.generation.get_untracked();
        let tab = ReceiptsTab::Dividend;
        let store = test_store(
            &session,
            HashMap::from([((generation, tab), TabState::Loading)]),
            HashMap::from([((generation, tab), CsvTabState::default())]),
        );

        store.ensure(tab);

        assert!(
            store
                .tabs
                .with_untracked(|map| map.contains_key(&(generation, tab)))
        );
    });
}
