use crate::api::dto::SearchFacets;
use crate::features::asset_balance::csv::{AssetBalanceRow, AssetBalanceRowData};
use crate::features::asset_balance::model::normalize_display_name;
use crate::support::list_search::support::create_search_options;
use crate::support::list_search::{FilterConfig, SearchOption, filter_by_config};

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

pub fn asset_balance_search_options(
    data: &[AssetBalanceRow],
    facets: Option<&SearchFacets>,
    has_csv_file: bool,
) -> Vec<SearchOption> {
    #[allow(clippy::collapsible_if)]
    if !has_csv_file {
        if let Some(securities) = facets.and_then(|facets| facets.securities.as_ref()) {
            return securities
                .iter()
                .map(|option| SearchOption {
                    value: option.value.clone(),
                    label: normalize_display_name(&option.label),
                })
                .collect();
        }
    }
    create_search_options(
        data,
        |row| row.security_code(),
        |row| row.security_name(),
        true,
        None,
    )
    .into_iter()
    .map(|mut option| {
        option.label = normalize_display_name(&option.label);
        option
    })
    .collect()
}

#[cfg(test)]
mod tests;
