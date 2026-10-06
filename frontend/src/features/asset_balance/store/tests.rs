use super::*;
use crate::api::ApiError;
use crate::api::dto::{AssetBalanceListResponse, SearchFacets};
use crate::features::asset_balance::lookup::AssetBalanceLookupStore;
use crate::features::asset_balance::store::DataOps;
use crate::features::asset_balance::store::csv::AssetBalanceCsvStore;
use crate::features::dividend_per_share::DividendMaps;
use crate::session::SessionStore;
use crate::support::pagination::collect_list_pages;
use crate::testing::asset_balance::*;
use crate::testing::block_on;
use std::collections::HashMap;
use std::future::{Ready, ready};

fn first_page_with_facets() -> AssetBalanceListResponse {
    let mut first = balance_page(0..1000, 2300, Some(rust_decimal_macros::dec!(10)));
    first.facets = Some(SearchFacets {
        securities: Some(vec![crate::api::dto::FacetOption {
            value: "7203".to_string(),
            label: "トヨタ自動車".to_string(),
            count: None,
        }]),
        ..SearchFacets::default()
    });
    first
}

#[test]
fn list_pages_join_all_pages_in_order() {
    let mut second = balance_page(1000..2000, 2300, Some(rust_decimal_macros::dec!(20)));
    second.facets = Some(SearchFacets::default());
    let third = balance_page(2000..2300, 2300, None);
    let fetch = move |page_no: usize| -> Ready<Result<AssetBalanceListResponse, ApiError>> {
        ready(Ok(match page_no {
            1 => first_page_with_facets(),
            2 => second.clone(),
            _ => third.clone(),
        }))
    };

    let page = block_on(collect_list_pages(LIST_PER_PAGE, LIST_MAX_PAGES, fetch)).expect("fetch");

    assert_eq!(page.rows.len(), 2300);
    assert_eq!(page.total, Some(2300));
    assert_eq!(page.rows[0].id.as_str(), "id-0");
    assert_eq!(page.rows[2299].id.as_str(), "id-2299");
    // summary・facets は1ページ目のものだけを採用し、以降のページのものは捨てる
    assert_eq!(
        page.summary.map(|summary| summary.total_purchase_amount),
        Some(rust_decimal_macros::dec!(10))
    );
    assert_eq!(
        page.facets
            .and_then(|facets| facets.securities)
            .map(|securities| securities.len()),
        Some(1)
    );
}

#[test]
fn list_pages_stop_when_first_page_is_short() {
    let fetch = |_: usize| {
        ready(Ok::<_, ApiError>(balance_page(
            0..3,
            3,
            Some(rust_decimal_macros::dec!(10)),
        )))
    };
    let page = block_on(collect_list_pages(LIST_PER_PAGE, LIST_MAX_PAGES, fetch)).expect("fetch");
    assert_eq!(page.rows.len(), 3);
    assert_eq!(page.total, Some(3));
    assert!(page.summary.is_some());
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
        assert_eq!(loaded.rows[0].id.as_str(), "id-7203");
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
        let generation = Generation::new(1);
        let balances: RwSignal<BalanceSlot> = RwSignal::new(None);
        let dividends: RwSignal<DividendMaps> = RwSignal::new(DividendMaps::default());
        let lookup = RwSignal::new(AssetBalanceLookupStore::new());
        lookup.update(|store| store.seed(generation, &[balance_row(7203), balance_row(6758)]));

        let mut replaced = balance_row(7203);
        replaced.shares = rust_decimal_macros::dec!(200);
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
        let generation = Generation::new(1);
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
        let generation = Generation::new(1);
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
        let generation = Generation::new(1);
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
        let generation = Generation::new(1);
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

        assert!(
            !store
                .data_ops
                .with_untracked(|ops| ops.is_current_list(list_rev))
        );
        assert!(
            !store
                .data_ops
                .with_untracked(|ops| ops.is_current_poll(poll_rev))
        );
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

        assert!(
            !store
                .data_ops
                .with_untracked(|ops| ops.is_current_poll(old_poll_rev))
        );
        assert!(dividends.get_untracked().per_share.is_empty());
    });
}

#[test]
fn has_current_balances_only_matches_same_generation() {
    let generation = Generation::new(1);
    let other = Generation::new(2);
    let slot: BalanceSlot = Some((
        generation,
        Ok(LoadedAssetBalances {
            rows: vec![balance_row(7203)],
            total: 1,
            summary: None,
            facets: None,
            truncated: false,
        }),
    ));
    assert!(has_current_balances(&slot, generation));
    assert!(!has_current_balances(&slot, other));
    assert!(!has_current_balances(&None, generation));
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

#[test]
fn csv_state_returns_none_when_no_user() {
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        let balances: RwSignal<BalanceSlot> = RwSignal::new(None);
        let dividends: RwSignal<DividendMaps> = RwSignal::new(DividendMaps::default());
        let lookup = RwSignal::new(AssetBalanceLookupStore::new());
        let _csv: RwSignal<AssetCsvSlot> = RwSignal::new(None);
        let data_ops: RwSignal<DataOps> = RwSignal::new(DataOps::default());

        let store = AssetBalanceCsvStore::new(session, balances, dividends, lookup, data_ops);

        assert_eq!(store.csv_state(), CsvTabState::default());
    });
}

#[test]
fn csv_state_returns_value_when_user_exists() {
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        let generation = session.generation.get_untracked();
        let balances: RwSignal<BalanceSlot> = RwSignal::new(None);
        let dividends: RwSignal<DividendMaps> = RwSignal::new(DividendMaps::default());
        let lookup = RwSignal::new(AssetBalanceLookupStore::new());
        let _csv: RwSignal<AssetCsvSlot> = RwSignal::new(None);
        let data_ops: RwSignal<DataOps> = RwSignal::new(DataOps::default());

        let store = AssetBalanceCsvStore::new(session, balances, dividends, lookup, data_ops);

        session.user.set(Some(user("alice")));
        store.update_csv(generation, |state| {
            state.file_name = Some("asset.csv".to_string());
        });
        assert_eq!(store.csv_state().file_name.as_deref(), Some("asset.csv"));
    });
}
