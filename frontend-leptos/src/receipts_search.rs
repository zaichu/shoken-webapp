use rust_decimal::Decimal;

pub struct FilterConfig<T> {
    pub string_fields: Vec<fn(&T) -> &str>,
    pub partial_string_fields: Vec<fn(&T) -> &str>,
    pub date_field: Option<fn(&T) -> &str>,
    pub year_search: bool,
    pub year_month_search: bool,
    pub date_search: bool,
    pub date_range_search: bool,
    pub amount_fields: Vec<fn(&T) -> Decimal>,
}

pub fn filter_by_config<'a, T>(data: &'a [T], _query: &str, _config: &FilterConfig<T>) -> Vec<&'a T> {
    data.iter().collect()
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchOption { pub value: String, pub label: String }
pub fn create_year_options<T>(_data: &[T], _date: impl Fn(&T) -> &str) -> Vec<SearchOption> { vec![] }
pub fn get_unique_values<T>(_data: &[T], _getter: impl Fn(&T) -> &str) -> Vec<String> { vec![] }
pub fn matches_year(_date: &str, _query: &str) -> bool { false }
pub fn matches_year_month(_date: &str, _query: &str) -> bool { false }
#[cfg(test)]
mod tests;
