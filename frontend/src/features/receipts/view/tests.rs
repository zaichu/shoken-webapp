use super::cards::*;
use super::pickers::*;
use super::summary::*;
use super::tabs::*;
use super::workspace::utility_rail_id;
use crate::api::dto::{DividendSummary, DomesticStockSummary, MutualfundSummary};
use crate::features::receipts::filter::{
    column_order, filter_receipts, DateSegment, ReceiptSearch,
};
use crate::features::receipts::kind::{group_label, is_date_group_key, ColumnTier};
use crate::features::receipts::{
    ReceiptCell, ReceiptItem, ReceiptRow, ReceiptSummary, ReceiptTabData, ReceiptsTab,
};
use crate::support::row::Row::Saved;
use crate::testing::receipts::{dividends, domestic, funds};
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
    let groups = ReceiptsTab::Dividend.table_groups(&filtered, &rows, "9432 2024");
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].label, "NTT");
    assert_eq!(groups[0].summary, ["¥500", "¥100", "¥400"]);
    let filtered = filter_receipts(ReceiptsTab::Dividend, &rows, "9432");
    let groups = ReceiptsTab::Dividend.table_groups(&filtered, &rows, "9432");
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
        let groups = ReceiptsTab::Dividend.table_groups(&filtered, &rows, query);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].label, expected);
    }
}
#[test]
fn mutual_fund_name_groups_and_domestic_daily_groups() {
    let rows = funds();
    assert_eq!(
        ReceiptsTab::MutualFund.table_groups(&rows, &rows, "\"Alpha Fund A\"")[0].label,
        "Alpha Fund A"
    );
    let rows = domestic();
    let groups = ReceiptsTab::DomesticStock.table_groups(&rows, &rows, "7203");
    assert_eq!(groups[0].label, "2024年3月1日");
    assert!(groups[0].summary.is_empty());
}

#[test]
fn domestic_daily_summary_remains_for_multiple_rows_only() {
    let mut rows = domestic();
    let mut second = match &rows[0] {
        Saved(ReceiptItem::DomesticStock(row)) => row.clone(),
        _ => unreachable!(),
    };
    second.id = "second".to_string().into();
    rows.push(Saved(ReceiptItem::DomesticStock(second.clone())));
    second.id = "next-day".to_string().into();
    second.trade_date = "2024-03-02".into();
    rows.push(Saved(ReceiptItem::DomesticStock(second)));
    let groups = ReceiptsTab::DomesticStock.table_groups(&rows, &rows, "");
    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0].label, "2024年3月2日");
    assert_eq!(groups[0].rows.len(), 1);
    assert!(groups[0].summary.is_empty());
    assert_eq!(groups[1].label, "2024年3月1日");
    assert_eq!(groups[1].rows.len(), 2);
    assert_eq!(groups[1].summary, ["¥2,000", "¥406", "¥1,594"]);
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
fn next_tab_index_cycles_through_tabs() {
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
        let headers = tab.headers();
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
            tab.headers().len(),
            tab.column_widths().len(),
            "{tab:?}: 列幅の数がヘッダー数と一致しない"
        );
    }
    let rows = domestic();
    let order = column_order(ReceiptsTab::DomesticStock, &rows, "特定");
    let headers = ReceiptsTab::DomesticStock.headers();
    let widths = ReceiptsTab::DomesticStock.column_widths();
    let displayed: Vec<(&str, &str)> = order.iter().map(|&i| (headers[i], widths[i])).collect();
    let fields = ReceiptsTab::DomesticStock.card_fields();
    assert_eq!(displayed[0], (headers[fields.date], widths[fields.date]));
    assert_eq!(
        displayed[2],
        (headers[fields.account], widths[fields.account])
    );
    assert_eq!(displayed[3], (headers[fields.name], widths[fields.name]));
}

// 「表示項目は極力削らない」方針: 数量はできるだけ早い帯から出す。
// 配当金は列が少なく md(768px)帯でも入るため Md。国内株式・投資信託は金額列が多く
// md 帯で数量まで出すと銘柄名が潰れるため Wide(レールを畳んだ lg・xl から)にとどめる
#[test]
fn quantity_column_tiers_match_width_budget() {
    let index_of = |tab: ReceiptsTab| {
        tab.headers()
            .iter()
            .position(|header| *header == "数量")
            .expect("数量列がある")
    };
    assert_eq!(
        ReceiptsTab::Dividend.column_tiers()[index_of(ReceiptsTab::Dividend)],
        ColumnTier::Md
    );
    for tab in [ReceiptsTab::DomesticStock, ReceiptsTab::MutualFund] {
        assert_eq!(tab.column_tiers()[index_of(tab)], ColumnTier::Wide);
    }
}

// 段のクラスは帯ごとの表示を制御する CSS ルールのフック。Core だけクラス無しで、
// 残る段は互いに区別でき、CSS にも対応するルールが無いと隠れないまま全帯で出る
#[test]
fn non_core_tiers_have_distinct_class_backed_by_a_css_rule() {
    let css = include_str!("../../../../style/input.css");
    assert!(ColumnTier::Core.class().trim().is_empty());

    let classes: Vec<&str> = [ColumnTier::Md, ColumnTier::Wide, ColumnTier::Wider]
        .iter()
        .map(|tier| tier.class().trim())
        .collect();
    for class in &classes {
        assert!(!class.is_empty(), "段にクラスが無いと隠れない");
        assert!(
            css.contains(&format!(".{class} ")),
            "{class} の CSS ルールが無い"
        );
    }
    let mut unique = classes.clone();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(unique.len(), classes.len(), "段のクラスが区別できない");
}

// span_masks が `tier <= max` で集計見出しの結合幅を数えるため、宣言順の大小関係に依存する
#[test]
fn column_tier_declares_core_as_narrowest_then_widening() {
    assert!(ColumnTier::Core < ColumnTier::Md);
    assert!(ColumnTier::Md < ColumnTier::Wide);
    assert!(ColumnTier::Wide < ColumnTier::Wider);
}

// パネルは常時マウントし、トグルも取得状態にかかわらず出す。
// 開閉では表の列幅を変えないため、表示強制の分岐は持たない

// 開閉トグルの aria-controls が指す aside の id。空や重複だと ARIA の参照が効かない
#[test]
fn utility_rail_ids_are_non_empty_and_unique_per_tab() {
    let ids: Vec<String> = ReceiptsTab::ALL
        .iter()
        .map(|tab| utility_rail_id(*tab))
        .collect();
    for id in &ids {
        assert!(!id.trim().is_empty(), "id が空だと参照先が無い");
        assert!(
            !id.contains(char::is_whitespace),
            "{id} は id として使えない"
        );
    }
    let mut unique = ids.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(unique.len(), ReceiptsTab::ALL.len());
}

#[test]
fn card_row_data_maps_card_fields() {
    let rows = dividends();
    let cells = rows[0].cells();
    let order = column_order(ReceiptsTab::Dividend, &rows, "");
    let card = card_row_data(
        ReceiptsTab::Dividend,
        &cells,
        ReceiptsTab::Dividend.headers(),
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
            *negative
                == (is_profit_label(ReceiptsTab::Dividend, &detail.label) && is_negative_text(text))
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
        ReceiptsTab::Dividend,
        &cells,
        ReceiptsTab::Dividend.headers(),
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
    card_row_data(tab, &cells, tab.headers(), &order, tab.card_fields(), false)
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
    assert!(detail_negative(&card, "実現損益"));
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
    assert!(!detail_negative(&card, "税引後"));
}

#[test]
fn dividend_subtotal_is_not_profit() {
    assert!(!is_profit_label(ReceiptsTab::Dividend, "税引後"));
    assert!(is_profit_label(ReceiptsTab::DomesticStock, "税引後"));
    assert!(is_profit_label(ReceiptsTab::MutualFund, "税引後"));
    assert!(!is_profit_label(ReceiptsTab::DomesticStock, "税額"));
}

#[test]
fn negative_labeled_value_requires_profit_label_and_negative_text() {
    assert!(is_negative_labeled_value(
        ReceiptsTab::DomesticStock,
        "税引後",
        "-¥1,234"
    ));
    assert!(!is_negative_labeled_value(
        ReceiptsTab::DomesticStock,
        "税引後",
        "¥1,234"
    ));
    assert!(!is_negative_labeled_value(
        ReceiptsTab::DomesticStock,
        "税額",
        "-¥1,234"
    ));
    assert!(!is_negative_labeled_value(
        ReceiptsTab::Dividend,
        "税引後",
        "-¥1,234"
    ));
}

#[test]
fn card_details_link_security_code_and_copy_name() {
    let rows = dividends();
    let cells = rows[0].cells();
    let order = column_order(ReceiptsTab::Dividend, &rows, "");
    let card = card_row_data(
        ReceiptsTab::Dividend,
        &cells,
        ReceiptsTab::Dividend.headers(),
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
        ReceiptsTab::MutualFund,
        &cells,
        ReceiptsTab::MutualFund.headers(),
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
    let groups = ReceiptsTab::Dividend.table_groups(&rows, &rows, "");
    assert_eq!(groups[0].key, "2026-06");
    assert_eq!(groups[0].label, "2026年6月");
    let ids: Vec<Option<&str>> = groups
        .iter()
        .flat_map(|group| group.rows.iter().map(|(id, _, _)| id.as_deref()))
        .collect();
    assert_eq!(ids, [Some("new"), Some("other"), Some("old")]);
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
    let groups = ReceiptsTab::Dividend.table_groups(&rows, &rows, "9432");
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
    let groups = ReceiptsTab::DomesticStock.table_groups(&domestic_rows, &domestic_rows, "");
    assert_eq!(groups[0].rows.len(), 1);
    let fund_rows = funds();
    let groups = ReceiptsTab::MutualFund.table_groups(&fund_rows, &fund_rows, "");
    assert_eq!(groups[0].rows.len(), 1);
}

#[test]
fn displayed_tiers_returns_non_empty_for_each_tab() {
    use crate::features::receipts::filter::{column_order, promoted_column};
    use crate::features::receipts::view::table::displayed_tiers;

    for tab in ReceiptsTab::ALL {
        let rows = match tab {
            ReceiptsTab::Dividend => dividends(),
            ReceiptsTab::DomesticStock => domestic(),
            ReceiptsTab::MutualFund => funds(),
        };
        let order = column_order(tab, &rows, "");
        let promoted = promoted_column(tab, &rows, "");
        let tiers = displayed_tiers(tab, &order, promoted);
        assert_eq!(tiers.len(), order.len());
        assert!(
            !tiers.is_empty(),
            "{tab:?}: displayed_tiers should not be empty"
        );
        assert!(
            tiers.contains(&ColumnTier::Core),
            "{tab:?}: should have Core tier"
        );
    }
}

#[test]
fn displayed_tiers_promotes_column_to_core() {
    use crate::features::receipts::filter::{column_order, promoted_column};
    use crate::features::receipts::view::table::displayed_tiers;

    let rows = domestic();
    let tab = ReceiptsTab::DomesticStock;
    let order = column_order(tab, &rows, "特定");
    let promoted = promoted_column(tab, &rows, "特定");
    let tiers = displayed_tiers(tab, &order, promoted);
    if let Some(p) = promoted {
        // promoted は列 id なので、order 内の位置で tiers を引く
        let pos = order.iter().position(|&c| c == p).unwrap();
        assert_eq!(tiers[pos], ColumnTier::Core);
    }
}

#[test]
fn card_row_data_details_follow_column_reorder() {
    let rows = dividends();
    let query = "特定";
    let order = column_order(ReceiptsTab::Dividend, &rows, query);
    assert_eq!(order[1], 2);
    let cells = rows[0].cells();
    let card = card_row_data(
        ReceiptsTab::Dividend,
        &cells,
        ReceiptsTab::Dividend.headers(),
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

fn csv_source_user(id: &str) -> crate::api::dto::SessionUser {
    crate::api::dto::SessionUser {
        id: id.to_string(),
        email: format!("{id}@example.com"),
        name: None,
        picture_url: None,
    }
}

fn csv_source_store() -> crate::features::receipts::ReceiptsStore {
    use crate::features::receipts::filter::ReceiptSearch;
    use crate::features::receipts::{ReceiptsStore, ReceiptsTab};
    use crate::session::{Generation, SessionStore};
    use leptos::prelude::*;
    use std::collections::{HashMap, HashSet};

    let session = SessionStore::new();
    session.loaded.set(true);
    session.user.set(Some(csv_source_user("alice")));

    let active_tab = RwSignal::new(ReceiptsTab::Dividend);
    let visited = RwSignal::new(HashSet::from([ReceiptsTab::Dividend]));
    let cache = RwSignal::new(HashMap::new());
    let csv = RwSignal::new(HashMap::new());
    let csv_files = RwSignal::new(HashMap::new());
    let refresh_error = RwSignal::new(HashMap::new());
    let fetch_rev = RwSignal::new(HashMap::new());

    let fetch = Action::new_unsync(move |(generation, tab): &(Generation, ReceiptsTab)| {
        let _generation = *generation;
        let _tab = *tab;
        async move {
            let _ = _tab.fetch_list().await;
        }
    });

    ReceiptsStore {
        session,
        active_tab,
        search: RwSignal::new(ReceiptSearch::default()),
        expanded: RwSignal::new(HashSet::new()),
        mobile_summary_expanded: RwSignal::new(false),
        utility_rail_open: RwSignal::new(true),
        expanded_epoch: RwSignal::new(None),
        visited,
        cache,
        fetch,
        csv,
        csv_files,
        refresh_error,
        fetch_rev,
    }
}

#[test]
fn receipt_csv_source_constants_match_expected_values() {
    use super::panel::ReceiptCsvSource;
    use crate::ui::csv_section::CsvSource;
    use leptos::prelude::*;

    let owner = Owner::new();
    owner.with(|| {
        let store = csv_source_store();
        for tab in ReceiptsTab::ALL {
            let source = ReceiptCsvSource { store, tab };
            assert_eq!(source.input_id(), tab.csv_input_id());
            assert_eq!(source.save_action(), "追加で保存");
            assert_eq!(source.mode_label(), "追加保存");
            assert_eq!(source.toggle_testid(), "receipt-csv-toggle");
            assert_eq!(source.section_class(), "sm:rounded-t-xl");
        }
    });
}

#[test]
fn receipt_csv_source_input_disabled_reflects_store_state() {
    use super::panel::ReceiptCsvSource;
    use crate::ui::csv_section::CsvSource;
    use leptos::prelude::*;

    let owner = Owner::new();
    owner.with(|| {
        let source = ReceiptCsvSource {
            store: csv_source_store(),
            tab: ReceiptsTab::Dividend,
        };
        assert!(
            !source.input_disabled(),
            "authenticated and not busy -> enabled"
        );
    });
}

#[test]
fn receipt_csv_source_db_count_returns_zero_when_no_data() {
    use super::panel::ReceiptCsvSource;
    use crate::ui::csv_section::CsvSource;
    use leptos::prelude::*;

    let owner = Owner::new();
    owner.with(|| {
        let source = ReceiptCsvSource {
            store: csv_source_store(),
            tab: ReceiptsTab::Dividend,
        };
        assert_eq!(source.db_count(), 0);
    });
}

#[test]
fn receipt_csv_source_delete_disabled_reflects_state() {
    use super::panel::ReceiptCsvSource;
    use crate::support::csv_flow::CsvTabState;
    use crate::ui::csv_section::CsvSource;
    use leptos::prelude::*;

    let owner = Owner::new();
    owner.with(|| {
        let source = ReceiptCsvSource {
            store: csv_source_store(),
            tab: ReceiptsTab::Dividend,
        };

        let idle_state = CsvTabState::default();
        assert!(!source.delete_disabled(&idle_state), "idle -> not disabled");

        let saving_state = CsvTabState {
            saving: true,
            ..Default::default()
        };
        assert!(source.delete_disabled(&saving_state), "saving -> disabled");

        let deleting_state = CsvTabState {
            deleting: true,
            ..Default::default()
        };
        assert!(
            source.delete_disabled(&deleting_state),
            "deleting -> disabled"
        );
    });
}
