use crate::features::asset_balance::csv::{AssetBalanceRow, AssetBalanceRowData};
use crate::features::asset_balance::fund::AssetSearch;
use crate::support::list_search::{FilterConfig, filter_by_config};

fn asset_balance_filter_config() -> FilterConfig<AssetBalanceRow> {
    FilterConfig {
        string_fields: None,
        partial_string_fields: Some(vec![
            |row| row.security_code(),
            |row| row.security_name(),
            |row| row.account().unwrap_or(""),
        ]),
        date_field: None,
        year_search: false,
        year_month_search: false,
        date_search: false,
        date_range_search: false,
        amount_fields: None,
    }
}

/// テキストの部分一致と、口座の選択条件で絞る。口座選択中は口座なしの行を除く
pub fn filter_asset_balances<'a>(
    data: &'a [AssetBalanceRow],
    search: &AssetSearch,
) -> Vec<&'a AssetBalanceRow> {
    let filtered = filter_by_config(data, &search.text, &asset_balance_filter_config());
    if search.accounts.is_empty() {
        return filtered;
    }
    filtered
        .into_iter()
        .filter(|row| {
            row.account()
                .is_some_and(|account| search.accounts.iter().any(|a| a == account))
        })
        .collect()
}

/// 行に出ている口座の選択肢(重複なし・出現順)
pub fn stock_accounts(data: &[AssetBalanceRow]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for row in data {
        if let Some(account) = row.account()
            && !out.iter().any(|a| a == account)
        {
            out.push(account.to_string());
        }
    }
    out
}

#[cfg(test)]
mod tests;
