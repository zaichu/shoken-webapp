use crate::features::receipts::{ReceiptRow, ReceiptsTab};
use crate::support::list_search::support::{create_search_options, reorder_columns_by_search};
use crate::support::list_search::{
    SearchOption, create_year_options, filter_by_config, get_unique_values, is_js_whitespace,
};
use shared::normalize::normalize_display_name;
use std::collections::HashSet;

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

    fn is_empty(&self) -> bool {
        self.securities.is_empty()
            && self.years.is_empty()
            && self.products.is_empty()
            && self.accounts.is_empty()
            && self.date.is_empty()
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
        self.query.is_empty() && self.selected_queries.is_empty()
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
    pub dates: bool,
}

pub fn filter_receipts(tab: ReceiptsTab, rows: &[ReceiptRow], query: &str) -> Vec<ReceiptRow> {
    filter_by_config(rows, query, &tab.filter_config())
        .into_iter()
        .cloned()
        .collect()
}

pub fn search_categories(tab: ReceiptsTab, rows: &[ReceiptRow]) -> SearchCategories {
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
        securities: {
            let mut seen = HashSet::new();
            create_search_options(
                &sorted,
                ReceiptRow::code,
                ReceiptRow::name,
                true,
                Some(ReceiptRow::date),
            )
            .into_iter()
            .map(|mut option| {
                option.label = normalize_display_name(&option.label);
                option.value = normalize_display_name(&option.value);
                option
            })
            // 全角・半角だけ違う名前は同じ選択肢にまとめる(絞り込み・グループ化も正規化後の値で一致する)
            .filter(|option| seen.insert(option.value.clone()))
            .collect()
        },
        products: if tab.product_category() {
            to_options(get_unique_values(&sorted, ReceiptRow::product))
        } else {
            vec![]
        },
        accounts: if tab.account_category() {
            to_options(get_unique_values(&sorted, ReceiptRow::account))
        } else {
            vec![]
        },
        years: create_year_options(&sorted, ReceiptRow::date),
        dates: true,
    }
}

pub fn column_order(tab: ReceiptsTab, rows: &[ReceiptRow], query: &str) -> Vec<usize> {
    let base: Vec<_> = (0..tab.headers().len()).collect();
    reorder_columns_by_search(&base, rows, query, tab.reorder_rules(), tab.reorder_fixed())
}

/// 検索に一致して前に出す列。column_order と同じ規則で最初に一致したもの
pub fn promoted_column(tab: ReceiptsTab, rows: &[ReceiptRow], query: &str) -> Option<usize> {
    if query.is_empty() {
        return None;
    }
    let query = normalize_display_name(query).to_lowercase();
    tab.reorder_rules()
        .iter()
        .find(|rule| rows.iter().any(|row| (rule.matches)(row, &query)))
        .map(|rule| rule.column_key)
}

#[cfg(test)]
pub(crate) mod tests;
