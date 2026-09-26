use crate::receipts::search::{is_valid_iso_date, SearchOption};
use rust_decimal::Decimal;
use std::collections::HashMap;

pub struct ColumnReorderRule<T> {
    pub column_key: usize,
    pub matches: fn(&T, &str) -> bool,
}

pub fn reorder_columns_by_search<T>(
    base: &[usize],
    data: &[T],
    query: &str,
    rules: &[ColumnReorderRule<T>],
    fixed: usize,
) -> Vec<usize> {
    if !query.is_empty() {
        let query = query.to_lowercase();
        for rule in rules {
            if data.iter().any(|row| (rule.matches)(row, &query)) && base.contains(&rule.column_key)
            {
                return base
                    .iter()
                    .take(fixed)
                    .copied()
                    .chain(std::iter::once(rule.column_key))
                    .chain(
                        base.iter()
                            .copied()
                            .filter(|key| *key != rule.column_key)
                            .skip(fixed),
                    )
                    .collect();
            }
        }
    }
    base.to_vec()
}

// ブラウザーの localeCompare による照合順序を利用する。
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(
    inline_js = "export function receiptLocaleCompare(a, b) { return a.localeCompare(b); }"
)]
extern "C" {
    #[wasm_bindgen::prelude::wasm_bindgen(js_name = receiptLocaleCompare)]
    fn locale_compare_js(a: &str, b: &str) -> i32;
}

pub fn locale_compare(a: &str, b: &str) -> std::cmp::Ordering {
    #[cfg(target_arch = "wasm32")]
    {
        locale_compare_js(a, b).cmp(&0)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        a.cmp(b)
    }
}

pub fn create_search_options<T>(
    data: &[T],
    value: fn(&T) -> &str,
    label: fn(&T) -> &str,
    prefix: bool,
    date: Option<fn(&T) -> &str>,
) -> Vec<SearchOption> {
    let mut entries: HashMap<String, (String, Option<&str>)> = HashMap::new();
    for row in data {
        let raw_value = value(row);
        let key = if raw_value.is_empty() {
            label(row)
        } else {
            raw_value
        };
        let text = if prefix && !raw_value.is_empty() {
            format!("{raw_value}: {}", label(row))
        } else {
            label(row).to_string()
        };
        let item_date = date.map(|get| get(row)).filter(|d| is_valid_iso_date(d));
        match entries.get_mut(key) {
            Some((old_label, old_date)) if item_date.is_some() && item_date > *old_date => {
                *old_label = text;
                *old_date = item_date;
            }
            None => {
                entries.insert(key.to_string(), (text, item_date));
            }
            _ => {}
        }
    }
    let mut options: Vec<_> = entries
        .into_iter()
        .map(|(value, (label, _))| SearchOption { value, label })
        .collect();
    options.sort_by(|a, b| locale_compare(&a.value, &b.value));
    options
}

#[derive(Debug, PartialEq)]
pub struct GroupSummary {
    pub filter: String,
    pub values: Vec<Decimal>,
}

pub fn group_and_summarize<T>(
    data: &[T],
    key: impl Fn(&T) -> String,
    fields: &[fn(&T) -> Decimal],
    descending: bool,
) -> Vec<GroupSummary> {
    let mut groups: HashMap<String, Vec<Decimal>> = HashMap::new();
    for row in data {
        let values = groups
            .entry(key(row))
            .or_insert_with(|| vec![Decimal::ZERO; fields.len()]);
        for (sum, field) in values.iter_mut().zip(fields) {
            *sum += field(row);
        }
    }
    let mut groups: Vec<_> = groups
        .into_iter()
        .map(|(filter, values)| GroupSummary { filter, values })
        .collect();
    groups.sort_by(|a, b| {
        if descending {
            locale_compare(&b.filter, &a.filter)
        } else {
            locale_compare(&a.filter, &b.filter)
        }
    });
    groups
}

#[cfg(test)]
mod tests;
