use crate::features::asset_balance::csv::{AssetBalanceRow, AssetBalanceRowData};
use crate::support::list_search::{FilterConfig, filter_by_config};

pub fn asset_balance_filter_config() -> FilterConfig<AssetBalanceRow> {
    FilterConfig {
        string_fields: None,
        partial_string_fields: Some(vec![|row| row.security_code(), |row| row.security_name()]),
        date_field: None,
        year_search: false,
        year_month_search: false,
        date_search: false,
        date_range_search: false,
        amount_fields: None,
    }
}

pub fn filter_asset_balances<'a>(
    data: &'a [AssetBalanceRow],
    query: &str,
) -> Vec<&'a AssetBalanceRow> {
    filter_by_config(data, query, &asset_balance_filter_config())
}

#[cfg(test)]
mod tests;
