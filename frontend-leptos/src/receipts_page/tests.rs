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
        .flat_map(|group| group.rows.iter().map(|(id, _, _)| id.as_str()))
        .collect();
    assert_eq!(ids, ["new", "other", "old"]);
}

#[test]
fn card_key_separates_idless_rows_by_position() {
    let raw_key = dividends()[0].raw_key();
    let key = card_key("dividend", "", &raw_key, 0);
    assert!(key.starts_with("dividend:p:"));
    assert!(key.contains("日本電信電話"));
    assert_eq!(key, card_key("dividend", "", &raw_key, 0));
    assert_ne!(key, card_key("dividend", "", &raw_key, 1));
    assert_ne!(key, card_key("mutualfund", "", &raw_key, 0));
    assert_eq!(card_key("dividend", "old", &raw_key, 0), "dividend:r:old");
    assert_eq!(card_key("dividend", "old", &raw_key, 1), "dividend:r:old");
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
    // 先頭行を絞り込みで除いても残る行のカードキーは変わらない
    let positions: Vec<usize> = ordinals[&first.raw_key()].iter().copied().collect();
    assert_eq!(positions, [2, 3]);
    assert_ne!(
        card_key("dividend", "", &first.raw_key(), positions[0]),
        card_key("dividend", "", &first.raw_key(), positions[1]),
    );
}

#[test]
fn idless_rows_with_rounding_identical_display_stay_separate() {
    // 表示上の数量は両方「1.00」に丸められるが、絞り込みは丸め前の値で行う
    let mut first = dividends()[0].clone();
    let mut second = dividends()[0].clone();
    for item in [&mut first, &mut second] {
        if let ReceiptItem::Dividend(row) = item {
            row.id.clear();
        }
    }
    if let ReceiptItem::Dividend(row) = &mut first {
        row.shares = dec!(1.001);
    }
    if let ReceiptItem::Dividend(row) = &mut second {
        row.shares = dec!(1.002);
    }
    assert_eq!(first.cells(), second.cells());
    assert_ne!(first.raw_key(), second.raw_key());

    let all_rows = vec![first, second];
    let keys_for = |groups: &[TableGroup]| {
        let mut ordinals = idless_row_ordinals(&all_rows);
        groups
            .iter()
            .flat_map(|group| group.rows.iter())
            .map(|(id, raw_key, _)| {
                card_key(
                    "dividend",
                    id,
                    raw_key,
                    card_ordinal(&mut ordinals, id, raw_key),
                )
            })
            .collect::<Vec<_>>()
    };

    // カード生成と同じく table_groups -> キュー照合 -> card_key で通す
    let groups = table_groups(ReceiptsTab::Dividend, &all_rows, &all_rows, "");
    let group_keys: Vec<&str> = groups
        .iter()
        .flat_map(|group| group.rows.iter().map(|(_, raw_key, _)| raw_key.as_str()))
        .collect();
    assert_eq!(group_keys.len(), 2);
    assert_ne!(group_keys[0], group_keys[1]);
    let keys_before = keys_for(&groups);
    let [key_first, key_second] = keys_before.as_slice() else {
        panic!("2行分のカードキーがある");
    };
    assert_ne!(key_first, key_second);
    let key_second = key_second.clone();

    // 先の行だけが外れる絞り込みの後でも、残った行のカードキーは変わらない
    let filtered = filter_receipts(ReceiptsTab::Dividend, &all_rows, "1.002");
    assert_eq!(filtered.len(), 1);
    let groups = table_groups(ReceiptsTab::Dividend, &filtered, &all_rows, "1.002");
    let keys_after = keys_for(&groups);
    assert_eq!(keys_after, [key_second]);
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
fn kpi_styles_match_tone() {
    assert_eq!(kpi_card_bg("emerald"), "border-teal-200 bg-teal-50");
    assert_eq!(kpi_card_bg("red"), "border-rose-100 bg-rose-50");
    assert_eq!(kpi_card_bg("other"), "border-slate-200 bg-white");
    assert_eq!(kpi_value_color("emerald"), "text-teal-700");
    assert_eq!(kpi_value_color("red"), "text-red-500");
    assert_eq!(kpi_value_color("other"), "text-slate-800");
}

#[test]
fn summary_and_empty_hint_labels_match_tabs() {
    assert_eq!(
        summary_labels(ReceiptsTab::Dividend),
        ["配当金", "税額", "受取額"]
    );
    assert_eq!(
        summary_labels(ReceiptsTab::DomesticStock),
        ["損益", "税額", "税引後"]
    );
    assert_eq!(
        summary_labels(ReceiptsTab::MutualFund),
        ["実現損益", "税額", "税引損益"]
    );
    assert_eq!(
        empty_hint(ReceiptsTab::Dividend),
        "配当金明細をCSVで追加してください"
    );
    assert_eq!(
        empty_hint(ReceiptsTab::DomesticStock),
        "国内株式明細をCSVで追加してください"
    );
    assert_eq!(
        empty_hint(ReceiptsTab::MutualFund),
        "投資信託明細をCSVで追加してください"
    );
}

#[test]
fn date_input_helpers_follow_field_and_value() {
    let mut state = ReceiptSearch::new(true);
    state.date_inputs.month_value = "2026-06".into();
    state.date_inputs.range_start = "2026-06-01".into();
    assert_eq!(date_input_value(&state, DateInputField::Month), "2026-06");
    assert_eq!(
        date_input_value(&state, DateInputField::RangeStart),
        "2026-06-01"
    );
    assert_eq!(date_input_type(DateInputField::Month), "month");
    assert_eq!(date_input_type(DateInputField::Date), "date");
    assert_eq!(date_input_type(DateInputField::RangeEnd), "date");
    assert_eq!(format_date_input_label("", "年月"), "年月");
    assert_eq!(format_date_input_label("2026-06-01", "x"), "2026/06/01");
}

#[test]
fn visible_date_segment_falls_back_to_month_without_years() {
    let mut state = ReceiptSearch::new(false);
    state.date_segment = DateSegment::Year;
    assert_eq!(visible_date_segment(&state, false), DateSegment::Month);
    assert_eq!(visible_date_segment(&state, true), DateSegment::Year);
    state.date_segment = DateSegment::Date;
    assert_eq!(visible_date_segment(&state, false), DateSegment::Date);
}

#[test]
fn group_label_formats_iso_keys_only() {
    assert_eq!(group_label("2026-06"), "2026年6月");
    assert_eq!(group_label("2024-03-05"), "2024年3月5日");
    assert_eq!(group_label("特定"), "特定");
    assert_eq!(group_label("2026-6"), "2026-6");
}

#[test]
fn latest_name_uses_newest_settlement_per_code() {
    let mut rows = dividends();
    let mut same_date = match &rows[1] {
        ReceiptItem::Dividend(row) => row.clone(),
        _ => panic!("dividend row"),
    };
    same_date.security_name = "別名".into();
    rows.push(ReceiptItem::Dividend(same_date));
    let groups = table_groups(ReceiptsTab::Dividend, &rows, &rows, "9432");
    assert_eq!(groups[0].label, "ＮＴＴ");
}

#[test]
fn group_rows_contain_only_matching_rows() {
    let domestic_rows = domestic();
    let groups = table_groups(
        ReceiptsTab::DomesticStock,
        &domestic_rows,
        &domestic_rows,
        "",
    );
    assert_eq!(groups[0].rows.len(), 1);
    let fund_rows = funds();
    let groups = table_groups(ReceiptsTab::MutualFund, &fund_rows, &fund_rows, "");
    assert_eq!(groups[0].rows.len(), 1);
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
