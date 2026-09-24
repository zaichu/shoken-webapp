use super::*;
use crate::dto::{Dividend, DomesticStock, Mutualfund};

pub(crate) fn dividends() -> Vec<ReceiptItem> {
    let base: Dividend = serde_json::from_value(serde_json::json!({"id":"old", "settlement_date":"2024-06-21", "product":"国内株式", "account":"特定", "security_code":"9432", "security_name":"日本電信電話", "unit_price":5, "shares":100, "dividends_before_tax":500, "taxes":100, "net_amount_received":400, "created_at":"", "updated_at":""})).unwrap();
    let mut latest = base.clone();
    latest.id = "new".into();
    latest.settlement_date = "2026-06-01".into();
    latest.security_name = "ＮＴＴ".into();
    let mut other = latest.clone();
    other.id = "other".into();
    other.security_code = "7203".into();
    other.security_name = "トヨタ自動車".into();
    other.account = "NISA".into();
    vec![
        ReceiptItem::Dividend(base),
        ReceiptItem::Dividend(latest),
        ReceiptItem::Dividend(other),
    ]
}
pub(crate) fn domestic() -> Vec<ReceiptItem> {
    let row: DomesticStock = serde_json::from_value(serde_json::json!({"id":"stock", "trade_date":"2024-03-01", "settlement_date":"2024-03-05", "security_code":"7203", "security_name":"トヨタ自動車", "account":"特定", "shares":100, "asked_price":2000, "proceeds":200000, "purchase_price":199000, "realized_profit_and_loss":1000, "taxes":888, "realized_profit_and_loss_after_tax":112, "created_at":"", "updated_at":""})).unwrap();
    vec![ReceiptItem::DomesticStock(row)]
}
pub(crate) fn funds() -> Vec<ReceiptItem> {
    let row: Mutualfund = serde_json::from_value(serde_json::json!({"id":"fund", "trade_date":"2024-03-01", "settlement_date":"2024-03-05", "fund_name":"Alpha Fund A", "account":"NISA", "shares":100, "exchange_rate":1, "cancellation_unit_price_yen":100, "cancellation_amount_yen":1000, "average_acquisition_price_yen":100, "realized_profit_and_loss":900, "taxes":0, "realized_profit_and_loss_after_tax":900, "created_at":"", "updated_at":""})).unwrap();
    vec![ReceiptItem::MutualFund(row)]
}
#[test]
fn dividend_filter_uses_exact_fields_and_all_amount_fields() {
    let rows = dividends();
    for query in [
        "9432 2024",
        "日本電信電話",
        "国内株式 特定 2024",
        "5 2024",
        "100 2024",
        "500 2024",
        "400 2024",
    ] {
        assert_eq!(
            filter_receipts(ReceiptsTab::Dividend, &rows, query),
            rows[..1],
            "{query}"
        );
    }
    assert!(filter_receipts(ReceiptsTab::Dividend, &rows, "943").is_empty());
}
#[test]
fn domestic_filter_uses_trade_date_and_only_react_amounts() {
    let rows = domestic();
    for query in [
        "7203",
        "トヨタ自動車",
        "特定",
        "2024-03-01",
        "100",
        "2000",
        "200000",
        "199000",
        "1000",
    ] {
        assert_eq!(
            filter_receipts(ReceiptsTab::DomesticStock, &rows, query),
            rows,
            "{query}"
        );
    }
    for query in ["2024-03-05", "888", "112", "トヨタ"] {
        assert!(
            filter_receipts(ReceiptsTab::DomesticStock, &rows, query).is_empty(),
            "{query}"
        );
    }
}
#[test]
fn mutual_fund_filter_is_exact_and_has_no_amount_search() {
    let rows = funds();
    for query in [
        "\"Alpha Fund A\"",
        "nisa",
        "2024-03-01",
        "2024-03-01..2024-03-02",
    ] {
        assert_eq!(
            filter_receipts(ReceiptsTab::MutualFund, &rows, query),
            rows,
            "{query}"
        );
    }
    for query in ["Alpha", "900", "2024-03-05"] {
        assert!(
            filter_receipts(ReceiptsTab::MutualFund, &rows, query).is_empty(),
            "{query}"
        );
    }
}
#[test]
fn whitespace_returns_all_three_tabs() {
    for (tab, rows) in [
        (ReceiptsTab::Dividend, dividends()),
        (ReceiptsTab::DomesticStock, domestic()),
        (ReceiptsTab::MutualFund, funds()),
    ] {
        assert_eq!(filter_receipts(tab, &rows, " \t　\u{feff}"), rows);
    }
}
#[test]
fn categories_use_latest_name_sorted_dates_and_react_tab_fields() {
    let result = search_categories(ReceiptsTab::Dividend, &dividends());
    assert!(result.dates);
    assert_eq!(
        result
            .securities
            .iter()
            .map(|o| o.label.as_str())
            .collect::<Vec<_>>(),
        ["7203: トヨタ自動車", "9432: ＮＴＴ"]
    );
    assert_eq!(result.products[0].value, "国内株式");
    assert_eq!(
        result
            .accounts
            .iter()
            .map(|o| o.value.as_str())
            .collect::<Vec<_>>(),
        ["特定", "NISA"]
    );
    assert_eq!(
        result
            .years
            .iter()
            .map(|o| o.value.as_str())
            .collect::<Vec<_>>(),
        ["2024", "2026"]
    );
    let stock = search_categories(ReceiptsTab::DomesticStock, &domestic());
    assert!(stock.products.is_empty());
    assert_eq!(stock.accounts[0].value, "特定");
    let fund = search_categories(ReceiptsTab::MutualFund, &funds());
    assert_eq!(fund.securities[0].value, "Alpha Fund A");
    assert!(fund.products.is_empty());
    assert!(fund.accounts.is_empty());
}
#[test]
fn category_selections_quote_spaces_and_combine_with_year() {
    let mut state = ReceiptSearch::new(true);
    state.select_quick(SearchKey::Securities, "Alpha Fund A".into());
    state.select_year("2024".into());
    assert_eq!(state.query, "2024 \"Alpha Fund A\"");
    assert_eq!(
        filter_receipts(ReceiptsTab::MutualFund, &funds(), &state.query),
        funds()
    );
    state.select_quick(SearchKey::Securities, "".into());
    assert_eq!(state.query, "2024");
}

#[test]
fn combined_query_uses_react_order_and_quotes_whitespace() {
    let queries = SelectedQueries {
        date: " 2026-06 ".into(),
        securities: "9433: KDDI".into(),
        years: "2025".into(),
        products: "国内 株式".into(),
        accounts: "特定・一般".into(),
    };
    assert_eq!(
        build_combined_query(&queries),
        "2026-06 \"9433: KDDI\" 2025 \"国内 株式\" 特定・一般"
    );
    assert_eq!(
        format_query_token("  Alpha \"Fund\"  "),
        "\"Alpha \\\"Fund\\\"\""
    );

    let mut state = ReceiptSearch::new(true);
    state.select_quick(SearchKey::Products, " ".into());
    assert!(!state.is_default());
}

#[test]
fn date_and_year_queries_keep_react_search_keys_separate() {
    let mut state = ReceiptSearch::new(true);
    state.select_year("2026".into());
    assert_eq!(state.query, "2026");
    assert!(state.selected_queries.years.is_empty());

    state.select_quick(SearchKey::Years, "2025".into());
    assert_eq!(state.query, "2026 2025");
}

#[test]
fn range_query_matches_react_empty_and_open_ranges() {
    assert_eq!(build_range_query("", ""), "");
    assert_eq!(build_range_query("2026-06-01", ""), "2026-06-01..");
    assert_eq!(build_range_query("", "2026-06-30"), "..2026-06-30");
    assert_eq!(
        build_range_query("2026-06-01", "2026-06-30"),
        "2026-06-01..2026-06-30"
    );
}

#[test]
fn quick_buttons_toggle_and_date_condition_precedes_other_conditions() {
    let mut state = ReceiptSearch::new(true);
    assert_eq!(state.date_segment, DateSegment::Year);

    state.select_quick(SearchKey::Securities, "9433".into());
    state.select_quick(SearchKey::Products, "国内株式".into());
    state.set_month("2026-06".into());
    assert_eq!(state.query, "2026-06 9433 国内株式");

    state.select_quick(SearchKey::Products, "国内株式".into());
    assert_eq!(state.query, "2026-06 9433");
    assert!(state.selected_queries.products.is_empty());
}

#[test]
fn changing_date_segment_only_clears_date_state() {
    let mut state = ReceiptSearch::new(true);
    state.select_quick(SearchKey::Accounts, "特定".into());
    state.select_year("2026".into());
    assert_eq!(state.query, "2026 特定");

    state.change_date_segment(DateSegment::Month);
    assert_eq!(state.query, "特定");
    assert_eq!(state.date_segment, DateSegment::Month);
    assert_eq!(state.date_inputs, DateInputs::default());
    assert_eq!(state.selected_queries.accounts, "特定");
}

#[test]
fn range_inputs_and_clear_match_react_state() {
    let mut state = ReceiptSearch::new(true);
    state.change_date_segment(DateSegment::Range);
    state.set_range_start("2026-06-01".into());
    assert_eq!(state.query, "2026-06-01..");
    state.set_range_end("2026-06-30".into());
    assert_eq!(state.query, "2026-06-01..2026-06-30");

    state.clear(false);
    assert!(state.query.is_empty());
    assert_eq!(state.selected_queries, SelectedQueries::default());
    assert_eq!(state.date_inputs, DateInputs::default());
    assert_eq!(state.date_segment, DateSegment::Month);
}
#[test]
fn tab_column_rules_use_whole_query_not_tokens() {
    assert_eq!(
        column_order(ReceiptsTab::Dividend, &dividends(), "特定"),
        [0, 2, 1, 3, 4, 5, 6, 7, 8, 9]
    );
    assert_eq!(
        column_order(ReceiptsTab::Dividend, &dividends(), "国内株式 特定"),
        (0..10).collect::<Vec<_>>()
    );
    assert_eq!(
        column_order(ReceiptsTab::DomesticStock, &domestic(), "特定"),
        [0, 1, 3, 2, 4, 5, 6, 7, 8, 9, 10]
    );
    assert_eq!(
        column_order(ReceiptsTab::MutualFund, &funds(), "NISA"),
        (0..10).collect::<Vec<_>>()
    );
}
