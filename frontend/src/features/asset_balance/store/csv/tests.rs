use super::*;
use crate::features::asset_balance::lookup::AssetBalanceLookupStore;
use crate::features::asset_balance::store::DataOps;
use crate::features::dividend_per_share::DividendMaps;
use crate::session::SessionStore;
use crate::support::csv_flow::CsvTabMeta;
use crate::testing::asset_balance::*;
use std::collections::HashMap;

fn store_with(session: &SessionStore, balances: RwSignal<BalanceSlot>) -> AssetBalanceCsvStore {
    AssetBalanceCsvStore::new(
        *session,
        balances,
        RwSignal::new(None),
        RwSignal::new(DividendMaps::default()),
        RwSignal::new(AssetBalanceLookupStore::new()),
        RwSignal::new(DataOps::default()),
    )
}

#[test]
fn csv_meta_returns_default_without_user() {
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        let store = store_with(&session, RwSignal::new(None));

        assert_eq!(store.csv_meta(), CsvTabMeta::default());
    });
}

#[test]
fn csv_meta_reflects_delete_confirm() {
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        session.user.set(Some(user("alice")));
        let store = store_with(&session, RwSignal::new(None));

        store.open_delete_confirm();
        let meta = store.csv_meta();
        assert!(meta.show_delete_confirm);
        assert!(!meta.deleting);
        assert!(meta.error.is_none());
    });
}

#[test]
fn csv_meta_is_scoped_to_session_generation() {
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        session.user.set(Some(user("alice")));
        let store = store_with(&session, RwSignal::new(None));

        store.open_delete_confirm();
        assert!(store.csv_meta().show_delete_confirm);

        session.mark_unauthenticated();
        assert_eq!(
            store.csv_meta(),
            CsvTabMeta::default(),
            "ログアウトでCSV状態は見えなくなる"
        );

        session.user.set(Some(user("bob")));
        assert_eq!(
            store.csv_meta(),
            CsvTabMeta::default(),
            "別ユーザーでも見えない"
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
        let store = store_with(&session, RwSignal::new(None));
        store.open_delete_confirm();
        assert!(store.csv_meta().show_delete_confirm);

        session.mark_unauthenticated();
        store.apply_delete_result(generation, Ok(()));

        assert_eq!(
            store.csv_meta(),
            CsvTabMeta::default(),
            "古い世代の遅れ結果は現在世代に影響しない"
        );
    });
}

#[test]
fn delete_requires_open_confirm_and_rejects_double_start() {
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        session.user.set(Some(user("alice")));
        let store = store_with(&session, RwSignal::new(None));

        assert!(
            store.try_begin_delete().is_none(),
            "確認を表示していないと削除を開始しない"
        );

        store.open_delete_confirm();
        assert!(store.try_begin_delete().is_some());
        assert!(store.try_begin_delete().is_none(), "削除中は再開始しない");
        let meta = store.csv_meta();
        assert!(meta.deleting);
        assert!(!meta.show_delete_confirm);
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
                rows: vec![balance_row(9904)],
                total: 1,
                summary: None,
                truncated: false,
            }),
        )));
        let dividends: RwSignal<DividendMaps> = RwSignal::new(DividendMaps {
            per_share: HashMap::from([("9904".to_string(), 50.0)]),
            status: HashMap::new(),
        });
        let store = AssetBalanceCsvStore::new(
            session,
            balances,
            RwSignal::new(None),
            dividends,
            RwSignal::new(AssetBalanceLookupStore::new()),
            RwSignal::new(DataOps::default()),
        );
        store.update_csv(generation, |state| state.deleting = true);

        store.apply_delete_result(generation, Ok(()));

        let meta = store.csv_meta();
        assert!(!meta.deleting);
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
        let store = store_with(&session, balances);
        store.update_csv(generation, |state| state.deleting = true);

        store.apply_delete_result(generation, Ok(()));

        assert!(!store.csv_meta().deleting);
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
fn saved_count_ready_waits_for_inflight_refresh() {
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        session.user.set(Some(user("alice")));
        let generation = session.generation.get_untracked();
        let loaded = LoadedAssetBalances {
            total: 1,
            rows: vec![balance_row(9904)],
            summary: None,
            truncated: false,
        };
        let balances = RwSignal::new(Some((generation, Ok(loaded))));
        let store = store_with(&session, balances);
        assert!(store.saved_count_ready());
        store.data_ops.update(DataOps::begin_list_fetch);
        assert!(
            !store.saved_count_ready(),
            "取り直し中はキャッシュの古い件数を未確定として扱う"
        );

        // 取り直しが失敗しても古い件数を確定扱いに戻さない
        store
            .data_ops
            .update(|ops| ops.end_list_fetch(ops.list_rev));
        store
            .data_ops
            .update(|ops| ops.refresh_error = Some("error".to_string()));
        assert!(!store.saved_count_ready());
        assert_eq!(store.saved_count_error().as_deref(), Some("error"));
    });
}

#[test]
fn db_count_ignores_stale_generation_balances() {
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        let stale = session.generation.get_untracked().next();
        let loaded = LoadedAssetBalances {
            total: 5,
            rows: vec![],
            summary: None,
            truncated: false,
        };
        let balances = RwSignal::new(Some((stale, Ok(loaded))));
        let store = store_with(&session, balances);
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
        let store = store_with(&session, RwSignal::new(None));

        store.open_delete_confirm();
        assert!(store.csv_meta().show_delete_confirm);

        store.confirm_delete_all();
        assert!(store.csv_meta().deleting);
    });
}
