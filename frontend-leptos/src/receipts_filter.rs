use crate::receipts::{ReceiptItem, ReceiptsTab};
use crate::receipts_search::{
    create_year_options, filter_by_config, get_unique_values, is_js_whitespace, FilterConfig,
    SearchOption,
};
use crate::receipts_search_support::{
    create_search_options, reorder_columns_by_search, ColumnReorderRule,
};
use rust_decimal::Decimal;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SearchKey {
    Securities,
    Years,
    Products,
    Accounts,
    Date,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SelectedQueries {
    pub securities: String,
    pub years: String,
    pub products: String,
    pub accounts: String,
    pub date: String,
}

impl SelectedQueries {
    pub fn get(&self, key: SearchKey) -> &str {
        match key {
            SearchKey::Securities => &self.securities,
            SearchKey::Years => &self.years,
            SearchKey::Products => &self.products,
            SearchKey::Accounts => &self.accounts,
            SearchKey::Date => &self.date,
        }
    }

    fn set(&mut self, key: SearchKey, value: String) {
        match key {
            SearchKey::Securities => self.securities = value,
            SearchKey::Years => self.years = value,
            SearchKey::Products => self.products = value,
            SearchKey::Accounts => self.accounts = value,
            SearchKey::Date => self.date = value,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DateSegment {
    Year,
    Month,
    Date,
    Range,
}

impl DateSegment {
    pub fn label(self) -> &'static str {
        match self {
            Self::Year => "年",
            Self::Month => "月",
            Self::Date => "日",
            Self::Range => "範囲",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DateInputs {
    pub year_value: String,
    pub month_value: String,
    pub date_value: String,
    pub range_start: String,
    pub range_end: String,
}

pub fn build_range_query(start: &str, end: &str) -> String {
    if start.is_empty() && end.is_empty() {
        String::new()
    } else {
        format!("{start}..{end}")
    }
}

pub fn format_query_token(value: &str) -> String {
    let trimmed = value.trim_matches(is_js_whitespace);
    if trimmed.chars().any(is_js_whitespace) {
        format!("\"{}\"", trimmed.replace('"', "\\\""))
    } else {
        trimmed.to_string()
    }
}

pub fn build_combined_query(queries: &SelectedQueries) -> String {
    [
        SearchKey::Date,
        SearchKey::Securities,
        SearchKey::Years,
        SearchKey::Products,
        SearchKey::Accounts,
    ]
    .into_iter()
    .map(|key| queries.get(key).trim_matches(is_js_whitespace))
    .filter(|value| !value.is_empty())
    .map(format_query_token)
    .collect::<Vec<_>>()
    .join(" ")
}

pub fn initial_date_segment(has_years: bool) -> DateSegment {
    if has_years {
        DateSegment::Year
    } else {
        DateSegment::Month
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReceiptSearch {
    pub query: String,
    pub selected_queries: SelectedQueries,
    pub date_segment: DateSegment,
    pub date_inputs: DateInputs,
}

impl Default for ReceiptSearch {
    fn default() -> Self {
        Self::new(true)
    }
}

impl ReceiptSearch {
    pub fn new(has_years: bool) -> Self {
        Self {
            query: String::new(),
            selected_queries: SelectedQueries::default(),
            date_segment: initial_date_segment(has_years),
            date_inputs: DateInputs::default(),
        }
    }

    pub fn is_default(&self) -> bool {
        self.query.is_empty()
    }

    pub fn select_quick(&mut self, key: SearchKey, value: String) {
        let should_toggle_off = matches!(key, SearchKey::Products | SearchKey::Accounts)
            && self.selected_queries.get(key) == value;
        self.selected_queries.set(
            key,
            if value.is_empty() || should_toggle_off {
                String::new()
            } else {
                value
            },
        );
        self.rebuild_query();
    }

    pub fn change_date_segment(&mut self, segment: DateSegment) {
        if self.date_segment == segment {
            return;
        }
        self.date_segment = segment;
        self.date_inputs = DateInputs::default();
        self.selected_queries.date.clear();
        self.rebuild_query();
    }

    pub fn select_year(&mut self, value: String) {
        self.date_inputs.year_value = value.clone();
        self.set_date_query(value);
    }

    pub fn set_month(&mut self, value: String) {
        self.date_inputs.month_value = value.clone();
        self.set_date_query(value);
    }

    pub fn set_date(&mut self, value: String) {
        self.date_inputs.date_value = value.clone();
        self.set_date_query(value);
    }

    pub fn set_range_start(&mut self, value: String) {
        self.date_inputs.range_start = value;
        self.update_range_query();
    }

    pub fn set_range_end(&mut self, value: String) {
        self.date_inputs.range_end = value;
        self.update_range_query();
    }

    pub fn clear(&mut self, has_years: bool) {
        *self = Self::new(has_years);
    }

    fn set_date_query(&mut self, value: String) {
        self.selected_queries.date = value;
        self.rebuild_query();
    }

    fn update_range_query(&mut self) {
        self.selected_queries.date =
            build_range_query(&self.date_inputs.range_start, &self.date_inputs.range_end);
        self.rebuild_query();
    }

    fn rebuild_query(&mut self) {
        self.query = build_combined_query(&self.selected_queries);
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SearchCategories {
    pub securities: Vec<SearchOption>,
    pub products: Vec<SearchOption>,
    pub accounts: Vec<SearchOption>,
    pub years: Vec<SearchOption>,
}

impl ReceiptItem {
    pub fn date(&self) -> &str {
        match self {
            Self::Dividend(r) => &r.settlement_date,
            Self::DomesticStock(r) => &r.trade_date,
            Self::MutualFund(r) => &r.trade_date,
        }
    }
    pub fn code(&self) -> &str {
        match self {
            Self::Dividend(r) => &r.security_code,
            Self::DomesticStock(r) => &r.security_code,
            Self::MutualFund(_) => "",
        }
    }
    pub fn name(&self) -> &str {
        match self {
            Self::Dividend(r) => &r.security_name,
            Self::DomesticStock(r) => &r.security_name,
            Self::MutualFund(r) => &r.fund_name,
        }
    }
    pub fn account(&self) -> &str {
        match self {
            Self::Dividend(r) => &r.account,
            Self::DomesticStock(r) => &r.account,
            Self::MutualFund(r) => &r.account,
        }
    }
    pub fn product(&self) -> &str {
        match self {
            Self::Dividend(r) => &r.product,
            _ => "",
        }
    }
}

pub fn filter_receipts(tab: ReceiptsTab, rows: &[ReceiptItem], query: &str) -> Vec<ReceiptItem> {
    let mut config = FilterConfig {
        string_fields: Some(vec![
            ReceiptItem::code,
            ReceiptItem::name,
            ReceiptItem::account,
        ]),
        partial_string_fields: None,
        date_field: Some(ReceiptItem::date),
        year_search: true,
        year_month_search: true,
        date_search: true,
        date_range_search: true,
        amount_fields: None,
    };
    match tab {
        ReceiptsTab::Dividend => {
            config
                .string_fields
                .as_mut()
                .unwrap()
                .push(ReceiptItem::product);
            config.amount_fields = Some(vec![
                |r| match r {
                    ReceiptItem::Dividend(r) => r.unit_price,
                    _ => Decimal::ZERO,
                },
                |r| match r {
                    ReceiptItem::Dividend(r) => r.shares,
                    _ => Decimal::ZERO,
                },
                |r| match r {
                    ReceiptItem::Dividend(r) => r.dividends_before_tax,
                    _ => Decimal::ZERO,
                },
                |r| match r {
                    ReceiptItem::Dividend(r) => r.taxes,
                    _ => Decimal::ZERO,
                },
                |r| match r {
                    ReceiptItem::Dividend(r) => r.net_amount_received,
                    _ => Decimal::ZERO,
                },
            ]);
        }
        ReceiptsTab::DomesticStock => {
            config.amount_fields = Some(vec![
                |r| match r {
                    ReceiptItem::DomesticStock(r) => r.shares,
                    _ => Decimal::ZERO,
                },
                |r| match r {
                    ReceiptItem::DomesticStock(r) => r.asked_price,
                    _ => Decimal::ZERO,
                },
                |r| match r {
                    ReceiptItem::DomesticStock(r) => r.proceeds,
                    _ => Decimal::ZERO,
                },
                |r| match r {
                    ReceiptItem::DomesticStock(r) => r.purchase_price,
                    _ => Decimal::ZERO,
                },
                |r| match r {
                    ReceiptItem::DomesticStock(r) => r.realized_profit_and_loss,
                    _ => Decimal::ZERO,
                },
            ]);
        }
        ReceiptsTab::MutualFund => {
            config.string_fields = Some(vec![ReceiptItem::name, ReceiptItem::account]);
        }
    }
    filter_by_config(rows, query, &config)
        .into_iter()
        .cloned()
        .collect()
}

pub fn search_categories(tab: ReceiptsTab, rows: &[ReceiptItem]) -> SearchCategories {
    let mut sorted = rows.to_vec();
    sorted.sort_by(|a, b| b.date().cmp(a.date()));
    let to_options = |values: Vec<String>| {
        values
            .into_iter()
            .map(|value| SearchOption {
                label: value.clone(),
                value,
            })
            .collect()
    };
    SearchCategories {
        securities: create_search_options(
            &sorted,
            ReceiptItem::code,
            ReceiptItem::name,
            true,
            Some(ReceiptItem::date),
        ),
        products: if tab == ReceiptsTab::Dividend {
            to_options(get_unique_values(&sorted, ReceiptItem::product))
        } else {
            vec![]
        },
        accounts: if tab != ReceiptsTab::MutualFund {
            to_options(get_unique_values(&sorted, ReceiptItem::account))
        } else {
            vec![]
        },
        years: create_year_options(&sorted, ReceiptItem::date),
    }
}

pub fn column_order(tab: ReceiptsTab, rows: &[ReceiptItem], query: &str) -> Vec<usize> {
    let base: Vec<_> = (0..if tab == ReceiptsTab::DomesticStock {
        11
    } else {
        10
    })
        .collect();
    let account = |r: &ReceiptItem, q: &str| r.account().to_lowercase().contains(q);
    match tab {
        ReceiptsTab::Dividend => reorder_columns_by_search(
            &base,
            rows,
            query,
            &[
                ColumnReorderRule {
                    column_key: 1,
                    matches: |r: &ReceiptItem, q| r.product().to_lowercase().contains(q),
                },
                ColumnReorderRule {
                    column_key: 2,
                    matches: account,
                },
            ],
            1,
        ),
        ReceiptsTab::DomesticStock => reorder_columns_by_search(
            &base,
            rows,
            query,
            &[ColumnReorderRule {
                column_key: 3,
                matches: account,
            }],
            2,
        ),
        ReceiptsTab::MutualFund => base,
    }
}

#[cfg(test)]
pub(crate) mod tests;
