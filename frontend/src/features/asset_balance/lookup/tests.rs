use super::*;
use rust_decimal_macros::dec;

fn row(code: &str) -> AssetBalance {
    AssetBalance {
        id: format!("id-{code}").into(),
        security_code: code.parse().unwrap(),
        security_name: "銘柄".to_string(),
        shares: dec!(100),
        average_purchase_price: dec!(2500),
        total_purchase_amount: dec!(250000),
        current_price: dec!(2600),
        daily_change: dec!(50),
        price_as_of: Some("2024-01-16".to_string()),
        created_at: String::new(),
        updated_at: String::new(),
    }
}

#[test]
fn find_matches_normalized_codes() {
    let rows = vec![row("9904"), row("brk.b")];
    assert_eq!(
        find_by_code(&rows, "9904").unwrap().security_code.as_str(),
        "9904"
    );
    assert_eq!(
        find_by_code(&rows, "9904: 架空自動車")
            .unwrap()
            .security_code
            .as_str(),
        "9904"
    );
    assert_eq!(
        find_by_code(&rows, "BRK.B").unwrap().security_code.as_str(),
        "brk.b"
    );
    assert_eq!(
        find_by_code(&rows, " 9904 ")
            .unwrap()
            .security_code
            .as_str(),
        "9904"
    );
    assert!(find_by_code(&rows, "").is_none());
    assert!(find_by_code(&rows, "   ").is_none());
    assert!(find_by_code(&rows, "9999").is_none());
    assert!(find_by_code(&[], "9904").is_none());
}

#[test]
fn store_seed_get_roundtrip() {
    let mut store = AssetBalanceLookupStore::new();
    assert!(store.get(Generation::new(1), "9904").is_none());
    store.seed(Generation::new(1), &[row("9904"), row("6758")]);
    assert_eq!(
        store
            .get(Generation::new(1), "9904")
            .unwrap()
            .security_code
            .as_str(),
        "9904"
    );
    assert_eq!(
        store
            .get(Generation::new(1), "9904: 架空自動車")
            .unwrap()
            .security_code
            .as_str(),
        "9904"
    );
    assert!(store.get(Generation::new(1), "").is_none());
    assert!(store.get(Generation::new(1), "9999").is_none());
}

#[test]
fn store_isolated_by_generation() {
    let mut store = AssetBalanceLookupStore::new();
    store.seed(Generation::new(1), &[row("9904")]);
    assert!(store.get(Generation::new(2), "9904").is_none());
    store.seed(Generation::new(2), &[row("6758")]);
    assert!(store.get(Generation::new(1), "9904").is_none());
    assert!(store.get(Generation::new(1), "6758").is_none());
    assert_eq!(
        store
            .get(Generation::new(2), "6758")
            .unwrap()
            .security_code
            .as_str(),
        "6758"
    );
    assert!(store.get(Generation::new(2), "9904").is_none());
}

#[test]
fn store_clear_and_stale() {
    let mut store = AssetBalanceLookupStore::new();
    store.seed(Generation::new(1), &[row("9904")]);
    store.clear_if_stale(Generation::new(1));
    assert!(store.get(Generation::new(1), "9904").is_some());
    store.clear_if_stale(Generation::new(2));
    assert!(store.get(Generation::new(1), "9904").is_none());
    assert!(store.get(Generation::new(2), "9904").is_none());
    store.seed(Generation::new(2), &[row("9904")]);
    store.clear();
    assert!(store.get(Generation::new(2), "9904").is_none());
}

#[test]
fn needs_fetch_mirrors_hook_enabled() {
    let mut store = AssetBalanceLookupStore::new();
    assert!(!store.needs_fetch(Generation::new(1), "9904", false));
    assert!(!store.needs_fetch(Generation::new(1), "", true));
    assert!(!store.needs_fetch(Generation::new(1), "  ", true));
    assert!(store.needs_fetch(Generation::new(1), "9904", true));
    store.seed(Generation::new(1), &[row("9904")]);
    assert!(!store.needs_fetch(Generation::new(1), "9904", true));
    assert!(!store.needs_fetch(Generation::new(1), "9904: 架空自動車", true));
    assert!(store.needs_fetch(Generation::new(1), "9999", true));
    assert!(store.needs_fetch(Generation::new(2), "9904", true));
}

#[test]
fn store_single_overwrites_entry() {
    let mut store = AssetBalanceLookupStore::new();
    store.seed(Generation::new(1), &[row("9904")]);
    let mut updated = row("9904");
    updated.security_name = "更新後".to_string();
    store.store_single(Generation::new(1), "9904", vec![updated]);
    assert_eq!(
        store.get(Generation::new(1), "9904").unwrap().security_name,
        "更新後"
    );
    store.store_single(Generation::new(1), "", vec![row("0000")]);
    assert!(store.get(Generation::new(1), "0000").is_none());
    store.store_single(Generation::new(2), "9904", vec![]);
    assert!(store.get(Generation::new(1), "9904").is_none());
    assert!(store.get(Generation::new(2), "9904").is_none());
}

#[test]
fn store_single_on_new_generation_drops_all_old_entries() {
    let mut store = AssetBalanceLookupStore::new();
    store.seed(Generation::new(1), &[row("9904"), row("6758")]);
    store.store_single(Generation::new(2), "9907", vec![row("9907")]);
    assert!(store.get(Generation::new(1), "6758").is_none());
    assert_eq!(
        store
            .get(Generation::new(2), "9907")
            .unwrap()
            .security_code
            .as_str(),
        "9907"
    );
    assert!(store.get(Generation::new(2), "9904").is_none());
}
