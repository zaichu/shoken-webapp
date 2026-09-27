use crate::api::dto::{AssetBalance, AssetBalanceListResponse};
use crate::api::{ApiClient, ApiError};
use crate::features::asset_balance::model::normalize_security_code;
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
            let code = normalize_security_code(row.security_code.as_str());
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
        .find(|row| normalize_security_code(row.security_code.as_str()) == wanted)
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
mod tests;
