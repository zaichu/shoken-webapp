use super::*;
use crate::receipts_search::get_unique_values;
use rust_decimal_macros::dec;

#[derive(Clone)]
struct Item {
    code: &'static str,
    name: &'static str,
    category: &'static str,
    account: &'static str,
    date: &'static str,
    amount: Decimal,
    tax: Decimal,
}
fn item(code: &'static str, name: &'static str, date: &'static str) -> Item {
    Item {
        code,
        name,
        date,
        category: "",
        account: "",
        amount: dec!(0),
        tax: dec!(0),
    }
}
fn option(value: &str, label: &str) -> SearchOption {
    SearchOption {
        value: value.into(),
        label: label.into(),
    }
}
fn options(data: &[Item], prefix: bool, dates: bool) -> Vec<SearchOption> {
    create_search_options(
        data,
        |r| r.code,
        |r| r.name,
        prefix,
        if dates { Some(|r: &Item| r.date) } else { None },
    )
}
#[test]
fn search_options_with_prefix_and_empty_code() {
    assert_eq!(
        options(
            &[
                item("1234", "テスト1", ""),
                item("5678", "テスト2", ""),
                item("", "テスト3", "")
            ],
            true,
            false
        ),
        vec![
            option("1234", "1234: テスト1"),
            option("5678", "5678: テスト2"),
            option("テスト3", "テスト3")
        ]
    );
}
#[test]
fn search_options_without_prefix() {
    assert_eq!(
        options(
            &[item("1234", "テスト1", ""), item("5678", "テスト2", "")],
            false,
            false
        ),
        vec![option("1234", "テスト1"), option("5678", "テスト2")]
    );
}
#[test]
fn search_options_deduplicate() {
    assert_eq!(
        options(
            &[
                item("1234", "テスト1", ""),
                item("1234", "テスト1", ""),
                item("5678", "テスト2", "")
            ],
            true,
            false
        ),
        vec![
            option("1234", "1234: テスト1"),
            option("5678", "5678: テスト2")
        ]
    );
}
fn renamed() -> Vec<Item> {
    vec![
        item("9432", "日本電信電話", "2024-06-21"),
        item("9432", "ＮＴＴ", "2026-06-01"),
        item("1234", "テスト", "2023-01-01"),
    ]
}
#[test]
fn latest_name_old_first() {
    assert_eq!(
        options(&renamed(), true, true),
        vec![
            option("1234", "1234: テスト"),
            option("9432", "9432: ＮＴＴ")
        ]
    );
}
#[test]
fn latest_name_new_first() {
    let mut rows = renamed();
    rows.reverse();
    assert_eq!(
        options(&rows, true, true),
        vec![
            option("1234", "1234: テスト"),
            option("9432", "9432: ＮＴＴ")
        ]
    );
}
#[test]
fn latest_name_date_strings() {
    assert_eq!(
        options(&renamed()[..2], true, true),
        vec![option("9432", "9432: ＮＴＴ")]
    );
}
#[test]
fn no_date_keeps_first_name() {
    assert_eq!(
        options(&renamed()[..2], true, false),
        vec![option("9432", "9432: 日本電信電話")]
    );
}
#[test]
fn same_date_keeps_first_label() {
    let rows = [
        item("9432", "旧名", "2024-06-21"),
        item("9432", "新名", "2024-06-21"),
    ];
    assert_eq!(
        options(&rows, true, true),
        vec![option("9432", "9432: 旧名")]
    );
}
#[test]
fn invalid_initial_date_is_replaced() {
    let mut rows = renamed();
    rows[0].date = "not-a-date";
    assert_eq!(
        options(&rows[..2], true, true),
        vec![option("9432", "9432: ＮＴＴ")]
    );
}
#[test]
fn unique_values_preserve_whitespace_and_distinct_raw_values() {
    assert_eq!(
        get_unique_values(&[" 特定 ", "特定", " 特定 ", "\u{feff}", "　", ""], |r| r),
        vec![" 特定 ", "特定"]
    );
}

const BASE: &[usize] = &[0, 1, 2, 3, 4, 5];
fn column_data() -> Vec<Item> {
    let mut a = item("", "テスト1", "");
    a.category = "商品A";
    a.account = "特定口座";
    let mut b = item("", "テスト2", "");
    b.category = "商品B";
    b.account = "NISA";
    vec![a, b]
}
const ACCOUNT: ColumnReorderRule<Item> = ColumnReorderRule {
    column_key: 4,
    matches: |r, q| r.account.to_lowercase().contains(q),
};
#[test]
fn columns_empty_query() {
    assert_eq!(
        reorder_columns_by_search(BASE, &column_data(), "", &[ACCOUNT], 1),
        BASE
    );
}
#[test]
fn columns_move_match_forward() {
    assert_eq!(
        reorder_columns_by_search(BASE, &column_data(), "特定", &[ACCOUNT], 1),
        [0, 4, 1, 2, 3, 5]
    );
}
#[test]
fn columns_two_fixed() {
    assert_eq!(
        reorder_columns_by_search(BASE, &column_data(), "特定", &[ACCOUNT], 2),
        [0, 1, 4, 2, 3, 5]
    );
}
#[test]
fn columns_priority() {
    let rules = [
        ColumnReorderRule {
            column_key: 3,
            matches: |r: &Item, q| r.category.to_lowercase().contains(q),
        },
        ACCOUNT,
    ];
    assert_eq!(
        reorder_columns_by_search(BASE, &column_data(), "商品a", &rules, 1),
        [0, 3, 1, 2, 4, 5]
    );
}
#[test]
fn columns_no_match() {
    assert_eq!(
        reorder_columns_by_search(BASE, &column_data(), "存在しない", &[ACCOUNT], 1),
        BASE
    );
}
#[test]
fn columns_missing_key_skips_rule() {
    let rules = [
        ColumnReorderRule {
            column_key: 99,
            matches: |_: &Item, _| true,
        },
        ACCOUNT,
    ];
    assert_eq!(
        reorder_columns_by_search(BASE, &column_data(), "特定", &rules, 1),
        [0, 4, 1, 2, 3, 5]
    );
}

fn summary_data() -> Vec<Item> {
    [
        ("2023-01-01", "A", 100),
        ("2023-01-02", "A", 200),
        ("2023-02-01", "B", 300),
        ("2023-02-15", "C", 400),
    ]
    .into_iter()
    .map(|(date, category, amount)| {
        let mut r = item("", "", date);
        r.category = category;
        r.amount = Decimal::from(amount);
        r.tax = r.amount / dec!(10);
        r
    })
    .collect()
}
const FIELDS: &[fn(&Item) -> Decimal] = &[|r| r.amount, |r| r.tax];
fn summary(filter: &str, amount: i32, tax: i32) -> GroupSummary {
    GroupSummary {
        filter: filter.into(),
        values: vec![amount.into(), tax.into()],
    }
}
#[test]
fn groups_month_descending() {
    assert_eq!(
        group_and_summarize(&summary_data(), |r| r.date[..7].into(), FIELDS, true),
        vec![summary("2023-02", 700, 70), summary("2023-01", 300, 30)]
    );
}
#[test]
fn groups_category() {
    let mut rows = summary_data();
    rows.reverse();
    assert_eq!(
        group_and_summarize(&rows, |r| r.category.into(), FIELDS, false),
        vec![
            summary("A", 300, 30),
            summary("B", 300, 30),
            summary("C", 400, 40)
        ]
    );
}
fn check_group_order(desc: bool, expected: &[&str]) {
    let source = summary_data();
    let rows = vec![
        source[2].clone(),
        source[0].clone(),
        source[3].clone(),
        source[1].clone(),
    ];
    let original: Vec<_> = rows.iter().map(|r| r.date).collect();
    let groups = group_and_summarize(&rows, |r| r.category.into(), FIELDS, desc);
    assert_eq!(
        groups.iter().map(|r| r.filter.as_str()).collect::<Vec<_>>(),
        expected
    );
    assert_eq!(
        groups.iter().find(|r| r.filter == "A"),
        Some(&summary("A", 300, 30))
    );
    assert_eq!(rows.iter().map(|r| r.date).collect::<Vec<_>>(), original);
}
#[test]
fn groups_input_order_independent_ascending() {
    check_group_order(false, &["A", "B", "C"]);
}
#[test]
fn groups_input_order_independent_descending() {
    check_group_order(true, &["C", "B", "A"]);
}
#[test]
fn groups_search_query() {
    assert_eq!(
        group_and_summarize(&summary_data(), |_| "search".into(), FIELDS, false),
        vec![summary("search", 1000, 100)]
    );
}
