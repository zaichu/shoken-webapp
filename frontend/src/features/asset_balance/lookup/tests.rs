use super::*;
use rust_decimal_macros::dec;

fn row(code: &str) -> AssetBalance {
    AssetBalance {
        id: format!("id-{code}").into(),
        security_code: code.parse().unwrap(),
        security_name: "銘柄".to_string(),
        shares: dec!(100),
        executing_shares: dec!(0),
        average_purchase_price: dec!(2500),
        total_purchase_amount: dec!(250000),
        current_price: dec!(2600),
        daily_change: dec!(50),
        market_value: dec!(260000),
        profit_loss_rate: dec!(4),
        created_at: String::new(),
        updated_at: String::new(),
    }
}

#[test]
fn find_matches_normalized_codes() {
    let rows = vec![row("7203"), row("brk.b")];
    assert_eq!(
        find_by_code(&rows, "7203").unwrap().security_code.as_str(),
        "7203"
    );
    assert_eq!(
        find_by_code(&rows, "7203: トヨタ自動車")
            .unwrap()
            .security_code
            .as_str(),
        "7203"
    );
    assert_eq!(
        find_by_code(&rows, "BRK.B").unwrap().security_code.as_str(),
        "brk.b"
    );
    assert_eq!(
        find_by_code(&rows, " 7203 ")
            .unwrap()
            .security_code
            .as_str(),
        "7203"
    );
    assert!(find_by_code(&rows, "").is_none());
    assert!(find_by_code(&rows, "   ").is_none());
    assert!(find_by_code(&rows, "9999").is_none());
    assert!(find_by_code(&[], "7203").is_none());
}

#[test]
fn store_seed_get_roundtrip() {
    let mut store = AssetBalanceLookupStore::new();
    assert!(store.get(1, "7203").is_none());
    store.seed(1, &[row("7203"), row("6758")]);
    assert_eq!(store.get(1, "7203").unwrap().security_code.as_str(), "7203");
    assert_eq!(
        store
            .get(1, "7203: トヨタ自動車")
            .unwrap()
            .security_code
            .as_str(),
        "7203"
    );
    assert!(store.get(1, "").is_none());
    assert!(store.get(1, "9999").is_none());
}

#[test]
fn store_isolated_by_generation() {
    let mut store = AssetBalanceLookupStore::new();
    store.seed(1, &[row("7203")]);
    assert!(store.get(2, "7203").is_none());
    store.seed(2, &[row("6758")]);
    assert!(store.get(1, "7203").is_none());
    assert!(store.get(1, "6758").is_none());
    assert_eq!(store.get(2, "6758").unwrap().security_code.as_str(), "6758");
    assert!(store.get(2, "7203").is_none());
}

#[test]
fn store_clear_and_stale() {
    let mut store = AssetBalanceLookupStore::new();
    store.seed(1, &[row("7203")]);
    store.clear_if_stale(1);
    assert!(store.get(1, "7203").is_some());
    store.clear_if_stale(2);
    assert!(store.get(1, "7203").is_none());
    assert!(store.get(2, "7203").is_none());
    store.seed(2, &[row("7203")]);
    store.clear();
    assert!(store.get(2, "7203").is_none());
}

#[test]
fn needs_fetch_mirrors_hook_enabled() {
    let mut store = AssetBalanceLookupStore::new();
    assert!(!store.needs_fetch(1, "7203", false));
    assert!(!store.needs_fetch(1, "", true));
    assert!(!store.needs_fetch(1, "  ", true));
    assert!(store.needs_fetch(1, "7203", true));
    store.seed(1, &[row("7203")]);
    assert!(!store.needs_fetch(1, "7203", true));
    assert!(!store.needs_fetch(1, "7203: トヨタ自動車", true));
    assert!(store.needs_fetch(1, "9999", true));
    assert!(store.needs_fetch(2, "7203", true));
}

#[test]
fn store_single_overwrites_entry() {
    let mut store = AssetBalanceLookupStore::new();
    store.seed(1, &[row("7203")]);
    let mut updated = row("7203");
    updated.security_name = "更新後".to_string();
    store.store_single(1, "7203", vec![updated]);
    assert_eq!(store.get(1, "7203").unwrap().security_name, "更新後");
    store.store_single(1, "", vec![row("0000")]);
    assert!(store.get(1, "0000").is_none());
    store.store_single(2, "7203", vec![]);
    assert!(store.get(1, "7203").is_none());
    assert!(store.get(2, "7203").is_none());
}

#[test]
fn store_single_on_new_generation_drops_all_old_entries() {
    let mut store = AssetBalanceLookupStore::new();
    store.seed(1, &[row("7203"), row("6758")]);
    store.store_single(2, "8306", vec![row("8306")]);
    assert!(store.get(1, "6758").is_none());
    assert_eq!(store.get(2, "8306").unwrap().security_code.as_str(), "8306");
    assert!(store.get(2, "7203").is_none());
}
