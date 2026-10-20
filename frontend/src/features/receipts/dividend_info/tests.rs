use super::*;
use crate::api::dto::Dividend;
use crate::features::receipts::ReceiptItem;
use crate::support::row::Row;
use rust_decimal_macros::dec;

fn dividend(code: &str, name: &str) -> ReceiptRow {
    Row::Saved(ReceiptItem::Dividend(Dividend {
        id: "id".to_string().into(),
        settlement_date: "2024-03-01".to_string(),
        product: "特定口座".to_string(),
        account: "SBI証券".parse().unwrap(),
        security_code: code.to_string(),
        security_name: name.to_string(),
        unit_price: dec!(30),
        shares: dec!(100),
        dividends_before_tax: dec!(3000),
        taxes: dec!(609),
        net_amount_received: dec!(2391),
        created_at: String::new(),
        updated_at: String::new(),
    }))
}

fn authenticated_session() -> SessionStore {
    let session = SessionStore::new();
    session.user.set(Some(crate::api::dto::SessionUser {
        id: "a".to_string(),
        email: "a@example.com".to_string(),
        name: None,
    }));
    session
}

fn init_test_executor() {
    let _ = any_spawner::Executor::init_futures_executor();
}

#[test]
fn search_security_code_gates_on_exact_code_or_name_match() {
    let rows = vec![
        dividend("9904", "架空自動車"),
        dividend("9914", "架空エレクトロニクス"),
    ];
    assert_eq!(search_security_code(&rows, "9904"), "9904");
    assert_eq!(search_security_code(&rows, "架空自動車"), "9904");
    assert_eq!(search_security_code(&rows, "9904: 架空自動車"), "9904");
    assert_eq!(search_security_code(&rows, "架空"), "");
    assert_eq!(search_security_code(&rows, ""), "");
    assert_eq!(search_security_code(&rows, "9999"), "");
    assert_eq!(search_security_code(&[], "9904"), "");
}

#[test]
fn search_security_code_rejects_non_ascii_codes() {
    let rows = vec![dividend("７２０３", "架空自動車")];
    assert_eq!(search_security_code(&rows, "７２０３"), "");
}

#[test]
fn set_code_dedupes_same_generation_code_and_clears_when_disabled() {
    let owner = Owner::new();
    owner.with(|| {
        let session = authenticated_session();
        let generation = session.generation.get_untracked();
        let store = DividendInfoStore::new(session);

        store.set_code(generation, false, "9904");
        assert!(store.current.get_untracked().is_none());

        store.set_code(generation, true, "７２０３");
        assert!(store.current.get_untracked().is_none());

        store.current.set(Some((generation, "9904".to_string())));
        store.set_code(generation, true, " 9904 ");
        assert_eq!(
            store.current.get_untracked(),
            Some((generation, "9904".to_string()))
        );

        store.set_code(generation, false, "9904");
        assert!(store.current.get_untracked().is_none());
        assert!(store.asset_balance.get_untracked().is_none());
        assert!(store.per_share.get_untracked().is_none());
    });
}

#[test]
fn set_code_bumps_revisions_so_stale_requests_stay_inactive() {
    init_test_executor();
    let owner = Owner::new();
    owner.with(|| {
        let session = authenticated_session();
        let generation = session.generation.get_untracked();
        let store = DividendInfoStore::new(session);

        store.set_code(generation, true, "9904");
        let first_poll = store.code_revision.get_untracked();
        let first_balance = store.balance_revision.get_untracked();

        store.set_code(generation, true, "9914");
        store.set_code(generation, true, "9904");
        let latest_poll = store.code_revision.get_untracked();
        assert!(latest_poll > first_poll);

        // A→B→A と戻っても、先に投げたポーリング・保有数取得は再有効化しない
        assert!(!store.is_poll_active(generation, first_poll, "9904"));
        assert!(!store.is_balance_active(generation, first_balance, "9904"));
        assert!(store.is_poll_active(generation, latest_poll, "9904"));
    });
}

#[test]
fn refresh_balance_supersedes_balance_fetch_and_keeps_poll_alive() {
    init_test_executor();
    let owner = Owner::new();
    owner.with(|| {
        let session = authenticated_session();
        let generation = session.generation.get_untracked();
        let store = DividendInfoStore::new(session);
        store.set_code(generation, true, "9904");
        let poll = store.code_revision.get_untracked();
        let balance = store.balance_revision.get_untracked();

        store.refresh_balance();
        assert_eq!(store.balance_revision.get_untracked(), balance + 1);
        assert!(!store.is_balance_active(generation, balance, "9904"));
        assert!(store.is_balance_active(generation, balance + 1, "9904"));
        assert_eq!(store.code_revision.get_untracked(), poll);
        assert!(store.is_poll_active(generation, poll, "9904"));

        store.set_code(generation, false, "9904");
        let bumped = store.balance_revision.get_untracked();
        store.refresh_balance();
        assert_eq!(store.balance_revision.get_untracked(), bumped);
    });
}

#[test]
fn per_share_display_uses_short_decimal_digits() {
    assert_eq!(per_share_display(Some(50.1), false), "¥50.1");
    assert_eq!(per_share_display(Some(50.0), false), "¥50");
    assert_eq!(per_share_display(Some(50.12345), false), "¥50.12345");
    assert_eq!(per_share_display(None, false), "—");
    assert_eq!(per_share_display(Some(50.1), true), "取得中...");
    assert_eq!(per_share_display(Some(f64::NAN), false), "—");
}

#[test]
fn percentage_value_formats_with_fixed_decimals() {
    assert_eq!(format_percentage_value(2.0), "2.00%");
    assert_eq!(format_percentage_value(0.9564), "0.96%");
    assert_eq!(format_percentage_value(1.005), "1.00%");
    assert_eq!(format_percentage_value(2.675), "2.67%");
    assert_eq!(format_percentage_value(f64::INFINITY), "inf%");
    assert_eq!(format_percentage_value(f64::NAN), "—");
}

#[test]
fn is_current_code_requires_matching_session_generation_and_code() {
    let owner = Owner::new();
    owner.with(|| {
        let session = authenticated_session();
        let generation = session.generation.get_untracked();
        let store = DividendInfoStore::new(session);

        assert!(!store.is_current_code(generation, "9904"));
        store.current.set(Some((generation, "9904".to_string())));
        assert!(store.is_current_code(generation, "9904"));
        assert!(!store.is_current_code(generation, "9914"));
        assert!(!store.is_current_code(generation.next(), "9904"));
        session.mark_unauthenticated();
        assert!(!store.is_current_code(generation, "9904"));
    });
}

#[test]
fn investment_amount_is_price_times_shares() {
    let row = AssetBalance {
        id: "id".to_string().into(),
        security_code: "9904".parse().unwrap(),
        security_name: "銘柄".to_string(),
        shares: dec!(100),
        average_purchase_price: dec!(2500),
        total_purchase_amount: dec!(250_000),
        current_price: dec!(2600),
        daily_change: dec!(50),
        price_as_of: Some("2024-01-16".to_string()),
        prev_close: Some(dec!(2550)),
        created_at: String::new(),
        updated_at: String::new(),
    };
    assert_eq!(investment_amount(&row), 250_000.0);
}

#[test]
fn dividend_rate_requires_positive_investment_and_amount() {
    assert_eq!(dividend_rate(400.0, 10_000.0), 4.0);
    assert_eq!(dividend_rate(0.0, 10_000.0), 0.0);
    assert_eq!(dividend_rate(400.0, 0.0), 0.0);
    assert_eq!(dividend_rate(0.0, 0.0), 0.0);
    assert!((dividend_rate(400.0, 10_000.0) - 4.0).abs() < 1e-9);
}
