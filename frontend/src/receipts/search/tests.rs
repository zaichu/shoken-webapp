use super::*;
use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Clone, Deserialize)]
struct Item {
    id: String,
    code: String,
    name: String,
    category: String,
    date: String,
    amount: Decimal,
}
#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct Config {
    string_fields: Vec<String>,
    partial_string_fields: Vec<String>,
    date_field: Option<String>,
    year_search: bool,
    year_month_search: bool,
    date_search: bool,
    date_range_search: bool,
    amount_fields: Vec<String>,
}
#[derive(Deserialize)]
struct Case {
    name: String,
    data: Vec<Item>,
    query: String,
    config: Config,
    expected_ids: Vec<String>,
}
#[derive(Deserialize)]
struct Fixture {
    cases: Vec<Case>,
}
fn string_getter(field: &str) -> fn(&Item) -> &str {
    match field {
        "code" => |r| &r.code,
        "name" => |r| &r.name,
        "category" => |r| &r.category,
        _ => panic!("unknown field"),
    }
}
fn fixture() -> Fixture {
    serde_json::from_str(include_str!("../../../tests/fixtures/receipts/search.json")).unwrap()
}
fn check(case: &Case) {
    let c = &case.config;
    let config = FilterConfig {
        string_fields: if c.string_fields.is_empty() {
            None
        } else {
            Some(c.string_fields.iter().map(|s| string_getter(s)).collect())
        },
        partial_string_fields: if c.partial_string_fields.is_empty() {
            None
        } else {
            Some(
                c.partial_string_fields
                    .iter()
                    .map(|s| string_getter(s))
                    .collect(),
            )
        },
        date_field: c
            .date_field
            .as_ref()
            .map(|_| (|r: &Item| r.date.as_str()) as fn(&Item) -> &str),
        year_search: c.year_search,
        year_month_search: c.year_month_search,
        date_search: c.date_search,
        date_range_search: c.date_range_search,
        amount_fields: if c.amount_fields.is_empty() {
            None
        } else {
            Some(
                c.amount_fields
                    .iter()
                    .map(|_| (|r: &Item| r.amount) as fn(&Item) -> Decimal)
                    .collect(),
            )
        },
    };
    let actual: Vec<_> = filter_by_config(&case.data, &case.query, &config)
        .iter()
        .map(|r| r.id.clone())
        .collect();
    assert_eq!(actual, case.expected_ids, "{}", case.name);
}
#[test]
fn shared_search_fixtures() {
    for case in fixture().cases {
        check(&case);
    }
}
#[test]
fn react_search_exact_match() {
    check(&fixture().cases[0]);
}
#[test]
fn react_search_exact_rejects_partial() {
    check(&fixture().cases[1]);
}
#[test]
fn react_search_partial_match() {
    check(&fixture().cases[2]);
}
#[test]
fn react_search_case_insensitive() {
    check(&fixture().cases[3]);
}
#[test]
fn react_search_multiple_partial_fields() {
    check(&fixture().cases[4]);
}
#[test]
fn react_search_and_tokens() {
    check(&fixture().cases[5]);
}
#[test]
fn react_search_quoted_token() {
    check(&fixture().cases[6]);
}
#[test]
fn react_search_half_width_label() {
    check(&fixture().cases[7]);
}
#[test]
fn react_search_full_width_colon_label() {
    check(&fixture().cases[8]);
}
#[test]
fn react_search_exact_or_partial() {
    check(&fixture().cases[9]);
}
#[test]
fn react_search_empty_query() {
    check(&fixture().cases[10]);
}
#[test]
fn react_search_year() {
    check(&fixture().cases[11]);
}
#[test]
fn react_search_year_month() {
    check(&fixture().cases[12]);
}
#[test]
fn react_search_date() {
    check(&fixture().cases[13]);
}
#[test]
fn react_search_closed_range() {
    check(&fixture().cases[14]);
}
#[test]
fn react_search_range_start() {
    check(&fixture().cases[15]);
}
#[test]
fn react_search_range_end() {
    check(&fixture().cases[16]);
}
#[test]
fn react_search_invalid_range() {
    check(&fixture().cases[17]);
}
#[test]
fn react_search_invalid_date() {
    check(&fixture().cases[18]);
}
#[test]
fn react_search_reversed_range() {
    check(&fixture().cases[19]);
}
#[test]
fn react_search_amount_partial() {
    check(&fixture().cases[20]);
}
#[test]
fn react_search_amount_exact() {
    check(&fixture().cases[21]);
}

#[test]
fn react_year_options_sorted_unique() {
    assert_eq!(
        create_year_options(&["2024-03-01", "2023-12-15", "2024-07-01"], |r| r),
        vec![option("2023", "2023年"), option("2024", "2024年")]
    );
}
#[test]
fn react_year_options_empty() {
    assert!(create_year_options::<&str>(&[], |r| r).is_empty());
}
#[test]
fn react_unique_values() {
    assert_eq!(
        get_unique_values(&["特定", "NISA", "特定", ""], |r| r),
        vec!["特定", "NISA"]
    );
}
#[test]
fn react_unique_values_empty() {
    assert!(get_unique_values::<&str>(&[], |r| r).is_empty());
}
#[test]
fn react_year_matches() {
    assert!(matches_year("2024-06-15", "2024"));
}
#[test]
fn react_year_mismatch() {
    assert!(!matches_year("2024-06-15", "2023"));
}
#[test]
fn react_month_matches() {
    assert!(matches_year_month("2024-03-15", "2024-03"));
}
#[test]
fn react_month_padded() {
    assert!(matches_year_month("2024-01-05", "2024-01"));
    assert!(!matches_year_month("2024-01-05", "2024-1"));
}
#[test]
fn react_month_mismatch() {
    assert!(!matches_year_month("2024-03-15", "2024-04"));
}
fn option(value: &str, label: &str) -> SearchOption {
    SearchOption {
        value: value.into(),
        label: label.into(),
    }
}

fn naive_valid_iso_date(value: &str) -> bool {
    let parts: Vec<&str> = value.split('-').collect();
    if parts.len() != 3
        || parts[0].len() != 4
        || parts[1].len() != 2
        || parts[2].len() != 2
        || !parts
            .iter()
            .all(|part| part.bytes().all(|b| b.is_ascii_digit()))
    {
        return false;
    }
    let (Ok(year), Ok(month), Ok(day)) = (
        parts[0].parse::<i32>(),
        parts[1].parse::<u8>(),
        parts[2].parse::<u8>(),
    ) else {
        return false;
    };
    let Ok(month) = time::Month::try_from(month) else {
        return false;
    };
    time::Date::from_calendar_date(year, month, day).is_ok()
}

fn string_item_config() -> FilterConfig<String> {
    FilterConfig {
        string_fields: None,
        partial_string_fields: Some(vec![|item: &String| item.as_str()]),
        date_field: None,
        year_search: false,
        year_month_search: false,
        date_search: false,
        date_range_search: false,
        amount_fields: None,
    }
}

proptest::proptest! {
    #[test]
    fn prop_parse_tokens_simple(query in "[a-zA-Z0-9 \t]+") {
        let tokens = parse_search_tokens(&query);
        let expected: Vec<String> = query
            .split(is_js_whitespace)
            .filter(|token| !token.is_empty())
            .map(|token| token.to_lowercase())
            .collect();
        proptest::prop_assert_eq!(tokens, expected);
    }

    #[test]
    fn prop_parse_tokens_never_empty(query in ".*") {
        for token in parse_search_tokens(&query) {
            proptest::prop_assert!(!token.is_empty());
            proptest::prop_assert_eq!(token.clone(), token.to_lowercase());
        }
    }

    #[test]
    fn prop_year_prefix_match(
        date in "[ -~]{0,20}",
        year in "[0-9]{4}",
        year_month in "[0-9]{4}-[0-9]{2}",
    ) {
        proptest::prop_assert_eq!(
            matches_year(&date, &year),
            date.len() >= 4 && date.starts_with(&year)
        );
        proptest::prop_assert_eq!(
            matches_year_month(&date, &year_month),
            date.len() >= 7 && date.starts_with(&year_month)
        );
        proptest::prop_assert!(matches_year(&year, &year));
        proptest::prop_assert!(matches_year_month(&year_month, &year_month));
    }

    #[test]
    fn prop_matches_date_range_naive(
        date in "[0-9]{4}-[0-9]{2}-[0-9]{2}",
        start_year in 1990i32..2031i32,
        start_month in 0u32..=13u32,
        start_day in 0u32..=32u32,
        end_year in 1990i32..2031i32,
        end_month in 0u32..=13u32,
        end_day in 0u32..=32u32,
        omit_start in proptest::bool::ANY,
        omit_end in proptest::bool::ANY,
    ) {
        let start = format!("{start_year:04}-{start_month:02}-{start_day:02}");
        let end = format!("{end_year:04}-{end_month:02}-{end_day:02}");
        let query = format!(
            "{}..{}",
            if omit_start { "" } else { &start },
            if omit_end { "" } else { &end },
        );
        let effective_start = if omit_start { "" } else { start.as_str() };
        let effective_end = if omit_end { "" } else { end.as_str() };

        let valid_start = effective_start.is_empty() || naive_valid_iso_date(effective_start);
        let valid_end = effective_end.is_empty() || naive_valid_iso_date(effective_end);
        let expected = if (effective_start.is_empty() && effective_end.is_empty())
            || !valid_start
            || !valid_end
            || (!effective_start.is_empty()
                && !effective_end.is_empty()
                && effective_start > effective_end)
        {
            false
        } else if effective_start.is_empty() {
            date.as_str() <= effective_end
        } else if effective_end.is_empty() {
            date.as_str() >= effective_start
        } else {
            date.as_str() >= effective_start && date.as_str() <= effective_end
        };
        proptest::prop_assert_eq!(
            matches_date_range(&date, &query),
            expected,
            "date={} query={}",
            date,
            query
        );
    }

    #[test]
    fn prop_year_options_naive(dates in proptest::collection::vec("[ -~]{0,12}", 0..16usize)) {
        let options = create_year_options(&dates, |date| date.as_str());
        let mut seen = HashSet::new();
        let mut years = Vec::new();
        for date in &dates {
            if date.len() >= 4 {
                let year = &date[..4];
                if seen.insert(year) {
                    years.push(year.to_string());
                }
            }
        }
        years.sort();
        let expected: Vec<SearchOption> = years
            .into_iter()
            .map(|year| SearchOption {
                value: year.clone(),
                label: format!("{year}年"),
            })
            .collect();
        proptest::prop_assert_eq!(options, expected);
    }

    #[test]
    fn prop_get_unique_values_naive(
        values in proptest::collection::vec(".*", 0..16usize),
    ) {
        let actual = get_unique_values(&values, |v| v.as_str());
        let mut seen = HashSet::new();
        let mut expected = Vec::new();
        for value in &values {
            if !value.trim_matches(is_js_whitespace).is_empty() && seen.insert(value.clone()) {
                expected.push(value.clone());
            }
        }
        proptest::prop_assert_eq!(actual, expected);
    }

    #[test]
    fn prop_filter_partial_match_subsequence(
        items in proptest::collection::vec("[a-zA-Z0-9]{0,12}", 0..16usize),
        query in "[a-z0-9]{1,4}",
    ) {
        let result = filter_by_config(&items, &query, &string_item_config());
        let expected: Vec<&String> = items
            .iter()
            .filter(|item| item.to_lowercase().contains(&query))
            .collect();
        proptest::prop_assert_eq!(result, expected);
    }

    #[test]
    fn prop_filter_blank_query_returns_all(
        items in proptest::collection::vec("[a-z0-9]{0,8}", 0..8usize),
        query in "[ \t　]{1,6}",
    ) {
        let result = filter_by_config(&items, &query, &string_item_config());
        proptest::prop_assert_eq!(result.len(), items.len());
    }

    #[test]
    fn prop_is_valid_iso_date_matches_naive_model(
        (y, m, d) in (0i32..10_000, 0i32..15, 0i32..35),
        raw in "[0-9-]{0,12}",
    ) {
        for input in [format!("{y:04}-{m:02}-{d:02}"), raw] {
            proptest::prop_assert_eq!(
                is_valid_iso_date(&input),
                naive_valid_iso_date(&input),
                "input={}",
                input
            );
        }
    }
}

#[test]
fn iso_date_rejects_misplaced_separators() {
    for malformed in ["2024-03001", "2024X03-01", "2024-03X01", "202403-01-"] {
        assert!(!is_valid_iso_date(malformed), "{malformed}");
    }
}

#[test]
fn iso_date_leap_year_rules() {
    assert!(is_valid_iso_date("2024-02-29"));
    assert!(is_valid_iso_date("2000-02-29"));
    assert!(!is_valid_iso_date("1900-02-29"));
    assert!(!is_valid_iso_date("2023-02-29"));
    assert!(!is_valid_iso_date("2024-02-30"));
}

#[test]
fn label_token_with_empty_code_is_kept_verbatim() {
    assert_eq!(parse_search_tokens(":"), vec![":".to_string()]);
    assert_eq!(parse_search_tokens("9432:"), vec!["9432".to_string()]);
    assert_eq!(parse_search_tokens("9432："), vec!["9432".to_string()]);
    // ラベル部に数字・小文字以外を含む場合はラベルとみなさずコロン付きのまま残す
    assert_eq!(parse_search_tokens("7-:"), vec!["7-:".to_string()]);
}

#[test]
fn parse_search_tokens_splits_unquoted_tokens() {
    assert_eq!(
        parse_search_tokens("  9432  ＮＴＴ "),
        vec!["9432".to_string(), "ｎｔｔ".to_string()]
    );
}

/// JS の行終端子は改行2種と U+2028/U+2029 だけを含む
#[test]
fn js_line_terminator_matches_spec() {
    for c in ['\n', '\r', '\u{2028}', '\u{2029}'] {
        assert!(is_js_line_terminator(c));
    }
    for c in [' ', '\t', '\u{000B}', '\u{2027}', '\u{2030}'] {
        assert!(!is_js_line_terminator(c));
    }
}
