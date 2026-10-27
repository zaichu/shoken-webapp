use crate::api::dto::{AssetBalance, AssetBalanceListResponse};
use crate::api::{ApiClient, ApiError};
use crate::features::asset_balance::model::normalize_security_code;
use crate::session::Generation;
use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub struct AssetBalanceLookupStore {
    generation: Option<Generation>,
    entries: HashMap<String, Vec<AssetBalance>>,
}

impl AssetBalanceLookupStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn seed(&mut self, generation: Generation, rows: &[AssetBalance]) {
        if self.generation != Some(generation) {
            self.entries.clear();
            self.generation = Some(generation);
        }
        for row in rows {
            let code = normalize_security_code(row.security_code.as_str());
            if code.is_empty() {
                continue;
            }
            let entries = self.entries.entry(code).or_default();
            // 同一銘柄コードでも口座違いの別行があり得るため、レコード ID で差し替える
            match entries.iter_mut().find(|entry| entry.id == row.id) {
                Some(entry) => *entry = row.clone(),
                None => entries.push(row.clone()),
            }
        }
    }

    /// 一覧の保存済み行と同じレコードを返す。
    /// 同一銘柄コードの口座別行を混同しないよう id で照合する
    pub fn get(&self, generation: Generation, row: &AssetBalance) -> Option<&AssetBalance> {
        if self.generation != Some(generation) {
            return None;
        }
        let normalized = normalize_security_code(row.security_code.as_str());
        if normalized.is_empty() {
            return None;
        }
        self.entries
            .get(&normalized)
            .and_then(|rows| rows.iter().find(|entry| entry.id == row.id))
    }

    pub fn needs_fetch(&self, generation: Generation, code: &str, authenticated: bool) -> bool {
        if !authenticated {
            return false;
        }
        let normalized = normalize_security_code(code);
        if normalized.is_empty() {
            return false;
        }
        self.generation != Some(generation) || !self.entries.contains_key(&normalized)
    }

    pub fn store_single(&mut self, generation: Generation, code: &str, rows: Vec<AssetBalance>) {
        if self.generation != Some(generation) {
            self.entries.clear();
            self.generation = Some(generation);
        }
        let normalized = normalize_security_code(code);
        if !normalized.is_empty() {
            self.entries.insert(normalized, rows);
        }
    }

    pub fn clear_if_stale(&mut self, generation: Generation) {
        if self.generation != Some(generation) {
            self.clear();
        }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.generation = None;
    }
}

/// コードだけで照合する用途(配当情報など口座を区別しないもの)向けの先頭行検索
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
            super::csv::LIST_PATH,
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
