use super::data::*;
use super::test_util::*;
use crate::asset_balance_lookup::AssetBalanceLookupStore;
use crate::dividend_per_share::DividendMaps;
use crate::dto::SearchFacets;
use crate::session::SessionStore;
use leptos::prelude::*;
use std::collections::HashMap;

#[test]
fn asset_balance_pages_join_all_pages_in_order() {
    let mut pages = AssetBalancePages::new();
    assert_eq!(pages.next_page(), 1);
    let mut first = balance_page(0..1000, 2300, Some(rust_decimal_macros::dec!(10)));
    first.facets = Some(SearchFacets {
        securities: Some(vec![crate::dto::FacetOption {
            value: "7203".to_string(),
            label: "トヨタ自動車".to_string(),
            count: None,
        }]),
        ..SearchFacets::default()
    });
    assert!(pages.push(first));
    assert_eq!(pages.next_page(), 2);
    let mut second = balance_page(1000..2000, 2300, Some(rust_decimal_macros::dec!(20)));
    second.facets = Some(SearchFacets::default());
    assert!(pages.push(second));
    assert_eq!(pages.next_page(), 3);
    assert!(!pages.push(balance_page(2000..2300, 2300, None)));

    let loaded = pages.finish();
    assert_eq!(loaded.rows.len(), 2300);
    assert_eq!(loaded.total, 2300);
    assert_eq!(loaded.rows[0].id, "id-0");
    assert_eq!(loaded.rows[2299].id, "id-2299");
    // summary・facets は1ページ目のものだけを採用し、以降のページのものは捨てる
    assert_eq!(
        loaded.summary.map(|summary| summary.total_purchase_amount),
        Some(rust_decimal_macros::dec!(10))
    );
    assert_eq!(
        loaded
            .facets
            .and_then(|facets| facets.securities)
            .map(|securities| securities.len()),
        Some(1)
    );
}

#[test]
fn asset_balance_pages_stop_when_first_page_is_short() {
    let mut pages = AssetBalancePages::new();
    assert!(!pages.push(balance_page(0..3, 3, Some(rust_decimal_macros::dec!(10)))));
    let loaded = pages.finish();
    assert_eq!(loaded.rows.len(), 3);
    assert_eq!(loaded.total, 3);
    assert!(loaded.summary.is_some());
}

#[test]
fn apply_loaded_asset_balances_replaces_same_generation_list() {
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        session.user.set(Some(user("alice")));
        let generation = session.generation.get_untracked();
        let balances: RwSignal<BalanceSlot> = RwSignal::new(Some((
            generation,
            Ok(LoadedAssetBalances {
                rows: vec![balance_row(6758)],
                total: 1,
                summary: None,
                facets: None,
                truncated: false,
            }),
        )));
        let dividends: RwSignal<DividendMaps> = RwSignal::new(DividendMaps {
            per_share: HashMap::from([("6758".to_string(), 40.0)]),
            status: HashMap::new(),
        });
        let lookup = RwSignal::new(AssetBalanceLookupStore::new());
        lookup.update(|store| store.seed(generation, &[balance_row(6758)]));

        let new_row = balance_row(7203);
        let (codes, missing) = apply_loaded_asset_balances(
            generation,
            LoadedAssetBalances {
                rows: vec![new_row],
                total: 1,
                summary: None,
                facets: None,
                truncated: false,
            },
            balances,
            dividends,
            lookup,
            false,
        );

        let loaded = balances
            .get_untracked()
            .and_then(|(cached, result)| (cached == generation).then_some(result))
            .and_then(|result| result.ok())
            .expect("loaded");
        assert_eq!(loaded.rows.len(), 1);
        assert_eq!(loaded.rows[0].id, "id-7203");
        assert!(dividends.get_untracked().per_share.is_empty());
        assert_eq!(codes, vec!["7203".to_string()]);
        assert!(
            missing.is_empty(),
            "一覧行を seed 済みなので個別再取得の対象はない"
        );
    });
}

#[test]
fn apply_loaded_replaces_same_generation_lookup_values() {
    let owner = Owner::new();
    owner.with(|| {
        let generation = 1;
        let balances: RwSignal<BalanceSlot> = RwSignal::new(None);
        let dividends: RwSignal<DividendMaps> = RwSignal::new(DividendMaps::default());
        let lookup = RwSignal::new(AssetBalanceLookupStore::new());
        lookup.update(|store| store.seed(generation, &[balance_row(7203), balance_row(6758)]));

        let mut replaced = balance_row(7203);
        replaced.shares = rust_decimal_macros::dec!(200);
        replaced.market_value = rust_decimal_macros::dec!(520000);
        apply_loaded_asset_balances(
            generation,
            LoadedAssetBalances {
                rows: vec![replaced],
                total: 1,
                summary: None,
                facets: None,
                truncated: false,
            },
            balances,
            dividends,
            lookup,
            false,
        );

        let shares =
            lookup.with_untracked(|store| store.get(generation, "7203").map(|row| row.shares));
        assert_eq!(
            shares,
            Some(rust_decimal_macros::dec!(200)),
            "同一世代の置換でも lookup の古い値を残さない"
        );
        assert!(
            lookup.with_untracked(|store| store.get(generation, "6758").is_none()),
            "置換後に消えた銘柄の lookup も消す"
        );
    });
}

#[test]
fn apply_loaded_keeps_dividends_while_preview_active() {
    let owner = Owner::new();
    owner.with(|| {
        let generation = 1;
        let balances: RwSignal<BalanceSlot> = RwSignal::new(None);
        let dividends: RwSignal<DividendMaps> = RwSignal::new(DividendMaps {
            per_share: HashMap::from([("7203".to_string(), 50.0)]),
            status: HashMap::new(),
        });
        let lookup = RwSignal::new(AssetBalanceLookupStore::new());

        apply_loaded_asset_balances(
            generation,
            LoadedAssetBalances {
                rows: vec![balance_row(7203)],
                total: 1,
                summary: None,
                facets: None,
                truncated: false,
            },
            balances,
            dividends,
            lookup,
            true,
        );

        assert_eq!(
            dividends.get_untracked().per_share.get("7203"),
            Some(&50.0),
            "プレビュー表示中の一覧適用ではプレビュー行の配当を消さない"
        );
        assert!(matches!(
            balances.get_untracked(),
            Some((cached, Ok(_))) if cached == generation
        ));
    });
}

#[test]
fn list_error_keeps_cached_rows_and_reports_refresh_error() {
    let owner = Owner::new();
    owner.with(|| {
        let generation = 1;
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
        let data_ops: RwSignal<DataOps> = RwSignal::new(DataOps::default());
        data_ops.update(DataOps::begin_list_fetch);
        let rev = data_ops.with_untracked(|ops| ops.list_rev);
        data_ops.update(|ops| ops.end_list_fetch(rev));

        apply_list_error(
            generation,
            rev,
            "サーバーエラーが発生しました".to_string(),
            balances,
            data_ops,
        );

        let loaded = balances
            .get_untracked()
            .and_then(|(cached, result)| (cached == generation).then_some(result))
            .and_then(|result| result.ok());
        assert_eq!(
            loaded.map(|loaded| loaded.rows.len()),
            Some(1),
            "再取得失敗でも表示中の一覧を消さない"
        );
        assert_eq!(
            data_ops.with_untracked(|ops| ops.refresh_error.clone()),
            Some("サーバーエラーが発生しました".to_string())
        );
    });
}

#[test]
fn list_error_without_cache_sets_error_slot() {
    let owner = Owner::new();
    owner.with(|| {
        let generation = 1;
        let balances: RwSignal<BalanceSlot> = RwSignal::new(None);
        let data_ops: RwSignal<DataOps> = RwSignal::new(DataOps::default());
        data_ops.update(DataOps::begin_list_fetch);
        let rev = data_ops.with_untracked(|ops| ops.list_rev);
        data_ops.update(|ops| ops.end_list_fetch(rev));

        apply_list_error(
            generation,
            rev,
            "認証が必要です".to_string(),
            balances,
            data_ops,
        );

        assert!(matches!(
            balances.get_untracked(),
            Some((cached, Err(_))) if cached == generation
        ));
        assert!(data_ops.with_untracked(|ops| ops.refresh_error.is_none()));
    });
}

#[test]
fn list_error_from_stale_fetch_is_dropped() {
    let owner = Owner::new();
    owner.with(|| {
        let generation = 1;
        let balances: RwSignal<BalanceSlot> = RwSignal::new(None);
        let data_ops: RwSignal<DataOps> = RwSignal::new(DataOps::default());
        data_ops.update(DataOps::begin_list_fetch);
        let stale_rev = data_ops.with_untracked(|ops| ops.list_rev);
        data_ops.update(DataOps::begin_list_fetch);

        apply_list_error(
            generation,
            stale_rev,
            "サーバーエラーが発生しました".to_string(),
            balances,
            data_ops,
        );

        assert!(balances.get_untracked().is_none());
        assert!(data_ops.with_untracked(|ops| ops.refresh_error.is_none()));
    });
}

#[test]
fn list_loading_counts_inflight_fetch() {
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
        let store = csv_store(&session, balances, RwSignal::new(DividendMaps::default()));

        assert!(
            !store.list_loading(),
            "キャッシュ済みで取得中でなければ false"
        );

        store.data_ops.update(DataOps::begin_list_fetch);
        let rev = store.data_ops.with_untracked(|ops| ops.list_rev);
        assert!(store.list_loading(), "キャッシュがあっても再取得中は true");

        store.data_ops.update(|ops| ops.end_list_fetch(rev));
        assert!(!store.list_loading());
    });
}

#[test]
fn begin_delete_invalidates_inflight_list_result() {
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        session.user.set(Some(user("alice")));
        let store = csv_store(
            &session,
            RwSignal::new(None),
            RwSignal::new(DividendMaps::default()),
        );
        store.data_ops.update(DataOps::begin_list_fetch);
        let inflight_rev = store.data_ops.with_untracked(|ops| ops.list_rev);

        store.open_delete_confirm();
        assert!(store.try_begin_delete().is_some());
        assert!(
            !store
                .data_ops
                .with_untracked(|ops| ops.is_current_list(inflight_rev)),
            "削除開始で進行中の一覧取得結果は適用されなくなる"
        );
    });
}

#[test]
fn delete_success_invalidates_late_list_and_poll_results() {
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
        store.data_ops.update(DataOps::begin_list_fetch);
        let list_rev = store.data_ops.with_untracked(|ops| ops.list_rev);
        store.data_ops.update(DataOps::next_poll_rev);
        let poll_rev = store.data_ops.with_untracked(|ops| ops.poll_rev);
        store.update_csv(generation, |state| state.deleting = true);

        store.apply_delete_result(generation, Ok(()));

        assert!(!store
            .data_ops
            .with_untracked(|ops| ops.is_current_list(list_rev)));
        assert!(!store
            .data_ops
            .with_untracked(|ops| ops.is_current_poll(poll_rev)));
    });
}

#[test]
fn poll_rev_supersedes_earlier_poll() {
    let owner = Owner::new();
    owner.with(|| {
        let data_ops: RwSignal<DataOps> = RwSignal::new(DataOps::default());
        data_ops.update(DataOps::next_poll_rev);
        let first = data_ops.with_untracked(|ops| ops.poll_rev);
        data_ops.update(DataOps::next_poll_rev);
        let second = data_ops.with_untracked(|ops| ops.poll_rev);
        assert!(!data_ops.with_untracked(|ops| ops.is_current_poll(first)));
        assert!(data_ops.with_untracked(|ops| ops.is_current_poll(second)));
    });
}

#[test]
fn stale_list_completion_does_not_clear_inflight_of_current_generation() {
    let owner = Owner::new();
    owner.with(|| {
        let data_ops: RwSignal<DataOps> = RwSignal::new(DataOps::default());
        data_ops.update(DataOps::begin_list_fetch);
        let stale_rev = data_ops.with_untracked(|ops| ops.list_rev);
        // ユーザー切替相当のリセット後、旧世代の完了が新世代の取得中を消さない
        data_ops.update(DataOps::reset);
        data_ops.update(DataOps::begin_list_fetch);
        let current_rev = data_ops.with_untracked(|ops| ops.list_rev);

        data_ops.update(|ops| ops.end_list_fetch(stale_rev));
        assert!(data_ops.with_untracked(|ops| !ops.inflight.is_empty()));

        data_ops.update(|ops| ops.end_list_fetch(current_rev));
        assert!(data_ops.with_untracked(|ops| ops.inflight.is_empty()));
    });
}

#[test]
fn begin_file_preview_stops_old_poll_and_clears_dividends() {
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        session.user.set(Some(user("alice")));
        let generation = session.generation.get_untracked();
        let dividends = RwSignal::new(DividendMaps {
            per_share: HashMap::from([("7203".to_string(), 50.0)]),
            status: HashMap::new(),
        });
        let store = csv_store(&session, RwSignal::new(None), dividends);
        store.data_ops.update(DataOps::next_poll_rev);
        let old_poll_rev = store.data_ops.with_untracked(|ops| ops.poll_rev);

        assert!(store.begin_file_preview(generation, "next.csv".to_string()));

        assert!(!store
            .data_ops
            .with_untracked(|ops| ops.is_current_poll(old_poll_rev)));
        assert!(dividends.get_untracked().per_share.is_empty());
    });
}

#[test]
fn data_ops_reset_clears_inflight_and_error() {
    let mut ops = DataOps {
        list_rev: 2,
        poll_rev: 3,
        inflight: std::collections::HashSet::from([2]),
        refresh_error: Some("x".to_string()),
    };
    ops.reset();
    assert_eq!(ops.list_rev, 3);
    assert_eq!(ops.poll_rev, 4);
    assert!(ops.inflight.is_empty());
    assert!(ops.refresh_error.is_none());
}
