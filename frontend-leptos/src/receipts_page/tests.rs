use super::*;
use crate::dto::{DividendSummary, DomesticStockSummary, MutualfundSummary};
use crate::receipts_filter::{
    filter_receipts,
    tests::{dividends, domestic, funds},
};
use rust_decimal_macros::dec;

#[test]
fn headers_use_api_when_empty_and_filtered_client_for_search_and_whitespace() {
    let cases = [
        (
            ReceiptsTab::Dividend,
            dividends(),
            ReceiptSummary::Dividend(DividendSummary {
                total_dividends_before_tax: dec!(9999),
                total_taxes: dec!(9999),
                total_net_amount_received: dec!(9999),
            }),
            "9432 2024",
            [dec!(500), dec!(100), dec!(400)],
        ),
        (
            ReceiptsTab::DomesticStock,
            domestic(),
            ReceiptSummary::DomesticStock(DomesticStockSummary {
                total_realized_profit_and_loss: dec!(9999),
                total_taxes: dec!(9999),
                total_realized_profit_and_loss_after_tax: dec!(9999),
            }),
            "7203",
            [dec!(1000), dec!(203), dec!(797)],
        ),
        (
            ReceiptsTab::MutualFund,
            funds(),
            ReceiptSummary::MutualFund(MutualfundSummary {
                total_realized_profit_and_loss: dec!(9999),
                total_taxes: dec!(9999),
                total_realized_profit_and_loss_after_tax: dec!(9999),
            }),
            "NISA",
            [dec!(900), dec!(0), dec!(900)],
        ),
    ];
    for (tab, rows, summary, query, expected) in cases {
        let mut data = ReceiptTabData {
            rows: rows.clone(),
            summary: Some(summary),
        };
        assert_eq!(
            header_summary(tab, &data, "", false)
                .iter()
                .map(|v| v.1)
                .collect::<Vec<_>>(),
            [dec!(9999); 3]
        );
        data.rows = filter_receipts(tab, &rows, query);
        assert_eq!(
            header_summary(tab, &data, query, false)
                .iter()
                .map(|v| v.1)
                .collect::<Vec<_>>(),
            expected
        );
        data.rows = filter_receipts(tab, &rows, "　");
        assert_ne!(header_summary(tab, &data, "　", false)[0].1, dec!(9999));
        data.rows = filter_receipts(tab, &rows, "該当なし");
        assert_eq!(
            header_summary(tab, &data, "該当なし", false)
                .iter()
                .map(|v| v.1)
                .collect::<Vec<_>>(),
            [Decimal::ZERO; 3]
        );
        assert_eq!(header_summary(tab, &data, "", false)[0].1, dec!(9999));
    }
}
#[test]
fn dividend_search_groups_by_latest_name_from_unfiltered_rows() {
    let rows = dividends();
    let filtered = filter_receipts(ReceiptsTab::Dividend, &rows, "9432 2024");
    let groups = table_groups(ReceiptsTab::Dividend, &filtered, &rows, "9432 2024");
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].label, "ＮＴＴ");
    assert_eq!(groups[0].summary, ["¥ 500", "¥ 100", "¥ 400"]);
    let filtered = filter_receipts(ReceiptsTab::Dividend, &rows, "9432");
    let groups = table_groups(ReceiptsTab::Dividend, &filtered, &rows, "9432");
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].rows.len(), 2);
    assert_eq!(groups[0].summary, ["¥ 1,000", "¥ 200", "¥ 800"]);
}
#[test]
fn dividend_product_and_account_group_keys_follow_priority() {
    let rows = dividends();
    for (query, expected) in [
        ("国内株式 特定", "国内株式"),
        ("特定", "特定"),
        ("2024", "2024年6月"),
    ] {
        let filtered = filter_receipts(ReceiptsTab::Dividend, &rows, query);
        let groups = table_groups(ReceiptsTab::Dividend, &filtered, &rows, query);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].label, expected);
    }
}
#[test]
fn mutual_fund_name_groups_and_domestic_daily_groups_match_react() {
    let rows = funds();
    assert_eq!(
        table_groups(ReceiptsTab::MutualFund, &rows, &rows, "\"Alpha Fund A\"")[0].label,
        "Alpha Fund A"
    );
    let rows = domestic();
    let groups = table_groups(ReceiptsTab::DomesticStock, &rows, &rows, "7203");
    assert_eq!(groups[0].label, "2024年3月1日");
    assert_eq!(groups[0].summary, ["¥ 1,000", "¥ 203", "¥ 797"]);
}
#[test]
fn hyphenated_instrument_names_are_not_formatted_as_dates() {
    assert_eq!(group_label("eMAXIS-Slim"), "eMAXIS-Slim");
    assert_eq!(group_label("Alpha-Fund-A"), "Alpha-Fund-A");
}
#[test]
fn next_year_option_index_cycles_between_first_and_last() {
    assert_eq!(next_year_option_index(Some(0), 3, "ArrowDown"), Some(1));
    assert_eq!(next_year_option_index(Some(2), 3, "ArrowDown"), Some(0));
    assert_eq!(next_year_option_index(Some(2), 3, "ArrowRight"), Some(0));
    assert_eq!(next_year_option_index(Some(0), 3, "ArrowUp"), Some(2));
    assert_eq!(next_year_option_index(Some(1), 3, "ArrowUp"), Some(0));
    assert_eq!(next_year_option_index(Some(0), 3, "ArrowLeft"), Some(2));
}
#[test]
fn next_year_option_index_home_end_and_no_focused_option() {
    assert_eq!(next_year_option_index(Some(2), 3, "Home"), Some(0));
    assert_eq!(next_year_option_index(Some(0), 3, "End"), Some(2));
    assert_eq!(next_year_option_index(None, 3, "ArrowDown"), Some(0));
    assert_eq!(next_year_option_index(None, 3, "ArrowUp"), Some(2));
}
#[test]
fn next_year_option_index_returns_none_without_options_or_for_other_keys() {
    assert_eq!(next_year_option_index(Some(0), 0, "ArrowDown"), None);
    assert_eq!(next_year_option_index(None, 0, "End"), None);
    assert_eq!(next_year_option_index(Some(0), 3, "Enter"), None);
    assert_eq!(next_year_option_index(Some(0), 3, "Escape"), None);
}
