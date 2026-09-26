use super::store::{
    has_stale_generation, mark_tab_for_refresh, prune_stale_generation, tab_settled,
};
use super::*;
use crate::api::ApiError;
use crate::csv_flow::CsvPreview;
use crate::csv_flow::CsvTabState;
use crate::receipts_csv::CsvPreviewRow;
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

fn test_store(
    session: &SessionStore,
    cache: HashMap<(u64, ReceiptsTab), TabState>,
    csv: HashMap<(u64, ReceiptsTab), CsvTabState<CsvPreviewRow>>,
) -> ReceiptsStore {
    ReceiptsStore {
        session: *session,
        active_tab: RwSignal::new(ReceiptsTab::Dividend),
        search: RwSignal::new(ReceiptSearch::default()),
        expanded: RwSignal::new(HashSet::new()),
        mobile_summary_expanded: RwSignal::new(false),
        expanded_epoch: RwSignal::new(None),
        visited: RwSignal::new(HashSet::new()),
        cache: RwSignal::new(cache),
        fetch: Action::new_unsync(|_: &(u64, ReceiptsTab)| async {}),
        csv: RwSignal::new(csv),
        csv_files: RwSignal::new(HashMap::new()),
    }
}

fn upload_response(inserted: usize) -> crate::dto::CsvUploadResponse {
    crate::dto::CsvUploadResponse {
        inserted,
        skipped: 0,
        errors: vec![],
    }
}

#[test]
fn csv_state_is_scoped_to_generation_and_cleared_by_logout() {
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        session.user.set(Some(user("alice")));
        let generation = session.generation.get_untracked();
        let store = test_store(
            &session,
            HashMap::new(),
            HashMap::from([(
                (generation, ReceiptsTab::Dividend),
                CsvTabState {
                    file_name: Some("dividend.csv".to_string()),
                    import_result: Some(upload_response(3)),
                    show_delete_confirm: true,
                    ..Default::default()
                },
            )]),
        );

        assert_eq!(
            store.csv_state(ReceiptsTab::Dividend).file_name.as_deref(),
            Some("dividend.csv")
        );

        session.mark_unauthenticated();
        assert_eq!(
            store.csv_state(ReceiptsTab::Dividend),
            CsvTabState::default(),
            "世代が進むとCSV状態は見えなくなる"
        );

        session.user.set(Some(user("alice")));
        assert_eq!(
            store.csv_state(ReceiptsTab::Dividend),
            CsvTabState::default()
        );
    });
}

#[test]
fn upload_success_sets_result_and_refreshes_cached_list() {
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
                    rows: Vec::new(),
                    summary: None,
                    truncated: false,
                }),
            )]),
            HashMap::from([(
                (generation, tab),
                CsvTabState {
                    file_name: Some("a.csv".to_string()),
                    saving: true,
                    ..Default::default()
                },
            )]),
        );

        assert!(
            store.apply_upload_result(generation, tab, Ok(upload_response(2))),
            "キャッシュ済みタブは再取得対象になる"
        );

        let state = store.csv_state(tab);
        assert!(!state.saving);
        assert!(state.file_name.is_none());
        assert_eq!(
            state.import_result.as_ref().map(|result| result.inserted),
            Some(2)
        );
        assert!(matches!(store.tab_state(tab), TabState::Loading,));
    });
}

#[test]
fn upload_success_does_not_fetch_uncached_tab() {
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        session.user.set(Some(user("alice")));
        let generation = session.generation.get_untracked();
        let tab = ReceiptsTab::MutualFund;
        let store = test_store(&session, HashMap::new(), HashMap::new());

        assert!(!store.apply_upload_result(generation, tab, Ok(upload_response(1))));

        assert!(store
            .cache
            .with_untracked(|map| !map.contains_key(&(generation, tab))));
    });
}

#[test]
fn upload_error_sets_react_message_and_keeps_file() {
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        session.user.set(Some(user("alice")));
        let generation = session.generation.get_untracked();
        let tab = ReceiptsTab::DomesticStock;
        let store = test_store(
            &session,
            HashMap::new(),
            HashMap::from([(
                (generation, tab),
                CsvTabState {
                    file_name: Some("stocks.csv".to_string()),
                    saving: true,
                    ..Default::default()
                },
            )]),
        );

        store.apply_upload_result(generation, tab, Err(ApiError::http(401)));

        let state = store.csv_state(tab);
        assert!(!state.saving);
        assert_eq!(state.file_name.as_deref(), Some("stocks.csv"));
        assert_eq!(state.error.as_deref(), Some("認証が必要です"));
        assert!(store.error().is_none());
    });
}

#[test]
fn preview_result_applies_only_in_current_generation() {
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
                    file_name: Some("a.csv".to_string()),
                    previewing: true,
                    ..Default::default()
                },
            )]),
        );

        let response = crate::dto::CsvPreviewResponse {
            total_rows: 2,
            valid_rows: 2,
            errors: vec![],
            rows: vec![
                serde_json::json!({"security_name": "トヨタ自動車", "shares": 100}),
                serde_json::json!({"security_name": "三菱UFJ", "shares": 200}),
            ],
        };
        store.apply_preview_result(generation, tab, Ok(response));

        let state = store.csv_state(tab);
        assert!(!state.previewing);
        let preview = state.preview.expect("preview set");
        assert_eq!(preview.rows.len(), 2);

        // プレビュー失敗は通知せず解析中だけ解除する
        store.csv.update(|map| {
            map.entry((generation, tab)).or_default().previewing = true;
        });
        store.apply_preview_result(generation, tab, Err(ApiError::http(400)));
        let state = store.csv_state(tab);
        assert!(!state.previewing);
        assert!(state.error.is_none());
    });
}

#[test]
fn delete_success_empties_cached_list_and_clears_result() {
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
                    rows: vec![ReceiptItem::Dividend(
                        serde_json::from_value(serde_json::json!({
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
                        .expect("dividend"),
                    )],
                    summary: None,
                    truncated: false,
                }),
            )]),
            HashMap::from([(
                (generation, tab),
                CsvTabState {
                    import_result: Some(upload_response(1)),
                    deleting: true,
                    ..Default::default()
                },
            )]),
        );

        store.apply_delete_result(generation, tab, Ok(()));

        let state = store.csv_state(tab);
        assert!(!state.deleting);
        assert!(state.import_result.is_none());
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

        assert!(store
            .cache
            .with_untracked(|map| !map.contains_key(&(generation, tab))));
    });
}

#[test]
fn stale_generation_results_are_dropped() {
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        session.user.set(Some(user("alice")));
        let generation = session.generation.get_untracked();
        let tab = ReceiptsTab::Dividend;
        let store = test_store(
            &session,
            HashMap::from([((generation, tab), TabState::Loading)]),
            HashMap::from([(
                (generation, tab),
                CsvTabState {
                    saving: true,
                    deleting: true,
                    previewing: true,
                    ..Default::default()
                },
            )]),
        );

        session.mark_unauthenticated();

        store.apply_preview_result(
            generation,
            tab,
            Ok(crate::dto::CsvPreviewResponse {
                total_rows: 1,
                valid_rows: 1,
                errors: vec![],
                rows: vec![serde_json::json!({})],
            }),
        );
        assert!(!store.apply_upload_result(generation, tab, Ok(upload_response(1))));
        store.apply_delete_result(generation, tab, Ok(()));

        let stale = store
            .csv
            .with_untracked(|map| map.get(&(generation, tab)).cloned())
            .unwrap_or_default();
        assert!(stale.previewing && stale.saving && stale.deleting);
        assert!(stale.preview.is_none() && stale.import_result.is_none());
        assert!(matches!(
            store
                .cache
                .with_untracked(|map| map.get(&(generation, tab)).cloned()),
            Some(TabState::Loading)
        ));
    });
}

#[test]
fn delete_confirm_opens_closes_and_guards_double_confirm() {
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        session.user.set(Some(user("alice")));
        let tab = ReceiptsTab::Dividend;
        let store = test_store(&session, HashMap::new(), HashMap::new());

        store.open_delete_confirm(tab);
        assert!(store.csv_state(tab).show_delete_confirm);
        store.close_delete_confirm(tab);
        assert!(!store.csv_state(tab).show_delete_confirm);

        store.open_delete_confirm(tab);
        assert!(store.try_begin_delete(tab).is_some());
        let state = store.csv_state(tab);
        assert!(!state.show_delete_confirm);
        assert!(state.deleting);
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

        assert!(store.try_begin_save(tab).is_none());
        assert!(store.try_begin_delete(tab).is_none());
        store.open_delete_confirm(tab);
        assert!(!store.csv_state(tab).show_delete_confirm);
    });
}

#[test]
fn save_csv_requires_selected_file_and_idle_state() {
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        session.user.set(Some(user("alice")));
        let generation = session.generation.get_untracked();
        let tab = ReceiptsTab::Dividend;
        let store = test_store(&session, HashMap::new(), HashMap::new());

        assert!(store.try_begin_save(tab).is_none(), "ファイル未選択");
        assert!(!store.csv_state(tab).saving);

        store.csv.update(|map| {
            map.entry((generation, tab)).or_default().previewing = true;
        });
        assert!(store.try_begin_save(tab).is_none());
    });
}

#[test]
fn error_returns_list_fetch_errors_only() {
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        session.user.set(Some(user("alice")));
        let generation = session.generation.get_untracked();
        let tab = ReceiptsTab::Dividend;
        let csv_error = HashMap::from([(
            (generation, tab),
            CsvTabState {
                error: Some("リクエストが不正です".to_string()),
                ..Default::default()
            },
        )]);
        let store = test_store(
            &session,
            HashMap::from([(
                (generation, ReceiptsTab::DomesticStock),
                TabState::Failed("データ取得に失敗しました".to_string()),
            )]),
            csv_error,
        );

        assert_eq!(store.error().as_deref(), Some("データ取得に失敗しました"));

        let store = test_store(
            &session,
            HashMap::new(),
            HashMap::from([(
                (generation, tab),
                CsvTabState {
                    error: Some("リクエストが不正です".to_string()),
                    ..Default::default()
                },
            )]),
        );
        assert!(store.error().is_none());
    });
}

#[test]
fn rail_error_prioritizes_selected_tab() {
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        session.user.set(Some(user("alice")));
        let generation = session.generation.get_untracked();
        let selected = ReceiptsTab::MutualFund;
        let csv_error = || {
            HashMap::from([(
                (generation, selected),
                CsvTabState {
                    error: Some("CSVの保存に失敗しました".to_string()),
                    ..Default::default()
                },
            )])
        };

        let store = test_store(
            &session,
            HashMap::from([(
                (generation, ReceiptsTab::Dividend),
                TabState::Failed("配当の取得に失敗しました".to_string()),
            )]),
            csv_error(),
        );
        assert_eq!(
            store.rail_error(selected).as_deref(),
            Some("CSVの保存に失敗しました")
        );

        let store = test_store(
            &session,
            HashMap::from([(
                (generation, selected),
                TabState::Failed("投信の取得に失敗しました".to_string()),
            )]),
            csv_error(),
        );
        assert_eq!(
            store.rail_error(selected).as_deref(),
            Some("投信の取得に失敗しました")
        );

        let store = test_store(
            &session,
            HashMap::from([(
                (generation, ReceiptsTab::Dividend),
                TabState::Failed("配当の取得に失敗しました".to_string()),
            )]),
            HashMap::new(),
        );
        assert_eq!(
            store.rail_error(selected).as_deref(),
            Some("配当の取得に失敗しました")
        );
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
        assert!(store
            .cache
            .with_untracked(|map| !map.contains_key(&(generation, ReceiptsTab::MutualFund))));
    });
}

#[test]
fn mark_tab_for_refresh_marks_only_existing_entry() {
    let mut map = HashMap::new();
    assert!(!mark_tab_for_refresh(&mut map, 0, ReceiptsTab::Dividend));

    map.insert(
        (0, ReceiptsTab::Dividend),
        TabState::Ready(ReceiptTabData {
            rows: Vec::new(),
            summary: None,
            truncated: false,
        }),
    );
    assert!(mark_tab_for_refresh(&mut map, 0, ReceiptsTab::Dividend));
    assert!(matches!(
        map.get(&(0, ReceiptsTab::Dividend)),
        Some(TabState::Loading)
    ));
    assert!(!mark_tab_for_refresh(&mut map, 1, ReceiptsTab::Dividend));
    assert!(!mark_tab_for_refresh(&mut map, 0, ReceiptsTab::MutualFund));
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
        let state = store.csv_state(tab);
        assert!(!state.deleting);
        assert!(state.error.is_none());
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
        assert!(store.csv_state(tab).deleting);
    });
}

#[test]
fn tab_labels_and_api_paths_match_react() {
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
fn store_accessors_reflect_auth_and_tab_state() {
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        let generation = session.generation.get_untracked();
        let mut busy_state = CsvTabState::<CsvPreviewRow>::default();
        busy_state.begin_preview("a.csv".to_string());
        let store = test_store(
            &session,
            HashMap::from([((generation, ReceiptsTab::MutualFund), TabState::Loading)]),
            HashMap::from([((generation, ReceiptsTab::Dividend), busy_state)]),
        );

        assert!(!store.is_authenticated());
        session.user.set(Some(user("alice")));
        assert!(store.is_authenticated());
        assert!(store.auth_loading());
        session.loaded.set(true);
        assert!(!store.auth_loading());
        assert!(store.csv_busy(ReceiptsTab::Dividend));
        assert!(!store.csv_busy(ReceiptsTab::DomesticStock));
        assert!(store.any_tab_fetching());
        let settled_store = test_store(&session, HashMap::new(), HashMap::new());
        assert!(!settled_store.any_tab_fetching());
    });
}

#[test]
fn has_csv_preview_requires_non_empty_preview_rows() {
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        session.user.set(Some(user("alice")));
        let generation = session.generation.get_untracked();
        let tab = ReceiptsTab::Dividend;
        let store = test_store(
            &session,
            HashMap::new(),
            HashMap::from([
                (
                    (generation, tab),
                    CsvTabState {
                        preview: Some(CsvPreview {
                            rows: vec![crate::receipts_csv::CsvPreviewRow::Dividend(
                                Default::default(),
                            )],
                            ..Default::default()
                        }),
                        ..Default::default()
                    },
                ),
                (
                    (generation, ReceiptsTab::DomesticStock),
                    CsvTabState {
                        preview: Some(CsvPreview::default()),
                        ..Default::default()
                    },
                ),
            ]),
        );

        assert!(store.has_csv_preview(tab));
        assert!(!store.has_csv_preview(ReceiptsTab::DomesticStock));
        assert!(!store.has_csv_preview(ReceiptsTab::MutualFund));
    });
}

#[test]
fn tab_settled_only_for_ready_or_failed() {
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        let store = test_store(&session, HashMap::new(), HashMap::new());
        let tab = ReceiptsTab::Dividend;
        assert!(!tab_settled(&store, 0, tab));
        store.cache.update(|map| {
            map.insert((0, tab), TabState::Loading);
        });
        assert!(!tab_settled(&store, 0, tab));
        store.cache.update(|map| {
            map.insert((0, tab), TabState::Failed("x".to_string()));
        });
        assert!(tab_settled(&store, 0, tab));
        store.cache.update(|map| {
            map.insert(
                (0, tab),
                TabState::Ready(ReceiptTabData {
                    rows: Vec::new(),
                    summary: None,
                    truncated: false,
                }),
            );
        });
        assert!(tab_settled(&store, 0, tab));
    });
}

#[test]
fn ensure_prunes_only_stale_generation_entries() {
    let _ = any_spawner::Executor::init_futures_executor();
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        session.user.set(Some(user("alice")));
        let generation = session.generation.get_untracked();
        let stale = generation + 1;
        let tab = ReceiptsTab::Dividend;
        let store = test_store(
            &session,
            HashMap::from([
                (
                    (generation, tab),
                    TabState::Ready(ReceiptTabData {
                        rows: Vec::new(),
                        summary: None,
                        truncated: false,
                    }),
                ),
                ((stale, tab), TabState::Loading),
            ]),
            HashMap::from([((stale, tab), CsvTabState::default())]),
        );

        store.ensure(ReceiptsTab::DomesticStock);

        assert!(store
            .cache
            .with_untracked(|map| map.contains_key(&(generation, tab))));
        assert!(!store
            .cache
            .with_untracked(|map| map.keys().any(|(cached, _)| *cached == stale)));
        assert!(!store
            .csv
            .with_untracked(|map| map.keys().any(|(cached, _)| *cached == stale)));
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
        let stale = generation + 1;
        let tab = ReceiptsTab::Dividend;
        let store = test_store(
            &session,
            HashMap::from([(
                (generation, tab),
                TabState::Ready(ReceiptTabData {
                    rows: Vec::new(),
                    summary: None,
                    truncated: false,
                }),
            )]),
            HashMap::from([
                (
                    (generation, tab),
                    CsvTabState {
                        file_name: Some("a.csv".to_string()),
                        ..Default::default()
                    },
                ),
                ((stale, tab), CsvTabState::default()),
            ]),
        );

        store.ensure(tab);

        assert!(store
            .csv
            .with_untracked(|map| map.contains_key(&(generation, tab))));
        assert!(!store
            .csv
            .with_untracked(|map| map.keys().any(|(cached, _)| *cached == stale)));
    });
}

#[test]
fn stale_generation_helpers_detect_and_remove_foreign_generations() {
    let mut map = HashMap::from([
        ((0u64, ReceiptsTab::Dividend), 1),
        ((1, ReceiptsTab::Dividend), 2),
        ((1, ReceiptsTab::DomesticStock), 3),
    ]);
    assert!(has_stale_generation(&map, 1));
    assert!(has_stale_generation(&map, 0));
    prune_stale_generation(&mut map, 1);
    assert_eq!(map.len(), 2);
    assert!(map.contains_key(&(1, ReceiptsTab::Dividend)));
    assert!(map.contains_key(&(1, ReceiptsTab::DomesticStock)));
    assert!(!has_stale_generation(&map, 1));
    assert!(has_stale_generation(&map, 0));

    let mut current_only = HashMap::from([((1u64, ReceiptsTab::MutualFund), 4)]);
    assert!(!has_stale_generation(&current_only, 1));
    prune_stale_generation(&mut current_only, 1);
    assert_eq!(current_only.len(), 1);
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
            HashMap::from([(
                (generation, tab),
                CsvTabState {
                    file_name: Some("a.csv".to_string()),
                    ..Default::default()
                },
            )]),
        );

        store.ensure(tab);

        assert!(store
            .cache
            .with_untracked(|map| map.contains_key(&(generation, tab))));
        assert!(store
            .csv
            .with_untracked(|map| map.contains_key(&(generation, tab))));
    });
}
