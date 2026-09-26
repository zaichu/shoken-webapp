use super::csv::*;
use super::data::*;
use super::test_util::*;
use crate::api::ApiError;
use crate::asset_balance::csv::AssetBalanceCsvRow;
use crate::csv_flow::CsvTabState;
use crate::dividend_per_share::DividendMaps;
use crate::session::SessionStore;
use leptos::prelude::*;
use std::collections::HashMap;

#[test]
fn csv_state_is_scoped_to_session_generation() {
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        session.user.set(Some(user("alice")));
        let generation = session.generation.get_untracked();
        let store = csv_store(
            &session,
            RwSignal::new(None),
            RwSignal::new(DividendMaps::default()),
        );
        store.update_csv(generation, |state| {
            state.file_name = Some("asset.csv".to_string());
        });
        assert_eq!(store.csv_state().file_name.as_deref(), Some("asset.csv"));

        session.mark_unauthenticated();
        assert_eq!(
            store.csv_state(),
            CsvTabState::default(),
            "ログアウトでCSV状態は見えなくなる"
        );

        session.user.set(Some(user("bob")));
        assert_eq!(
            store.csv_state(),
            CsvTabState::default(),
            "別ユーザーでも見えない"
        );
    });
}

#[test]
fn stale_generation_csv_results_are_discarded() {
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        session.user.set(Some(user("alice")));
        let generation = session.generation.get_untracked();
        let store = csv_store(
            &session,
            RwSignal::new(None),
            RwSignal::new(DividendMaps::default()),
        );
        store.update_csv(generation, |state| {
            state.file_name = Some("asset.csv".to_string());
            state.previewing = true;
        });

        session.mark_unauthenticated();
        session.user.set(Some(user("bob")));

        assert!(store
            .apply_preview_result(
                generation,
                Ok(csv_preview_response(vec![csv_preview_row("7203")])),
            )
            .is_none());
        assert!(!store.apply_upload_result(generation, Ok(csv_upload_response(2))));
        store.apply_delete_result(generation, Ok(()));

        // 古い世代のスロットは進行中のまま残るだけで、現在世代からは見えない
        let slot = store.csv.get_untracked();
        assert!(matches!(
            slot,
            Some((cached, ref state))
                if cached == generation && state.previewing && state.preview.is_none()
        ));
        assert_eq!(store.csv_state(), CsvTabState::default());
    });
}

#[test]
fn preview_success_collects_security_codes_and_failure_shows_error() {
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        session.user.set(Some(user("alice")));
        let generation = session.generation.get_untracked();
        let store = csv_store(
            &session,
            RwSignal::new(None),
            RwSignal::new(DividendMaps::default()),
        );
        store.update_csv(generation, |state| {
            state.file_name = Some("asset.csv".to_string());
            state.previewing = true;
        });

        let codes = store.apply_preview_result(
            generation,
            Ok(csv_preview_response(vec![
                csv_preview_row("7203"),
                csv_preview_row("6758"),
                csv_preview_row("7203"),
            ])),
        );
        assert_eq!(
            codes,
            Some(vec!["6758".to_string(), "7203".to_string()]),
            "プレビュー行の銘柄コードが配当取得の対象になる"
        );
        let state = store.csv_state();
        assert!(!state.previewing);
        assert_eq!(
            state.preview.as_ref().map(|preview| preview.rows.len()),
            Some(3)
        );

        store.update_csv(generation, |state| {
            state.file_name = Some("asset.csv".to_string());
            state.previewing = true;
        });
        assert!(store
            .apply_preview_result(generation, Err(ApiError::http(500)))
            .is_none());
        let state = store.csv_state();
        assert!(!state.previewing);
        assert_eq!(
            state.error.as_deref(),
            Some("サーバーエラーが発生しました"),
            "資産管理はプレビュー失敗を画面に出す"
        );
        assert_eq!(state.file_name.as_deref(), Some("asset.csv"));
    });
}

#[test]
fn save_success_clears_file_and_preview_and_requests_reload() {
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        session.user.set(Some(user("alice")));
        let generation = session.generation.get_untracked();
        let store = csv_store(
            &session,
            RwSignal::new(None),
            RwSignal::new(DividendMaps::default()),
        );
        store.update_csv(generation, |state| {
            state.file_name = Some("asset.csv".to_string());
            state.preview = Some(crate::csv_flow::CsvPreview {
                valid_rows: 1,
                rows: vec![AssetBalanceCsvRow::default()],
                ..Default::default()
            });
            state.saving = true;
        });

        assert!(
            store.apply_upload_result(generation, Ok(csv_upload_response(2))),
            "成功時は一覧の再取得を要求する"
        );
        let state = store.csv_state();
        assert!(!state.saving);
        assert!(state.file_name.is_none());
        assert!(state.preview.is_none());
        assert_eq!(state.import_result.map(|result| result.inserted), Some(2));
        assert!(store.csv_file.get_untracked().is_none());
    });
}

#[test]
fn save_error_keeps_preview_and_sets_message() {
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        session.user.set(Some(user("alice")));
        let generation = session.generation.get_untracked();
        let store = csv_store(
            &session,
            RwSignal::new(None),
            RwSignal::new(DividendMaps::default()),
        );
        store.update_csv(generation, |state| {
            state.file_name = Some("asset.csv".to_string());
            state.preview = Some(crate::csv_flow::CsvPreview::default());
            state.saving = true;
        });

        assert!(!store.apply_upload_result(generation, Err(ApiError::http(401)),));
        let state = store.csv_state();
        assert!(!state.saving);
        assert_eq!(state.file_name.as_deref(), Some("asset.csv"));
        assert!(state.preview.is_some());
        assert_eq!(state.error.as_deref(), Some("認証が必要です"));
    });
}

#[test]
fn save_requires_preview_rows() {
    let state = CsvTabState::<AssetBalanceCsvRow>::default();
    assert!(!can_save_csv(&state));
    let empty = CsvTabState::<AssetBalanceCsvRow> {
        preview: Some(crate::csv_flow::CsvPreview::default()),
        ..Default::default()
    };
    assert!(!can_save_csv(&empty), "有効行0件のプレビューでは保存しない");
    let ready = CsvTabState::<AssetBalanceCsvRow> {
        preview: Some(crate::csv_flow::CsvPreview {
            rows: vec![AssetBalanceCsvRow::default()],
            ..Default::default()
        }),
        ..Default::default()
    };
    assert!(can_save_csv(&ready));
}

#[test]
fn delete_requires_open_confirm_and_rejects_double_start() {
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        session.user.set(Some(user("alice")));
        let store = csv_store(
            &session,
            RwSignal::new(None),
            RwSignal::new(DividendMaps::default()),
        );
        assert!(
            store.try_begin_delete().is_none(),
            "確認を表示していないと削除を開始しない"
        );

        store.open_delete_confirm();
        assert!(store.try_begin_delete().is_some());
        assert!(store.try_begin_delete().is_none(), "削除中は再開始しない");
        let state = store.csv_state();
        assert!(state.deleting);
        assert!(!state.show_delete_confirm);
    });
}

#[test]
fn delete_success_clears_result_and_empties_cached_list() {
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        session.user.set(Some(user("alice")));
        let generation = session.generation.get_untracked();
        let balances: RwSignal<BalanceSlot> = RwSignal::new(Some((
            generation,
            Ok(LoadedAssetBalances {
                rows: vec![balance_row(7203)],
                total: 1,
                summary: None,
                facets: None,
                truncated: false,
            }),
        )));
        let dividends: RwSignal<DividendMaps> = RwSignal::new(DividendMaps {
            per_share: HashMap::from([("7203".to_string(), 50.0)]),
            status: HashMap::new(),
        });
        let store = csv_store(&session, balances, dividends);
        store.update_csv(generation, |state| {
            state.import_result = Some(csv_upload_response(3));
            state.deleting = true;
        });

        store.apply_delete_result(generation, Ok(()));

        let state = store.csv_state();
        assert!(!state.deleting);
        assert!(state.import_result.is_none(), "削除成功で保存結果を消す");
        let loaded = balances
            .get_untracked()
            .and_then(|(cached, result)| (cached == generation).then_some(result))
            .and_then(|result| result.ok());
        assert_eq!(
            loaded.map(|loaded| loaded.rows.len()),
            Some(0),
            "キャッシュ済みの一覧は空で上書きされる"
        );
        assert!(dividends.get_untracked().per_share.is_empty());
    });
}

#[test]
fn delete_success_without_cached_list_sets_empty() {
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        session.user.set(Some(user("alice")));
        let generation = session.generation.get_untracked();
        let balances: RwSignal<BalanceSlot> = RwSignal::new(None);
        let store = csv_store(&session, balances, RwSignal::new(DividendMaps::default()));
        store.update_csv(generation, |state| state.deleting = true);

        store.apply_delete_result(generation, Ok(()));

        let loaded = balances
            .get_untracked()
            .and_then(|(cached, result)| (cached == generation).then_some(result))
            .and_then(|result| result.ok());
        assert_eq!(
            loaded.map(|loaded| (loaded.rows.len(), loaded.total)),
            Some((0, 0)),
            "一覧未取得でも削除成功は空一覧を確定させ、読み込み表示を残さない"
        );
    });
}

#[test]
fn csv_status_text_follows_busy_flags() {
    for (field, expected) in [
        ("saving", "データを保存しています..."),
        ("deleting", "データを削除しています..."),
        ("previewing", "CSVファイルを解析しています..."),
    ] {
        let mut state = CsvTabState::<AssetBalanceCsvRow>::default();
        match field {
            "saving" => state.saving = true,
            "deleting" => state.deleting = true,
            _ => state.previewing = true,
        }
        assert_eq!(csv_status_text(&state), Some(expected));
    }
    let state = CsvTabState::<AssetBalanceCsvRow>::default();
    assert_eq!(csv_status_text(&state), None);
}

#[test]
fn csv_store_accessors_reflect_session_and_slots() {
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        let generation = session.generation.get_untracked();
        let loaded = LoadedAssetBalances {
            total: 3,
            rows: vec![balance_row(7203)],
            summary: None,
            facets: None,
            truncated: false,
        };
        let balances = RwSignal::new(Some((generation, Ok(loaded))));
        let store = csv_store(&session, balances, RwSignal::new(DividendMaps::default()));

        assert!(!store.is_authenticated());
        session.user.set(Some(user("alice")));
        assert!(store.is_authenticated());
        assert_eq!(store.db_count(), 3);
        assert!(!store.csv_busy());

        store.update_csv(generation, |state| {
            state.begin_preview("a.csv".to_string());
        });
        assert!(store.csv_busy());
    });
}

#[test]
fn db_count_ignores_stale_generation_balances() {
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        let stale = session.generation.get_untracked() + 1;
        let loaded = LoadedAssetBalances {
            total: 5,
            rows: vec![],
            summary: None,
            facets: None,
            truncated: false,
        };
        let balances = RwSignal::new(Some((stale, Ok(loaded))));
        let store = csv_store(&session, balances, RwSignal::new(DividendMaps::default()));
        session.user.set(Some(user("alice")));
        assert_eq!(store.db_count(), 0);
    });
}

#[test]
fn delete_confirm_flow_updates_csv_state() {
    let _ = any_spawner::Executor::init_futures_executor();
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        session.user.set(Some(user("alice")));
        let store = csv_store(
            &session,
            RwSignal::new(None),
            RwSignal::new(DividendMaps::default()),
        );

        store.open_delete_confirm();
        assert!(store.csv_state().show_delete_confirm);
        store.close_delete_confirm();
        assert!(!store.csv_state().show_delete_confirm);

        store.open_delete_confirm();
        store.confirm_delete_all();
        assert!(store.csv_state().deleting);
    });
}
