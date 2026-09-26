use crate::list_search::{is_js_whitespace, parse_search_tokens};

pub struct GroupKeyRule<'a, T> {
    pub test: fn(&T, &str) -> bool,
    pub key_fn: &'a dyn Fn(&T) -> String,
}

pub fn create_group_key_fn<'a, T>(
    search_query: &'a str,
    date_key_fn: fn(&T) -> String,
    rules: &'a [GroupKeyRule<'a, T>],
) -> Box<dyn Fn(&T) -> String + 'a> {
    if search_query.is_empty() {
        return Box::new(move |item: &T| date_key_fn(item));
    }

    let tokens = parse_search_tokens(search_query);

    Box::new(move |item: &T| {
        for rule in rules {
            if tokens.iter().any(|t| (rule.test)(item, t)) {
                return (rule.key_fn)(item);
            }
        }
        date_key_fn(item)
    })
}

pub fn derive_security_code_from_query<T>(
    query: &str,
    data: &[T],
    code_getter: fn(&T) -> &str,
    name_getter: fn(&T) -> &str,
) -> String {
    if query.is_empty() {
        return String::new();
    }

    let trimmed_start = query.trim_start_matches(is_js_whitespace);
    let code_end = trimmed_start
        .char_indices()
        .take_while(|(_, c)| c.is_ascii_alphanumeric())
        .map(|(index, c)| index + c.len_utf8())
        .last()
        .unwrap_or(0);
    let label_match = (code_end > 0)
        .then(|| {
            let suffix = trimmed_start[code_end..].trim_start_matches(is_js_whitespace);
            suffix
                .starts_with([':', '：'])
                .then(|| trimmed_start[..code_end].to_ascii_lowercase())
        })
        .flatten();

    if let Some(lower_code) = label_match {
        if let Some(item) = data
            .iter()
            .find(|item| code_getter(item).to_lowercase() == lower_code)
        {
            return code_getter(item).to_string();
        }
    }

    let tokens = parse_search_tokens(query);
    if let Some(item) = data.iter().find(|item| {
        tokens.iter().any(|t| {
            code_getter(item).to_lowercase() == *t || name_getter(item).to_lowercase() == *t
        })
    }) {
        return code_getter(item).to_string();
    }

    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct TestItem {
        code: String,
        name: String,
        product: String,
        date: String,
    }

    fn date_key_fn(item: &TestItem) -> String {
        format!("{}-{}", &item.date[..4], &item.date[5..7])
    }

    fn test_rule(item: &TestItem, t: &str) -> bool {
        item.code.to_lowercase() == t || item.name.to_lowercase() == t
    }

    fn test_key_fn(item: &TestItem) -> String {
        item.name.clone()
    }

    fn product_rule(item: &TestItem, t: &str) -> bool {
        item.product.to_lowercase() == t
    }

    fn product_key_fn(item: &TestItem) -> String {
        item.product.clone()
    }

    const RULES: &[GroupKeyRule<'static, TestItem>] = &[
        GroupKeyRule {
            test: test_rule,
            key_fn: &test_key_fn,
        },
        GroupKeyRule {
            test: product_rule,
            key_fn: &product_key_fn,
        },
    ];

    fn create_item() -> TestItem {
        TestItem {
            code: "7203".to_string(),
            name: "トヨタ自動車".to_string(),
            product: "国内株式".to_string(),
            date: "2024-03-15".to_string(),
        }
    }

    #[test]
    fn empty_query_returns_date_key() {
        let fn_ = create_group_key_fn("", date_key_fn, RULES);
        assert_eq!(fn_(&create_item()), "2024-03");
    }

    #[test]
    fn code_match_returns_name_group() {
        let fn_ = create_group_key_fn("7203", date_key_fn, RULES);
        assert_eq!(fn_(&create_item()), "トヨタ自動車");
    }

    #[test]
    fn name_match_returns_name_group() {
        let fn_ = create_group_key_fn("トヨタ自動車", date_key_fn, RULES);
        assert_eq!(fn_(&create_item()), "トヨタ自動車");
    }

    #[test]
    fn product_match_returns_product_group() {
        let fn_ = create_group_key_fn("国内株式", date_key_fn, RULES);
        assert_eq!(fn_(&create_item()), "国内株式");
    }

    #[test]
    fn no_match_returns_date_key() {
        let fn_ = create_group_key_fn("2024", date_key_fn, RULES);
        assert_eq!(fn_(&create_item()), "2024-03");
    }

    #[test]
    fn and_query_first_match_wins() {
        let fn_ = create_group_key_fn("7203 国内株式", date_key_fn, RULES);
        assert_eq!(fn_(&create_item()), "トヨタ自動車");
    }

    #[test]
    fn partial_match_rule_works() {
        let partial_rules = &[GroupKeyRule {
            test: |item: &TestItem, t: &str| item.name.to_lowercase().contains(t),
            key_fn: &|item: &TestItem| item.name.clone(),
        }];
        let fn_ = create_group_key_fn("トヨタ", date_key_fn, partial_rules);
        assert_eq!(fn_(&create_item()), "トヨタ自動車");
    }

    fn security_data() -> Vec<TestItem> {
        vec![
            TestItem {
                code: "7203".to_string(),
                name: "トヨタ自動車".to_string(),
                product: "".to_string(),
                date: String::new(),
            },
            TestItem {
                code: "9984".to_string(),
                name: "ソフトバンクグループ".to_string(),
                product: "".to_string(),
                date: String::new(),
            },
        ]
    }

    fn code_getter(item: &TestItem) -> &str {
        &item.code
    }
    fn name_getter(item: &TestItem) -> &str {
        &item.name
    }

    #[test]
    fn derive_security_code_empty_query() {
        assert_eq!(
            derive_security_code_from_query("", &security_data(), code_getter, name_getter),
            ""
        );
    }

    #[test]
    fn derive_security_code_half_width_colon() {
        assert_eq!(
            derive_security_code_from_query(
                "7203: トヨタ自動車",
                &security_data(),
                code_getter,
                name_getter
            ),
            "7203"
        );
    }

    #[test]
    fn derive_security_code_full_width_colon() {
        assert_eq!(
            derive_security_code_from_query(
                "7203：トヨタ自動車",
                &security_data(),
                code_getter,
                name_getter
            ),
            "7203"
        );
    }

    #[test]
    fn derive_security_code_exact_code_match() {
        assert_eq!(
            derive_security_code_from_query("7203", &security_data(), code_getter, name_getter),
            "7203"
        );
    }

    #[test]
    fn derive_security_code_exact_name_match() {
        assert_eq!(
            derive_security_code_from_query(
                "トヨタ自動車",
                &security_data(),
                code_getter,
                name_getter
            ),
            "7203"
        );
    }

    #[test]
    fn derive_security_code_no_match() {
        assert_eq!(
            derive_security_code_from_query("任天堂", &security_data(), code_getter, name_getter),
            ""
        );
    }

    #[test]
    fn derive_security_code_spaces_around_colon() {
        assert_eq!(
            derive_security_code_from_query(
                "  9984 ：ソフトバンクグループ",
                &security_data(),
                code_getter,
                name_getter
            ),
            "9984"
        );
    }

    #[test]
    fn derive_security_code_label_not_in_data() {
        assert_eq!(
            derive_security_code_from_query(
                "9999: 存在しない銘柄",
                &security_data(),
                code_getter,
                name_getter
            ),
            ""
        );
    }

    #[test]
    fn derive_security_code_case_insensitive() {
        let mixed_case = vec![TestItem {
            code: "ABC1".to_string(),
            name: "テスト銘柄".to_string(),
            product: "".to_string(),
            date: String::new(),
        }];
        assert_eq!(
            derive_security_code_from_query(
                "abc1: テスト銘柄",
                &mixed_case,
                code_getter,
                name_getter
            ),
            "ABC1"
        );
    }

    #[test]
    fn derive_security_code_treats_feff_as_whitespace() {
        assert_eq!(
            derive_security_code_from_query(
                "\u{FEFF}7203\u{FEFF}:トヨタ自動車",
                &security_data(),
                code_getter,
                name_getter
            ),
            "7203"
        );
    }

    #[test]
    fn derive_security_code_does_not_treat_u0085_as_whitespace() {
        assert_eq!(
            derive_security_code_from_query(
                "\u{0085}7203:トヨタ自動車",
                &security_data(),
                code_getter,
                name_getter
            ),
            ""
        );
    }
}
