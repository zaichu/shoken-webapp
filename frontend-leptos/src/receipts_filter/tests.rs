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
    let mut state = ReceiptSearch::default();
    state.select(0, "Alpha Fund A".into());
    state.select(3, "2024".into());
    assert_eq!(state.query, "\"Alpha Fund A\" 2024");
    assert_eq!(
        filter_receipts(ReceiptsTab::MutualFund, &funds(), &state.query),
        funds()
    );
    state.select(0, "".into());
    assert_eq!(state.query, "2024");
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
