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
            truncated: false,
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

#[test]
fn next_tab_index_cycles_like_react_tablist() {
    assert_eq!(next_tab_index(0, "ArrowRight"), Some(1));
    assert_eq!(next_tab_index(2, "ArrowRight"), Some(0));
    assert_eq!(next_tab_index(0, "ArrowLeft"), Some(2));
    assert_eq!(next_tab_index(1, "ArrowLeft"), Some(0));
    assert_eq!(next_tab_index(1, "Home"), Some(0));
    assert_eq!(next_tab_index(0, "End"), Some(2));
    assert_eq!(next_tab_index(1, "Enter"), None);
    assert_eq!(next_tab_index(1, "Escape"), None);
}

#[test]
fn negative_text_detection_matches_formatted_values() {
    assert!(is_negative_text("¥ -1,234"));
    assert!(is_negative_text("-500"));
    assert!(is_negative_text("-1.5"));
    assert!(!is_negative_text("¥ 1,234"));
    assert!(!is_negative_text("¥ -0"));
    assert!(!is_negative_text("+3"));
    assert!(!is_negative_text(""));
    assert!(!is_negative_text("ＮＴＴ"));
    assert!(!is_negative_text("-"));
    assert!(!is_negative_text("12."));
    assert!(!is_negative_text("1.2.3"));
}

#[test]
fn short_date_strips_year_only_for_iso_formatted() {
    assert_eq!(short_date("2024/03/01"), "03/01");
    assert_eq!(short_date("-"), "-");
    assert_eq!(short_date("ＮＴＴ"), "ＮＴＴ");
    assert_eq!(short_date("03/01"), "03/01");
}

#[test]
fn card_fields_point_at_expected_columns() {
    let cases = [
        (
            ReceiptsTab::Dividend,
            ("銘柄名", "受取額", "入金日", "口座"),
        ),
        (
            ReceiptsTab::DomesticStock,
            ("銘柄名", "税引後", "約定日", "口座"),
        ),
        (
            ReceiptsTab::MutualFund,
            ("ファンド名", "税引損益", "約定日", "口座"),
        ),
    ];
    for (tab, expected) in cases {
        let fields = card_fields(tab);
        let headers = table_headers(tab);
        assert_eq!(
            (
                headers[fields.name],
                headers[fields.primary],
                headers[fields.date],
                headers[fields.account]
            ),
            expected
        );
    }
    assert!(matches!(
        dividends()[0].cells()[3],
        ReceiptCell::SecurityCode(_)
    ));
    assert!(matches!(
        domestic()[0].cells()[1],
        ReceiptCell::SecurityCode(_)
    ));
    assert!(!funds()[0]
        .cells()
        .iter()
        .any(|cell| matches!(cell, ReceiptCell::SecurityCode(_))));
}

#[test]
fn table_column_widths_match_headers_and_follow_column_order() {
    for tab in ReceiptsTab::ALL {
        assert_eq!(
            table_headers(tab).len(),
            table_column_widths(tab).len(),
            "{tab:?}: 列幅の数がヘッダー数と一致しない"
        );
    }
    let rows = domestic();
    let order = column_order(ReceiptsTab::DomesticStock, &rows, "特定");
    let headers = table_headers(ReceiptsTab::DomesticStock);
    let widths = table_column_widths(ReceiptsTab::DomesticStock);
    let displayed: Vec<(&str, &str)> = order.iter().map(|&i| (headers[i], widths[i])).collect();
    assert_eq!(displayed[0], ("約定日", "84px"));
    assert_eq!(displayed[1], ("銘柄コード", "72px"));
    assert_eq!(displayed[2], ("口座", "60px"));
    assert_eq!(displayed[3], ("銘柄名", "156px"));
}

#[test]
fn card_row_data_matches_react_card_fields() {
    let rows = dividends();
    let cells = rows[0].cells();
    let order = column_order(ReceiptsTab::Dividend, &rows, "");
    let card = card_row_data(
        "dividend:r:old".to_string(),
        &cells,
        table_headers(ReceiptsTab::Dividend),
        &order,
        card_fields(ReceiptsTab::Dividend),
    );
    assert_eq!(card.key, "dividend:r:old");
    assert_eq!(card.name, "日本電信電話");
    assert_eq!(card.amount, "¥ 400");
    assert!(!card.amount_negative);
    assert_eq!(card.date, "06/21");
    assert_eq!(card.account, "特定");
    assert_eq!(card.details.len(), 10);
    assert_eq!(card.details[0].label, "入金日");
    assert!(card.details.iter().all(|detail| match &detail.value {
        CardDetailValue::Text { text, negative } => *negative == is_negative_text(text),
        CardDetailValue::SecurityCode(_) | CardDetailValue::CopyName { .. } => true,
    }));
}

#[test]
fn card_details_link_security_code_and_copy_name() {
    let rows = dividends();
    let cells = rows[0].cells();
    let order = column_order(ReceiptsTab::Dividend, &rows, "");
    let card = card_row_data(
        String::new(),
        &cells,
        table_headers(ReceiptsTab::Dividend),
        &order,
        card_fields(ReceiptsTab::Dividend),
    );
    assert!(card.details.iter().any(|detail| matches!(
        &detail.value,
        CardDetailValue::SecurityCode(raw) if raw == "9432"
    )));
    assert!(card.details.iter().any(|detail| matches!(
        &detail.value,
        CardDetailValue::CopyName { display, copy }
            if display == "日本電信電話" && copy == "日本電信電話(9432)"
    )));

    let rows = funds();
    let cells = rows[0].cells();
    let order = column_order(ReceiptsTab::MutualFund, &rows, "");
    let card = card_row_data(
        String::new(),
        &cells,
        table_headers(ReceiptsTab::MutualFund),
        &order,
        card_fields(ReceiptsTab::MutualFund),
    );
    assert!(!card
        .details
        .iter()
        .any(|detail| matches!(&detail.value, CardDetailValue::SecurityCode(_))));
    assert!(card.details.iter().any(|detail| matches!(
        &detail.value,
        CardDetailValue::CopyName { display, copy } if display == copy
    )));
}

#[test]
fn table_groups_carry_group_key_and_row_ids() {
    let rows = dividends();
    let groups = table_groups(ReceiptsTab::Dividend, &rows, &rows, "");
    assert_eq!(groups[0].key, "2026-06");
    assert_eq!(groups[0].label, "2026年6月");
    let ids: Vec<&str> = groups
        .iter()
        .flat_map(|group| group.rows.iter().map(|(id, _)| id.as_str()))
        .collect();
    assert_eq!(ids, ["new", "other", "old"]);
}

#[test]
fn card_key_separates_idless_rows_by_position() {
    let cells = dividends()[0].cells();
    let key = card_key("dividend", "", &cells, 0);
    assert!(key.starts_with("dividend:p:"));
    assert!(key.contains("日本電信電話"));
    assert_eq!(key, card_key("dividend", "", &cells, 0));
    assert_ne!(key, card_key("dividend", "", &cells, 1));
    assert_ne!(key, card_key("mutualfund", "", &cells, 0));
    assert_eq!(card_key("dividend", "old", &cells, 0), "dividend:r:old");
    assert_eq!(card_key("dividend", "old", &cells, 1), "dividend:r:old");
}

#[test]
fn idless_rows_keep_unfiltered_positions_as_card_ordinals() {
    let mut first = dividends()[0].clone();
    let mut second = dividends()[0].clone();
    let mut removed = dividends()[2].clone();
    let with_id = dividends()[1].clone();
    for item in [&mut first, &mut second, &mut removed] {
        if let ReceiptItem::Dividend(row) = item {
            row.id.clear();
        }
    }
    let ordinals = idless_row_ordinals(&[removed, with_id, first.clone(), second.clone()]);
    assert_eq!(ordinals.len(), 2);
    let content = first
        .cells()
        .iter()
        .map(cell_text)
        .collect::<Vec<_>>()
        .join("\u{1f}");
    // 先頭行を絞り込みで除いても残る行のカードキーは変わらない
    let positions: Vec<usize> = ordinals[&content].iter().copied().collect();
    assert_eq!(positions, [2, 3]);
    let cells = first.cells();
    assert_ne!(
        card_key("dividend", "", &cells, positions[0]),
        card_key("dividend", "", &cells, positions[1]),
    );
}

#[test]
fn security_code_acceptance_matches_react_regex() {
    assert!(is_security_code("9432"));
    assert!(is_security_code("BRK.B"));
    assert!(!is_security_code(""));
    assert!(!is_security_code("任天堂"));
    assert!(!is_security_code("9432:メモ"));
}

#[test]
fn card_row_data_details_follow_column_reorder() {
    let rows = dividends();
    let query = "特定";
    let order = column_order(ReceiptsTab::Dividend, &rows, query);
    assert_eq!(order[1], 2);
    let cells = rows[0].cells();
    let card = card_row_data(
        String::new(),
        &cells,
        table_headers(ReceiptsTab::Dividend),
        &order,
        card_fields(ReceiptsTab::Dividend),
    );
    assert_eq!(card.details[1].label, "口座");
    assert_eq!(card.name, "日本電信電話");
    assert_eq!(card.account, "特定");
}
