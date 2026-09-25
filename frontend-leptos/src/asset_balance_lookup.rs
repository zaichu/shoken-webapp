use crate::api::{ApiClient, ApiError};
use crate::asset_balance_domain::normalize_security_code;
use crate::dto::{AssetBalance, AssetBalanceListResponse};
use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub struct AssetBalanceLookupStore {
    generation: Option<u64>,
    entries: HashMap<String, Vec<AssetBalance>>,
}

impl AssetBalanceLookupStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn seed(&mut self, generation: u64, rows: &[AssetBalance]) {
        if self.generation != Some(generation) {
            self.entries.clear();
            self.generation = Some(generation);
        }
        for row in rows {
            let code = normalize_security_code(&row.security_code);
            if !code.is_empty() {
                self.entries
                    .entry(code)
                    .or_insert_with(|| vec![row.clone()]);
            }
        }
    }

    pub fn get(&self, generation: u64, code: &str) -> Option<&AssetBalance> {
        if self.generation != Some(generation) {
            return None;
        }
        let normalized = normalize_security_code(code);
        if normalized.is_empty() {
            return None;
        }
        self.entries
            .get(&normalized)
            .and_then(|rows| find_by_code(rows, code))
    }

    pub fn needs_fetch(&self, generation: u64, code: &str, authenticated: bool) -> bool {
        if !authenticated {
            return false;
        }
        let normalized = normalize_security_code(code);
        if normalized.is_empty() {
            return false;
        }
        self.generation != Some(generation) || !self.entries.contains_key(&normalized)
    }

    pub fn store_single(&mut self, generation: u64, code: &str, rows: Vec<AssetBalance>) {
        if self.generation != Some(generation) {
            self.entries.clear();
            self.generation = Some(generation);
        }
        let normalized = normalize_security_code(code);
        if !normalized.is_empty() {
            self.entries.insert(normalized, rows);
        }
    }

    pub fn clear_if_stale(&mut self, generation: u64) {
        if self.generation != Some(generation) {
            self.clear();
        }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.generation = None;
    }
}

pub fn find_by_code<'a>(rows: &'a [AssetBalance], code: &str) -> Option<&'a AssetBalance> {
    let wanted = normalize_security_code(code);
    if wanted.is_empty() {
        return None;
    }
    rows.iter()
        .find(|row| normalize_security_code(&row.security_code) == wanted)
}

pub async fn fetch_single_asset_balance(
    client: &ApiClient,
    code: &str,
) -> Result<Vec<AssetBalance>, ApiError> {
    let normalized = normalize_security_code(code);
    client
        .get_json::<AssetBalanceListResponse>(
            "/api/v1/asset-balances",
            &[
                ("page", "1"),
                ("per_page", "1"),
                ("security_code", normalized.as_str()),
            ],
        )
        .await
        .map(|list| list.data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn row(code: &str) -> AssetBalance {
        AssetBalance {
            id: format!("id-{code}"),
            security_code: code.to_string(),
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
        assert_eq!(find_by_code(&rows, "7203").unwrap().security_code, "7203");
        assert_eq!(
            find_by_code(&rows, "7203: トヨタ自動車")
                .unwrap()
                .security_code,
            "7203"
        );
        assert_eq!(find_by_code(&rows, "BRK.B").unwrap().security_code, "brk.b");
        assert_eq!(find_by_code(&rows, " 7203 ").unwrap().security_code, "7203");
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
        assert_eq!(store.get(1, "7203").unwrap().security_code, "7203");
        assert_eq!(
            store.get(1, "7203: トヨタ自動車").unwrap().security_code,
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
        assert_eq!(store.get(2, "6758").unwrap().security_code, "6758");
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
        assert_eq!(store.get(2, "8306").unwrap().security_code, "8306");
        assert!(store.get(2, "7203").is_none());
    }
}
