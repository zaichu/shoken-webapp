use super::cards::*;
use super::groups::*;
use super::pickers::*;
use super::summary::*;
use super::table::*;
use super::tabs::*;
use crate::api::dto::{DividendSummary, DomesticStockSummary, MutualfundSummary};
use crate::features::receipts::filter::{
    column_order, filter_receipts,
    tests::{dividends, domestic, funds},
    DateSegment, ReceiptSearch,
};
use crate::features::receipts::kind::{group_label, is_date_group_key};
use crate::features::receipts::{
    ReceiptCell, ReceiptItem, ReceiptRow, ReceiptSummary, ReceiptTabData, ReceiptsTab,
};
use crate::support::row::Row::Saved;
use crate::ui::card::StatTone;
use rust_decimal::Decimal;
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
fn header_tones_mark_negative_profit_only() {
    let tones = |profit: Decimal, taxes: Decimal, after_tax: Decimal| {
        let data = ReceiptTabData {
            rows: vec![],
            summary: Some(ReceiptSummary::DomesticStock(DomesticStockSummary {
                total_realized_profit_and_loss: profit,
                total_taxes: taxes,
                total_realized_profit_and_loss_after_tax: after_tax,
            })),
            truncated: false,
        };
        header_summary(ReceiptsTab::DomesticStock, &data, "", false)
            .iter()
            .map(|v| v.2)
            .collect::<Vec<_>>()
    };
    // 損益を持つ項目(前・税引)が負のときだけ Loss、0・正は Neutral
    assert_eq!(
        tones(dec!(-100), dec!(0), dec!(100)),
        [StatTone::Loss, StatTone::Neutral, StatTone::Neutral]
    );
    assert_eq!(
        tones(dec!(0), dec!(0), dec!(-50)),
        [StatTone::Neutral, StatTone::Neutral, StatTone::Loss]
    );
}
#[test]
fn dividend_search_groups_by_latest_name_from_unfiltered_rows() {
    let rows = dividends();
    let filtered = filter_receipts(ReceiptsTab::Dividend, &rows, "9432 2024");
    let groups = table_groups(ReceiptsTab::Dividend, &filtered, &rows, "9432 2024");
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].label, "NTT");
    assert_eq!(groups[0].summary, ["¥500", "¥100", "¥400"]);
    let filtered = filter_receipts(ReceiptsTab::Dividend, &rows, "9432");
    let groups = table_groups(ReceiptsTab::Dividend, &filtered, &rows, "9432");
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].rows.len(), 2);
    assert_eq!(groups[0].summary, ["¥1,000", "¥200", "¥800"]);
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
    assert_eq!(groups[0].summary, ["¥1,000", "¥203", "¥797"]);
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
    assert!(is_negative_text("-¥1,234"));
    assert!(is_negative_text("¥ -1,234"));
    assert!(is_negative_text("-500"));
    assert!(is_negative_text("-1.5"));
    assert!(!is_negative_text("¥1,234"));
    assert!(!is_negative_text("¥ 1,234"));
    assert!(!is_negative_text("-¥0"));
    assert!(!is_negative_text("¥ -0"));
    assert!(!is_negative_text("+3"));
    assert!(!is_negative_text(""));
    assert!(!is_negative_text("ＮＴＴ"));
    assert!(!is_negative_text("-"));
    assert!(!is_negative_text("12."));
    assert!(!is_negative_text("1.2.3"));
    // f64 としてパースできても桁チェックを通らない表記は負数扱いしない
    assert!(!is_negative_text("-1e5"));
    assert!(!is_negative_text("-inf"));
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
        (ReceiptsTab::Dividend, ("銘柄名", "入金日", "口座")),
        (ReceiptsTab::DomesticStock, ("銘柄名", "約定日", "口座")),
        (ReceiptsTab::MutualFund, ("ファンド名", "約定日", "口座")),
    ];
    for (tab, expected) in cases {
        let fields = tab.card_fields();
        let headers = table_headers(tab);
        assert_eq!(
            (
                headers[fields.name],
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
    assert_eq!(displayed[0], ("約定日", "96px"));
    assert_eq!(displayed[1], ("銘柄コード", "88px"));
    assert_eq!(displayed[2], ("口座", "76px"));
    assert_eq!(displayed[3], ("銘柄名", ""));
}

#[test]
fn card_row_data_matches_react_card_fields() {
    let rows = dividends();
    let cells = rows[0].cells();
    let order = column_order(ReceiptsTab::Dividend, &rows, "");
    let card = card_row_data(
        &cells,
        table_headers(ReceiptsTab::Dividend),
        &order,
        ReceiptsTab::Dividend.card_fields(),
        false,
    );
    assert!(matches!(
        &card.name,
        CardDetailValue::CopyName { display, .. } if display == "日本電信電話"
    ));
    assert_eq!(card.date, "06/21");
    assert_eq!(card.account, "特定");
    // 見出し(銘柄名・入金日・口座)と重複させず、残り7列を表の列順で入れる
    assert_eq!(card.details.len(), 7);
    assert_eq!(card.details[0].label, "商品");
    assert!(card
        .details
        .iter()
        .all(|detail| !["入金日", "口座", "銘柄名"].contains(&detail.label.as_str())));
    assert!(card.details.iter().all(|detail| match &detail.value {
        CardDetailValue::Text { text, negative } => {
            *negative == (is_profit_label(&detail.label) && is_negative_text(text))
        }
        CardDetailValue::SecurityCode(_) | CardDetailValue::CopyName { .. } => true,
    }));
}

#[test]
fn card_date_keeps_year_when_group_is_not_year_month() {
    let rows = dividends();
    let cells = rows[0].cells();
    let order = column_order(ReceiptsTab::Dividend, &rows, "");
    let card = card_row_data(
        &cells,
        table_headers(ReceiptsTab::Dividend),
        &order,
        ReceiptsTab::Dividend.card_fields(),
        true,
    );
    assert_eq!(card.date, "2024/06/21");
}

fn detail_negative(card: &CardRowData, label: &str) -> bool {
    card.details
        .iter()
        .find(|detail| detail.label == label)
        .is_some_and(|detail| matches!(detail.value, CardDetailValue::Text { negative: true, .. }))
}

fn card_for(tab: ReceiptsTab, row: ReceiptRow) -> CardRowData {
    let rows = vec![row];
    let cells = rows[0].cells();
    let order = column_order(tab, &rows, "");
    card_row_data(&cells, table_headers(tab), &order, tab.card_fields(), false)
}

#[test]
fn negative_tax_and_dividend_stay_neutral() {
    let mut stock = match domestic().remove(0) {
        Saved(ReceiptItem::DomesticStock(row)) => row,
        _ => unreachable!(),
    };
    stock.realized_profit_and_loss = rust_decimal::Decimal::from(-1000);
    stock.taxes = rust_decimal::Decimal::from(-203);
    stock.realized_profit_and_loss_after_tax = rust_decimal::Decimal::from(-797);
    let card = card_for(
        ReceiptsTab::DomesticStock,
        Saved(ReceiptItem::DomesticStock(stock)),
    );
    assert!(detail_negative(&card, "損益"));
    assert!(!detail_negative(&card, "税額"));

    let mut dividend = match dividends().remove(0) {
        Saved(ReceiptItem::Dividend(row)) => row,
        _ => unreachable!(),
    };
    dividend.taxes = rust_decimal::Decimal::from(-100);
    dividend.net_amount_received = rust_decimal::Decimal::from(-50);
    let card = card_for(
        ReceiptsTab::Dividend,
        Saved(ReceiptItem::Dividend(dividend)),
    );
    assert!(!detail_negative(&card, "税額"));
    assert!(!detail_negative(&card, "受取額"));
}

#[test]
fn dividend_subtotal_is_not_profit() {
    assert!(!summary_is_profit(ReceiptsTab::Dividend, "税引後"));
    assert!(summary_is_profit(ReceiptsTab::DomesticStock, "税引後"));
    assert!(summary_is_profit(ReceiptsTab::MutualFund, "税引損益"));
    assert!(!summary_is_profit(ReceiptsTab::DomesticStock, "税額"));
}

#[test]
fn card_details_link_security_code_and_copy_name() {
    let rows = dividends();
    let cells = rows[0].cells();
    let order = column_order(ReceiptsTab::Dividend, &rows, "");
    let card = card_row_data(
        &cells,
        table_headers(ReceiptsTab::Dividend),
        &order,
        ReceiptsTab::Dividend.card_fields(),
        false,
    );
    assert!(card.details.iter().any(|detail| matches!(
        &detail.value,
        CardDetailValue::SecurityCode(raw) if raw == "9432"
    )));
    // 銘柄名は見出し側でコピーできるため格子には入れない
    assert!(matches!(
        &card.name,
        CardDetailValue::CopyName { display, copy }
            if display == "日本電信電話" && copy == "日本電信電話(9432)"
    ));
    assert!(!card
        .details
        .iter()
        .any(|detail| matches!(&detail.value, CardDetailValue::CopyName { .. })));

    let rows = funds();
    let cells = rows[0].cells();
    let order = column_order(ReceiptsTab::MutualFund, &rows, "");
    let card = card_row_data(
        &cells,
        table_headers(ReceiptsTab::MutualFund),
        &order,
        ReceiptsTab::MutualFund.card_fields(),
        false,
    );
    assert!(!card
        .details
        .iter()
        .any(|detail| matches!(&detail.value, CardDetailValue::SecurityCode(_))));
    assert!(matches!(
        &card.name,
        CardDetailValue::CopyName { display, copy } if display == copy
    ));
}

#[test]
fn table_groups_carry_group_key_and_row_ids() {
    let rows = dividends();
    let groups = table_groups(ReceiptsTab::Dividend, &rows, &rows, "");
    assert_eq!(groups[0].key, "2026-06");
    assert_eq!(groups[0].label, "2026年6月");
    let ids: Vec<Option<&str>> = groups
        .iter()
        .flat_map(|group| group.rows.iter().map(|(id, _, _)| id.as_deref()))
        .collect();
    assert_eq!(ids, [Some("new"), Some("other"), Some("old")]);
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
    assert_eq!(kpi_value_color(StatTone::Loss), "text-negative");
    assert_eq!(kpi_value_color(StatTone::Neutral), "text-text");
}

#[test]
fn summary_and_empty_hint_labels_match_tabs() {
    assert_eq!(
        ReceiptsTab::Dividend.summary_labels(),
        ["配当金", "税額", "税引後"]
    );
    assert_eq!(
        ReceiptsTab::DomesticStock.summary_labels(),
        ["損益", "税額", "税引後"]
    );
    assert_eq!(
        ReceiptsTab::MutualFund.summary_labels(),
        ["実現損益", "税額", "税引損益"]
    );
    assert_eq!(
        ReceiptsTab::Dividend.empty_hint(),
        "配当金明細をCSVで追加してください"
    );
    assert_eq!(
        ReceiptsTab::DomesticStock.empty_hint(),
        "国内株式明細をCSVで追加してください"
    );
    assert_eq!(
        ReceiptsTab::MutualFund.empty_hint(),
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
    assert!(is_date_group_key("2026-06"));
    assert!(is_date_group_key("2024-03-05"));
    assert!(!is_date_group_key("日本電信電話"));
    assert!(!is_date_group_key("SBI証券"));
    assert!(!is_date_group_key("2026-6"));
    assert_eq!(group_label("2026-06"), "2026年6月");
    assert_eq!(group_label("2024-03-05"), "2024年3月5日");
    assert_eq!(group_label("特定"), "特定");
    assert_eq!(group_label("2026-6"), "2026-6");
}

#[test]
fn latest_name_uses_newest_settlement_per_code() {
    let mut rows = dividends();
    let mut same_date = match &rows[1] {
        Saved(ReceiptItem::Dividend(row)) => row.clone(),
        _ => panic!("dividend row"),
    };
    same_date.security_name = "別名".into();
    rows.push(Saved(ReceiptItem::Dividend(same_date)));
    let groups = table_groups(ReceiptsTab::Dividend, &rows, &rows, "9432");
    assert_eq!(groups[0].label, "NTT");
}

#[test]
fn cells_normalize_full_width_security_names() {
    let rows = dividends();
    assert!(rows[1].cells().iter().any(|cell| matches!(
        cell,
        ReceiptCell::InstrumentName { name, .. } if name == "NTT"
    )));
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
        &cells,
        table_headers(ReceiptsTab::Dividend),
        &order,
        ReceiptsTab::Dividend.card_fields(),
        false,
    );
    // 口座は前に出ても見出し側の項目なので格子には入らない
    assert_eq!(card.details[0].label, "商品");
    assert!(card.details.iter().all(|detail| detail.label != "口座"));
    assert!(matches!(
        &card.name,
        CardDetailValue::CopyName { display, .. } if display == "日本電信電話"
    ));
    assert_eq!(card.account, "特定");
}
