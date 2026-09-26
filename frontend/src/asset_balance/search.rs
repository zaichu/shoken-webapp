use crate::dto::{AssetBalance, SearchFacets};
use crate::receipts::search::{filter_by_config, FilterConfig, SearchOption};
use crate::receipts::search_support::create_search_options;

pub fn asset_balance_filter_config() -> FilterConfig<AssetBalance> {
    FilterConfig {
        string_fields: None,
        partial_string_fields: Some(vec![|row| &row.security_code, |row| &row.security_name]),
        date_field: None,
        year_search: false,
        year_month_search: false,
        date_search: false,
        date_range_search: false,
        amount_fields: None,
    }
}

pub fn filter_asset_balances<'a>(data: &'a [AssetBalance], query: &str) -> Vec<&'a AssetBalance> {
    filter_by_config(data, query, &asset_balance_filter_config())
}

pub fn asset_balance_search_options(
    data: &[AssetBalance],
    facets: Option<&SearchFacets>,
    has_csv_file: bool,
) -> Vec<SearchOption> {
    if !has_csv_file {
        if let Some(securities) = facets.and_then(|facets| facets.securities.as_ref()) {
            return securities
                .iter()
                .map(|option| SearchOption {
                    value: option.value.clone(),
                    label: option.label.clone(),
                })
                .collect();
        }
    }
    create_search_options(
        data,
        |row| row.security_code.as_str(),
        |row| row.security_name.as_str(),
        true,
        None,
    )
}

pub fn clear_search_query() -> String {
    String::new()
}

#[cfg(test)]
mod tests;
