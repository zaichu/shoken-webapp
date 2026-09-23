use super::*;
use serde::Deserialize;
use rust_decimal::Decimal;

#[derive(Clone, Deserialize)]
struct Item { id: String, code: String, name: String, category: String, date: String, amount: Decimal }
#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct Config { string_fields: Vec<String>, partial_string_fields: Vec<String>, date_field: Option<String>, year_search: bool, year_month_search: bool, date_search: bool, date_range_search: bool, amount_fields: Vec<String> }
#[derive(Deserialize)]
struct Case { name: String, data: Vec<Item>, query: String, config: Config, expected_ids: Vec<String> }
#[derive(Deserialize)]
struct Fixture { cases: Vec<Case> }
fn string_getter(field: &str) -> fn(&Item) -> &str {
 match field { "code" => |r| &r.code, "name" => |r| &r.name, "category" => |r| &r.category, _ => panic!("unknown field") }
}
fn fixture() -> Fixture { serde_json::from_str(include_str!("../../tests/fixtures/receipts/search.json")).unwrap() }
fn check(case: &Case) {
 let c = &case.config;
 let config = FilterConfig {
 string_fields: c.string_fields.iter().map(|s| string_getter(s)).collect(),
 partial_string_fields: c.partial_string_fields.iter().map(|s| string_getter(s)).collect(),
 date_field: c.date_field.as_ref().map(|_| (|r: &Item| r.date.as_str()) as fn(&Item)->&str),
 year_search: c.year_search, year_month_search: c.year_month_search, date_search: c.date_search, date_range_search: c.date_range_search,
 amount_fields: c.amount_fields.iter().map(|_| (|r: &Item| r.amount) as fn(&Item)->Decimal).collect(),
 };
 let actual: Vec<_> = filter_by_config(&case.data, &case.query, &config).iter().map(|r| r.id.clone()).collect();
 assert_eq!(actual, case.expected_ids, "{}", case.name);
}
#[test]
fn shared_search_fixtures() { for case in fixture().cases { check(&case); } }
#[test]
fn react_search_exact_match() { check(&fixture().cases[0]); }
#[test]
fn react_search_exact_rejects_partial() { check(&fixture().cases[1]); }
#[test]
fn react_search_partial_match() { check(&fixture().cases[2]); }
#[test]
fn react_search_case_insensitive() { check(&fixture().cases[3]); }
#[test]
fn react_search_multiple_partial_fields() { check(&fixture().cases[4]); }
#[test]
fn react_search_and_tokens() { check(&fixture().cases[5]); }
#[test]
fn react_search_quoted_token() { check(&fixture().cases[6]); }
#[test]
fn react_search_half_width_label() { check(&fixture().cases[7]); }
#[test]
fn react_search_full_width_colon_label() { check(&fixture().cases[8]); }
#[test]
fn react_search_exact_or_partial() { check(&fixture().cases[9]); }
#[test]
fn react_search_empty_query() { check(&fixture().cases[10]); }
#[test]
fn react_search_year() { check(&fixture().cases[11]); }
#[test]
fn react_search_year_month() { check(&fixture().cases[12]); }
#[test]
fn react_search_date() { check(&fixture().cases[13]); }
#[test]
fn react_search_closed_range() { check(&fixture().cases[14]); }
#[test]
fn react_search_range_start() { check(&fixture().cases[15]); }
#[test]
fn react_search_range_end() { check(&fixture().cases[16]); }
#[test]
fn react_search_invalid_range() { check(&fixture().cases[17]); }
#[test]
fn react_search_invalid_date() { check(&fixture().cases[18]); }
#[test]
fn react_search_reversed_range() { check(&fixture().cases[19]); }
#[test]
fn react_search_amount_partial() { check(&fixture().cases[20]); }
#[test]
fn react_search_amount_exact() { check(&fixture().cases[21]); }

#[test] fn react_year_options_sorted_unique() { assert_eq!(create_year_options(&["2024-03-01", "2023-12-15", "2024-07-01"], |r| r), vec![option("2023", "2023年"),option("2024", "2024年")]); }
#[test] fn react_year_options_empty() { assert!(create_year_options::<&str>(&[], |r| r).is_empty()); }
#[test] fn react_unique_values() { assert_eq!(get_unique_values(&["特定", "NISA", "特定", ""], |r| r), vec!["特定", "NISA"]); }
#[test] fn react_unique_values_empty() { assert!(get_unique_values::<&str>(&[], |r| r).is_empty()); }
#[test] fn react_year_matches() { assert!(matches_year("2024-06-15", "2024")); }
#[test] fn react_year_mismatch() { assert!(!matches_year("2024-06-15", "2023")); }
#[test] fn react_month_matches() { assert!(matches_year_month("2024-03-15", "2024-03")); }
#[test] fn react_month_padded() { assert!(matches_year_month("2024-01-05", "2024-01")); assert!(!matches_year_month("2024-01-05", "2024-1")); }
#[test] fn react_month_mismatch() { assert!(!matches_year_month("2024-03-15", "2024-04")); }
fn option(value: &str, label: &str) -> SearchOption { SearchOption { value: value.into(), label: label.into() } }
