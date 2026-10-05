#[path = "csv.rs"]
mod csv;
#[path = "fetch.rs"]
mod fetch;
#[path = "search.rs"]
mod search;

use super::store::{bump_fetch_rev, is_current_fetch, settle_tab_result};
use super::*;
use crate::api::ApiError;
use crate::features::receipts::filter::ReceiptSearch;
use crate::session::{Generation, SessionStore};
use crate::support::row::Row;
use leptos::prelude::*;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

fn user(id: &str) -> crate::api::dto::SessionUser {
    crate::api::dto::SessionUser {
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

            assert!(!session.is_current(fetch_generation));
        }
    });
}

#[test]
fn unauthorized_error_uses_401_message() {
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
        let fetch = Action::new_unsync(|_: &(Generation, ReceiptsTab)| async {});

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
                utility_rail_open: RwSignal::new(true),
                expanded_epoch: RwSignal::new(None),
                visited: RwSignal::new(HashSet::from([tab])),
                cache,
                fetch,
                csv: RwSignal::new(HashMap::new()),
                csv_files: RwSignal::new(HashMap::new()),
                refresh_error: RwSignal::new(HashMap::new()),
                fetch_rev: RwSignal::new(HashMap::new()),
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
fn revisit_refetches_only_visited_settled_tabs() {
    let _ = any_spawner::Executor::init_futures_executor();
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
        let ready = TabState::Ready(ReceiptTabData {
            rows: Vec::new(),
            summary: None,
            truncated: false,
        });
        let store = ReceiptsStore {
            session,
            active_tab: RwSignal::new(ReceiptsTab::Dividend),
            search: RwSignal::new(ReceiptSearch::default()),
            expanded: RwSignal::new(HashSet::new()),
            mobile_summary_expanded: RwSignal::new(false),
            utility_rail_open: RwSignal::new(true),
            expanded_epoch: RwSignal::new(None),
            visited: RwSignal::new(HashSet::from([
                ReceiptsTab::Dividend,
                ReceiptsTab::DomesticStock,
                ReceiptsTab::MutualFund,
            ])),
            cache: RwSignal::new(HashMap::from([
                ((generation, ReceiptsTab::Dividend), ready),
                ((generation, ReceiptsTab::DomesticStock), TabState::Loading),
                (
                    (generation, ReceiptsTab::MutualFund),
                    TabState::Failed("x".to_string()),
                ),
            ])),
            fetch,
            csv: RwSignal::new(HashMap::new()),
            csv_files: RwSignal::new(HashMap::new()),
            refresh_error: RwSignal::new(HashMap::new()),
            fetch_rev: RwSignal::new(HashMap::new()),
        };

        store.revisit();

        let dispatched = calls.borrow().clone();
        assert!(
            dispatched.contains(&(generation, ReceiptsTab::Dividend)),
            "取得済みの訪問タブは裏で取り直す"
        );
        assert!(
            dispatched.contains(&(generation, ReceiptsTab::MutualFund)),
            "失敗済みも再取得の対象"
        );
        assert!(
            !dispatched.contains(&(generation, ReceiptsTab::DomesticStock)),
            "取得中のタブは重複させない"
        );
    });
}

#[test]
fn revisit_skips_unauthenticated_and_unvisited() {
    let _ = any_spawner::Executor::init_futures_executor();
    let owner = Owner::new();
    owner.with(|| {
        let calls = Rc::new(RefCell::new(Vec::new()));
        let record = Rc::clone(&calls);
        let fetch = Action::new_unsync(move |(generation, tab): &(Generation, ReceiptsTab)| {
            record.borrow_mut().push((*generation, *tab));
            async {}
        });
        let session = SessionStore::new();
        let store = ReceiptsStore {
            session,
            active_tab: RwSignal::new(ReceiptsTab::Dividend),
            search: RwSignal::new(ReceiptSearch::default()),
            expanded: RwSignal::new(HashSet::new()),
            mobile_summary_expanded: RwSignal::new(false),
            utility_rail_open: RwSignal::new(true),
            expanded_epoch: RwSignal::new(None),
            visited: RwSignal::new(HashSet::from([ReceiptsTab::Dividend])),
            cache: RwSignal::new(HashMap::new()),
            fetch,
            csv: RwSignal::new(HashMap::new()),
            csv_files: RwSignal::new(HashMap::new()),
            refresh_error: RwSignal::new(HashMap::new()),
            fetch_rev: RwSignal::new(HashMap::new()),
        };

        store.revisit();
        assert!(calls.borrow().is_empty(), "未ログインでは何も取り直さない");

        session.user.set(Some(user("alice")));
        let generation = session.generation.get_untracked();
        let ready = || {
            TabState::Ready(ReceiptTabData {
                rows: Vec::new(),
                summary: None,
                truncated: false,
            })
        };
        store.cache.set(HashMap::from([
            ((generation, ReceiptsTab::Dividend), ready()),
            ((generation, ReceiptsTab::MutualFund), ready()),
        ]));
        store.revisit();
        assert_eq!(
            calls.borrow().clone(),
            vec![(generation, ReceiptsTab::Dividend)],
            "取得中でない訪問済みタブだけを取り直す(未訪問の MutualFund は触らない)"
        );
    });
}

#[test]
fn settle_tab_result_keeps_ready_on_refresh_failure() {
    // Ok は常に Ready で書く
    assert!(matches!(
        settle_tab_result(
            true,
            Ok(ReceiptTabData {
                rows: Vec::new(),
                summary: None,
                truncated: false,
            })
        ),
        Ok(TabState::Ready(_))
    ));
    // 表示済みがある裏再取得の失敗は一覧を消さずエラーを返す
    assert_eq!(
        settle_tab_result(true, Err(ApiError::http(500))),
        Err(ApiError::http(500).message())
    );
    // 初回取得(表示済みなし)の失敗は従来どおり Failed にする
    assert!(matches!(
        settle_tab_result(false, Err(ApiError::http(500))),
        Ok(TabState::Failed(_))
    ));
}

#[test]
fn delete_success_expires_inflight_revisit_fetch() {
    let mut fetch_rev: HashMap<(Generation, ReceiptsTab), u64> = HashMap::new();
    let generation = Generation::new(1);
    let tab = ReceiptsTab::Dividend;
    // 再訪で出した GET(rev=1)がまだ応答を返していない状態を再現する
    bump_fetch_rev(&mut fetch_rev, generation, tab);
    let inflight_rev = 1;
    assert!(is_current_fetch(&fetch_rev, generation, tab, inflight_rev));
    // 全件削除が成功すると取得が失効する
    bump_fetch_rev(&mut fetch_rev, generation, tab);
    assert!(!is_current_fetch(&fetch_rev, generation, tab, inflight_rev));
    assert!(is_current_fetch(&fetch_rev, generation, tab, 2));
}

#[test]
fn revisit_does_not_dispatch_while_fetch_pending() {
    let _ = any_spawner::Executor::init_futures_executor();
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        session.user.set(Some(user("alice")));
        let generation = session.generation.get_untracked();
        let calls = Rc::new(RefCell::new(Vec::new()));
        let record = Rc::clone(&calls);
        // 応答が返らないままの取得を再現する
        let fetch = Action::new_unsync(move |(generation, tab): &(Generation, ReceiptsTab)| {
            record.borrow_mut().push((*generation, *tab));
            std::future::pending()
        });
        let ready = || {
            TabState::Ready(ReceiptTabData {
                rows: Vec::new(),
                summary: None,
                truncated: false,
            })
        };
        let store = ReceiptsStore {
            session,
            active_tab: RwSignal::new(ReceiptsTab::Dividend),
            search: RwSignal::new(ReceiptSearch::default()),
            expanded: RwSignal::new(HashSet::new()),
            mobile_summary_expanded: RwSignal::new(false),
            utility_rail_open: RwSignal::new(true),
            expanded_epoch: RwSignal::new(None),
            visited: RwSignal::new(HashSet::from([
                ReceiptsTab::Dividend,
                ReceiptsTab::MutualFund,
            ])),
            cache: RwSignal::new(HashMap::from([
                ((generation, ReceiptsTab::Dividend), ready()),
                ((generation, ReceiptsTab::MutualFund), ready()),
            ])),
            fetch,
            csv: RwSignal::new(HashMap::new()),
            csv_files: RwSignal::new(HashMap::new()),
            refresh_error: RwSignal::new(HashMap::new()),
            fetch_rev: RwSignal::new(HashMap::new()),
        };

        store.revisit();
        store.revisit();

        assert_eq!(
            calls.borrow().len(),
            2,
            "応答待ちの往復では同じ要求を重ねない(1周目の2タブだけ)"
        );
    });
}

#[test]
fn generation_change_resets_search_tab_and_visited() {
    let _ = any_spawner::Executor::init_futures_executor();
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        session.user.set(Some(user("alice")));
        let old_generation = session.generation.get_untracked();
        let calls = Rc::new(RefCell::new(Vec::new()));
        let record = Rc::clone(&calls);
        let fetch = Action::new_unsync(move |(generation, tab): &(Generation, ReceiptsTab)| {
            record.borrow_mut().push((*generation, *tab));
            std::future::pending()
        });
        let store = ReceiptsStore {
            session,
            active_tab: RwSignal::new(ReceiptsTab::MutualFund),
            search: RwSignal::new(ReceiptSearch {
                query: "7203".to_string(),
                ..Default::default()
            }),
            expanded: RwSignal::new(HashSet::new()),
            mobile_summary_expanded: RwSignal::new(false),
            utility_rail_open: RwSignal::new(true),
            expanded_epoch: RwSignal::new(None),
            visited: RwSignal::new(HashSet::from([
                ReceiptsTab::Dividend,
                ReceiptsTab::MutualFund,
            ])),
            cache: RwSignal::new(HashMap::new()),
            fetch,
            csv: RwSignal::new(HashMap::new()),
            csv_files: RwSignal::new(HashMap::new()),
            refresh_error: RwSignal::new(HashMap::from([(
                (old_generation, ReceiptsTab::Dividend),
                "古いエラー".to_string(),
            )])),
            fetch_rev: RwSignal::new(HashMap::new()),
        };
        // 前ユーザーの世代で状態を作る
        store.ensure(ReceiptsTab::MutualFund);
        let alice_calls = calls.borrow().len();

        session.mark_unauthenticated();
        session.user.set(Some(user("bob")));
        store.ensure(ReceiptsTab::MutualFund);

        assert_eq!(
            store.search.with_untracked(|search| search.query.clone()),
            "",
            "前ユーザーの検索語は持ち越さない"
        );
        assert_eq!(
            store.active_tab.get_untracked(),
            ReceiptsTab::Dividend,
            "前ユーザーの選択タブは持ち越さない"
        );
        assert_eq!(
            store.visited.with_untracked(|visited| visited.clone()),
            HashSet::from([ReceiptsTab::Dividend]),
            "前ユーザーの訪問済みは持ち越さない"
        );
        assert!(
            store.refresh_error.with_untracked(|map| map.is_empty()),
            "前ユーザーの裏再取得エラーは残さない"
        );
        let bob_generation = session.generation.get_untracked();
        let bob_calls: Vec<(Generation, ReceiptsTab)> = calls.borrow()[alice_calls..].to_vec();
        assert_eq!(
            bob_calls,
            vec![(bob_generation, ReceiptsTab::Dividend)],
            "前ユーザー選択のタブは新しい世代では取らない"
        );
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
            utility_rail_open: RwSignal::new(true),
            expanded_epoch: RwSignal::new(None),
            visited: RwSignal::new(HashSet::from([tab])),
            cache: RwSignal::new(HashMap::new()),
            fetch: Action::new_unsync(|_: &(Generation, ReceiptsTab)| async {}),
            csv: RwSignal::new(HashMap::new()),
            csv_files: RwSignal::new(HashMap::new()),
            refresh_error: RwSignal::new(HashMap::new()),
            fetch_rev: RwSignal::new(HashMap::new()),
        };

        store.ensure(tab);
        store.expanded.update(|set| {
            set.insert("g0".to_string());
        });
        store.mobile_summary_expanded.set(true);
        store.toggle_utility_rail();

        session.mark_unauthenticated();
        session.user.set(Some(user("bob")));
        store.ensure(tab);

        assert!(store.expanded.with_untracked(|set| set.is_empty()));
        assert!(!store.mobile_summary_expanded.get_untracked());
        assert!(store.utility_rail_open.get_untracked());
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
            utility_rail_open: RwSignal::new(true),
            expanded_epoch: RwSignal::new(None),
            visited: RwSignal::new(HashSet::from([tab])),
            cache: RwSignal::new(HashMap::new()),
            fetch: Action::new_unsync(|_: &(Generation, ReceiptsTab)| async {}),
            csv: RwSignal::new(HashMap::new()),
            csv_files: RwSignal::new(HashMap::new()),
            refresh_error: RwSignal::new(HashMap::new()),
            fetch_rev: RwSignal::new(HashMap::new()),
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

// 開閉はデフォルト開きで始まり、タブ切替・再取得では変わらない。
// 変えるのはユーザーのトグルと世代(ユーザー)切替のリセットだけ
#[test]
fn utility_rail_open_stays_until_toggle_or_generation_change() {
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
            utility_rail_open: RwSignal::new(true),
            expanded_epoch: RwSignal::new(None),
            visited: RwSignal::new(HashSet::from([tab])),
            cache: RwSignal::new(HashMap::new()),
            fetch: Action::new_unsync(|_: &(Generation, ReceiptsTab)| async {}),
            csv: RwSignal::new(HashMap::new()),
            csv_files: RwSignal::new(HashMap::new()),
            refresh_error: RwSignal::new(HashMap::new()),
            fetch_rev: RwSignal::new(HashMap::new()),
        };

        // データ有無・未取得にかかわらずデフォルト開きで始まる
        store.ensure(ReceiptsTab::Dividend);
        assert!(store.utility_rail_open.get_untracked());
        store.select_tab(ReceiptsTab::DomesticStock);
        assert!(store.utility_rail_open.get_untracked());

        // トグルした開閉はタブ移動・再取得をまたいでそのまま残る
        store.toggle_utility_rail();
        assert!(!store.utility_rail_open.get_untracked());
        store.select_tab(ReceiptsTab::MutualFund);
        assert!(!store.utility_rail_open.get_untracked());
        store.select_tab(ReceiptsTab::Dividend);
        assert!(!store.utility_rail_open.get_untracked());
        store.toggle_utility_rail();
        assert!(store.utility_rail_open.get_untracked());
    });
}

// 開いたままなら絞り込みはレール内で確認できるので、表の上の件数バッジは畳んだときだけ出す
#[test]
fn utility_filter_badge_is_only_shown_while_collapsed_and_filtering() {
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
            utility_rail_open: RwSignal::new(true),
            expanded_epoch: RwSignal::new(None),
            visited: RwSignal::new(HashSet::from([tab])),
            cache: RwSignal::new(HashMap::new()),
            fetch: Action::new_unsync(|_: &(Generation, ReceiptsTab)| async {}),
            csv: RwSignal::new(HashMap::new()),
            csv_files: RwSignal::new(HashMap::new()),
            refresh_error: RwSignal::new(HashMap::new()),
            fetch_rev: RwSignal::new(HashMap::new()),
        };

        assert!(!store.utility_filter_badge_visible());
        store.search.set(ReceiptSearch {
            query: "7203".to_string(),
            ..Default::default()
        });
        assert!(!store.utility_filter_badge_visible());

        store.toggle_utility_rail();
        assert!(store.utility_filter_badge_visible());
        store.toggle_utility_rail();
        assert!(!store.utility_filter_badge_visible());
    });
}

#[test]
fn dividend_cells_match_columns_and_formatting() {
    let row: crate::api::dto::Dividend = serde_json::from_value(serde_json::json!({
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
    let cells = Row::Saved(ReceiptItem::Dividend(row)).cells();
    assert_eq!(
        cells.iter().map(ReceiptCell::text).collect::<Vec<_>>(),
        vec![
            "2024/03/01",
            "特定口座",
            "SBI証券",
            "7203",
            "トヨタ自動車",
            "¥30",
            "100",
            "¥3,000",
            "¥609",
            "¥2,391",
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

#[test]
fn refresh_error_returns_entry_for_current_generation_only() {
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        session.user.set(Some(user("alice")));
        let store = ReceiptsStore {
            session,
            active_tab: RwSignal::new(ReceiptsTab::Dividend),
            search: RwSignal::new(ReceiptSearch::default()),
            expanded: RwSignal::new(HashSet::new()),
            mobile_summary_expanded: RwSignal::new(false),
            utility_rail_open: RwSignal::new(true),
            expanded_epoch: RwSignal::new(None),
            visited: RwSignal::new(HashSet::new()),
            cache: RwSignal::new(HashMap::new()),
            fetch: Action::new_unsync(|_: &(Generation, ReceiptsTab)| async {}),
            csv: RwSignal::new(HashMap::new()),
            csv_files: RwSignal::new(HashMap::new()),
            refresh_error: RwSignal::new(HashMap::new()),
            fetch_rev: RwSignal::new(HashMap::new()),
        };
        let tab = ReceiptsTab::Dividend;

        assert_eq!(store.refresh_error(tab), None);

        let generation = session.generation.get_untracked();
        let stale = generation.next();
        store.refresh_error.update(|map| {
            map.insert((stale, tab), "旧世代のエラー".to_string());
        });
        assert_eq!(store.refresh_error(tab), None);

        store.refresh_error.update(|map| {
            map.insert((generation, tab), "再取得に失敗".to_string());
        });
        assert_eq!(store.refresh_error(tab).as_deref(), Some("再取得に失敗"));
        assert_eq!(store.refresh_error(ReceiptsTab::DomesticStock), None);
    });
}

#[test]
fn csv_input_disabled_covers_unauth_busy_and_fetching() {
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        session.loaded.set(true);
        let store = ReceiptsStore {
            session,
            active_tab: RwSignal::new(ReceiptsTab::Dividend),
            search: RwSignal::new(ReceiptSearch::default()),
            expanded: RwSignal::new(HashSet::new()),
            mobile_summary_expanded: RwSignal::new(false),
            utility_rail_open: RwSignal::new(true),
            expanded_epoch: RwSignal::new(None),
            visited: RwSignal::new(HashSet::new()),
            cache: RwSignal::new(HashMap::new()),
            fetch: Action::new_unsync(|_: &(Generation, ReceiptsTab)| async {}),
            csv: RwSignal::new(HashMap::new()),
            csv_files: RwSignal::new(HashMap::new()),
            refresh_error: RwSignal::new(HashMap::new()),
            fetch_rev: RwSignal::new(HashMap::new()),
        };
        let tab = ReceiptsTab::Dividend;

        // 未ログインは他条件に関わらず無効(||→&& 変異はここで検出できる)
        assert!(store.csv_input_disabled(tab));

        session.user.set(Some(user("alice")));
        assert!(!store.csv_input_disabled(tab));

        let generation = session.generation.get_untracked();
        store.csv.update(|map| {
            map.insert(
                (generation, tab),
                crate::support::csv_flow::CsvTabState {
                    previewing: true,
                    ..Default::default()
                },
            );
        });
        assert!(store.csv_input_disabled(tab));
        store.csv.set(HashMap::new());

        store.cache.update(|map| {
            map.insert((generation, tab), TabState::Loading);
        });
        assert!(store.csv_input_disabled(tab));
    });
}

#[test]
fn ensure_prunes_stale_generation_in_any_state_map() {
    let _ = any_spawner::Executor::init_futures_executor();
    let owner = Owner::new();
    owner.with(|| {
        let session = SessionStore::new();
        session.user.set(Some(user("alice")));
        let store = ReceiptsStore {
            session,
            active_tab: RwSignal::new(ReceiptsTab::Dividend),
            search: RwSignal::new(ReceiptSearch::default()),
            expanded: RwSignal::new(HashSet::new()),
            mobile_summary_expanded: RwSignal::new(false),
            utility_rail_open: RwSignal::new(true),
            expanded_epoch: RwSignal::new(None),
            visited: RwSignal::new(HashSet::new()),
            cache: RwSignal::new(HashMap::new()),
            fetch: Action::new_unsync(|_: &(Generation, ReceiptsTab)| async {}),
            csv: RwSignal::new(HashMap::new()),
            csv_files: RwSignal::new(HashMap::new()),
            refresh_error: RwSignal::new(HashMap::new()),
            fetch_rev: RwSignal::new(HashMap::new()),
        };
        let tab = ReceiptsTab::Dividend;
        let stale = session.generation.get_untracked().next();

        // csv マップだけに旧世代の残滓を入れる。||→&& 変異は4マップ全部を要求するので
        // 残滓が消えなければ変異を検出できる
        store.csv.update(|map| {
            map.insert(
                (stale, tab),
                crate::support::csv_flow::CsvTabState::default(),
            );
        });
        // fetch_rev の retain は世代一致を保持する。==→!= 変異は新旧を反転させるので
        // 現世代の残滓が消えれば検出できる
        let generation = session.generation.get_untracked();
        store.fetch_rev.update(|map| {
            map.insert((generation, tab), 1_u64);
            map.insert((stale, tab), 1_u64);
        });

        store.ensure(tab);

        assert!(store
            .csv
            .with_untracked(|map| map.get(&(stale, tab)).is_none()));
        assert!(store.fetch_rev.with_untracked(
            |map| map.get(&(generation, tab)).is_some() && map.get(&(stale, tab)).is_none()
        ));
    });
}
