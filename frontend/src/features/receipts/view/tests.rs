use super::cards::*;
use super::summary::*;
use super::tabs::*;
use crate::api::dto::{DividendSummary, DomesticStockSummary, MutualfundSummary};
use crate::features::receipts::filter::{column_order, filter_receipts};
use crate::features::receipts::kind::{ColumnTier, group_label, is_date_group_key};
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
            "9910 2024",
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
            "9904",
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
            total: None,
            summary: Some(summary),
            truncated: false,
        };
        assert_eq!(
            header_summary(tab, &data, "")
                .iter()
                .map(|v| v.1)
                .collect::<Vec<_>>(),
            [dec!(9999); 3]
        );
        data.rows = filter_receipts(tab, &rows, query);
        assert_eq!(
            header_summary(tab, &data, query)
                .iter()
                .map(|v| v.1)
                .collect::<Vec<_>>(),
            expected
        );
        data.rows = filter_receipts(tab, &rows, "　");
        assert_ne!(header_summary(tab, &data, "　")[0].1, dec!(9999));
        data.rows = filter_receipts(tab, &rows, "該当なし");
        assert_eq!(
            header_summary(tab, &data, "該当なし")
                .iter()
                .map(|v| v.1)
                .collect::<Vec<_>>(),
            [Decimal::ZERO; 3]
        );
        assert_eq!(header_summary(tab, &data, "")[0].1, dec!(9999));
    }
}

#[test]
fn header_tones_mark_negative_profit_only() {
    let tones = |profit: Decimal, taxes: Decimal, after_tax: Decimal| {
        let data = ReceiptTabData {
            rows: vec![],
            total: None,
            summary: Some(ReceiptSummary::DomesticStock(DomesticStockSummary {
                total_realized_profit_and_loss: profit,
                total_taxes: taxes,
                total_realized_profit_and_loss_after_tax: after_tax,
            })),
            truncated: false,
        };
        header_summary(ReceiptsTab::DomesticStock, &data, "")
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
    let filtered = filter_receipts(ReceiptsTab::Dividend, &rows, "9910 2024");
    let groups = ReceiptsTab::Dividend.table_groups(&filtered, &rows, "9910 2024");
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].label, "TELE");
    assert_eq!(groups[0].summary, ["¥500", "¥100", "¥400"]);
    let filtered = filter_receipts(ReceiptsTab::Dividend, &rows, "9910");
    let groups = ReceiptsTab::Dividend.table_groups(&filtered, &rows, "9910");
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].rows.len(), 2);
    assert_eq!(groups[0].summary, ["¥1,000", "¥200", "¥800"]);
}

// 表示単位は合計欄(グループ小計)だけに使い、明細の行は1円単位のまま出す
#[test]
fn receipt_rows_stay_in_yen_while_group_summary_follows_the_unit() {
    use crate::settings::{AmountUnit, amount_unit_signal};
    use leptos::prelude::Set;
    let unit = amount_unit_signal();
    unit.set(AmountUnit::Man);
    let texts = |rows: &[ReceiptRow]| {
        rows[0]
            .cells()
            .into_iter()
            .filter_map(|cell| match cell {
                ReceiptCell::Text(text) => Some(text),
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    let dividend_texts = texts(&dividends());
    assert!(dividend_texts.iter().any(|text| text == "¥500"));
    assert!(dividend_texts.iter().any(|text| text == "¥400"));
    assert!(!dividend_texts.iter().any(|text| text.contains("万円")));
    let domestic_texts = texts(&domestic());
    assert!(domestic_texts.iter().any(|text| text == "¥200,000"));
    assert!(!domestic_texts.iter().any(|text| text.contains("万円")));
    let fund_texts = texts(&funds());
    assert!(!fund_texts.iter().any(|text| text.contains("万円")));
    // 合計欄は単位に従う
    let rows = dividends();
    let groups = ReceiptsTab::Dividend.table_groups(&rows, &rows, "");
    assert!(
        groups
            .iter()
            .any(|group| group.summary.iter().any(|text| text.ends_with("万円")))
    );
    unit.set(AmountUnit::Yen);
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
    let groups = ReceiptsTab::DomesticStock.table_groups(&rows, &rows, "9904");
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
    assert_eq!(
        group_label("架空インデックス-Slim"),
        "架空インデックス-Slim"
    );
    assert_eq!(group_label("Alpha-Fund-A"), "Alpha-Fund-A");
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
    assert!(!is_negative_text("ＴＥＬＥ"));
    assert!(!is_negative_text("-"));
    assert!(!is_negative_text("12."));
    assert!(!is_negative_text("1.2.3"));
    // f64 としてパースできても桁チェックを通らない表記は負数扱いしない
    assert!(!is_negative_text("-1e5"));
    assert!(!is_negative_text("-inf"));
}

#[test]
fn card_fields_point_at_expected_columns() {
    let cases = [
        (
            ReceiptsTab::Dividend,
            ("銘柄名", "入金日", "口座", Some("銘柄コード")),
        ),
        (
            ReceiptsTab::DomesticStock,
            ("銘柄名", "約定日", "口座", Some("銘柄コード")),
        ),
        (
            ReceiptsTab::MutualFund,
            ("ファンド名", "約定日", "口座", None),
        ),
    ];
    for (tab, expected) in cases {
        let fields = tab.card_fields();
        let columns = tab.columns();
        assert_eq!(
            (
                columns[fields.name].header,
                columns[fields.date].header,
                columns[fields.account].header,
                fields.code.map(|index| columns[index].header)
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
    assert!(
        !funds()[0]
            .cells()
            .iter()
            .any(|cell| matches!(cell, ReceiptCell::SecurityCode(_)))
    );
}

#[test]
fn table_column_widths_match_headers_and_follow_column_order() {
    let rows = domestic();
    let order = column_order(ReceiptsTab::DomesticStock, &rows, "特定");
    let columns = ReceiptsTab::DomesticStock.columns();
    let displayed: Vec<(&str, &str)> = order
        .iter()
        .map(|&i| (columns[i].header, columns[i].width))
        .collect();
    let fields = ReceiptsTab::DomesticStock.card_fields();
    assert_eq!(
        displayed[0],
        (columns[fields.date].header, columns[fields.date].width)
    );
    assert_eq!(
        displayed[2],
        (
            columns[fields.account].header,
            columns[fields.account].width
        )
    );
    assert_eq!(
        displayed[3],
        (columns[fields.name].header, columns[fields.name].width)
    );
}

// 「表示項目は極力削らない」方針: 数量はできるだけ早い帯から出す。
// 配当金は列が少なく md(768px)帯でも入るため Md。国内株式・投資信託は金額列が多く
// md 帯で数量まで出すと銘柄名が潰れるため Wide(レールを畳んだ lg・xl から)にとどめる
#[test]
fn quantity_column_tiers_match_width_budget() {
    let index_of = |tab: ReceiptsTab| {
        tab.columns()
            .iter()
            .position(|column| column.header == "数量")
            .expect("数量列がある")
    };
    assert_eq!(
        ReceiptsTab::Dividend.columns()[index_of(ReceiptsTab::Dividend)].tier,
        ColumnTier::Md
    );
    for tab in [ReceiptsTab::DomesticStock, ReceiptsTab::MutualFund] {
        assert_eq!(tab.columns()[index_of(tab)].tier, ColumnTier::Wide);
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

#[test]
fn card_row_data_maps_card_fields() {
    let rows = dividends();
    let cells = rows[0].cells();
    let order = column_order(ReceiptsTab::Dividend, &rows, "");
    let card = card_row_data(
        ReceiptsTab::Dividend,
        "test".to_string(),
        &cells,
        &order,
        ReceiptsTab::Dividend.card_fields(),
    );
    assert!(matches!(
        &card.name,
        CardDetailValue::CopyName { display, .. } if display == "架空電信電話"
    ));
    assert_eq!(card.code.as_deref(), Some("9910"));
    assert_eq!(card.date, "2024/06/21");
    assert_eq!(card.account, "特定");
    // 見出し(銘柄コード・銘柄名・入金日・口座)を除き、商品を含む残り6列を表の列順で入れる
    assert_eq!(card.details.len(), 6);
    assert_eq!(card.details[0].label, "商品");
    assert!(card.details.iter().all(|detail| {
        !["銘柄コード", "銘柄名", "入金日", "口座"].contains(&detail.label.as_str())
    }));
    assert!(card.details.iter().all(|detail| match &detail.value {
        CardDetailValue::Text { text, negative } => {
            *negative
                == (is_profit_label(ReceiptsTab::Dividend, &detail.label) && is_negative_text(text))
        }
        CardDetailValue::CopyName { .. } => true,
    }));
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
    card_row_data(tab, "test".to_string(), &cells, &order, tab.card_fields())
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
fn card_header_carries_code_badge_value_and_copy_name() {
    let rows = dividends();
    let cells = rows[0].cells();
    let order = column_order(ReceiptsTab::Dividend, &rows, "");
    let card = card_row_data(
        ReceiptsTab::Dividend,
        "test".to_string(),
        &cells,
        &order,
        ReceiptsTab::Dividend.card_fields(),
    );
    // 銘柄コードは見出しのバッジに出すため明細には入れない
    assert_eq!(card.code.as_deref(), Some("9910"));
    assert!(
        card.details
            .iter()
            .all(|detail| detail.label != "銘柄コード")
    );
    // 銘柄名は見出し側でコピーできるため明細には入れない
    assert!(matches!(
        &card.name,
        CardDetailValue::CopyName { display, copy }
            if display == "架空電信電話" && copy == "架空電信電話(9910)"
    ));
    assert!(
        !card
            .details
            .iter()
            .any(|detail| matches!(&detail.value, CardDetailValue::CopyName { .. }))
    );

    let rows = funds();
    let cells = rows[0].cells();
    let order = column_order(ReceiptsTab::MutualFund, &rows, "");
    let card = card_row_data(
        ReceiptsTab::MutualFund,
        "test".to_string(),
        &cells,
        &order,
        ReceiptsTab::MutualFund.card_fields(),
    );
    assert_eq!(card.code, None);
    assert!(matches!(
        &card.name,
        CardDetailValue::CopyName { display, copy } if display == copy
    ));

    // コード列を持つタブで空コードは欠損値としてバッジに残す
    let row: crate::api::dto::Dividend = serde_json::from_value(serde_json::json!({
        "id": "no-code", "settlement_date": "2024-06-21", "product": "投資信託",
        "account": "特定", "security_code": "", "security_name": "コードなし銘柄",
        "unit_price": 5, "shares": 100, "dividends_before_tax": 500, "taxes": 100,
        "net_amount_received": 400, "created_at": "", "updated_at": ""
    }))
    .unwrap();
    let rows = vec![Saved(ReceiptItem::Dividend(row))];
    let cells = rows[0].cells();
    let order = column_order(ReceiptsTab::Dividend, &rows, "");
    let card = card_row_data(
        ReceiptsTab::Dividend,
        "test".to_string(),
        &cells,
        &order,
        ReceiptsTab::Dividend.card_fields(),
    );
    assert_eq!(card.code.as_deref(), Some(""));
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
fn group_label_formats_iso_keys_only() {
    assert!(is_date_group_key("2026-06"));
    assert!(is_date_group_key("2024-03-05"));
    assert!(!is_date_group_key("架空電信電話"));
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
    let groups = ReceiptsTab::Dividend.table_groups(&rows, &rows, "9910");
    assert_eq!(groups[0].label, "TELE");
}

#[test]
fn cells_normalize_full_width_security_names() {
    let rows = dividends();
    assert!(rows[1].cells().iter().any(|cell| matches!(
        cell,
        ReceiptCell::InstrumentName { name, .. } if name == "TELE"
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
        "test".to_string(),
        &cells,
        &order,
        ReceiptsTab::Dividend.card_fields(),
    );
    // 口座は前に出ても見出し側の項目なので明細には入らない
    assert_eq!(card.details[0].label, "商品");
    assert!(card.details.iter().all(|detail| detail.label != "口座"));
    assert!(matches!(
        &card.name,
        CardDetailValue::CopyName { display, .. } if display == "架空電信電話"
    ));
    assert_eq!(card.account, "特定");
}

fn csv_source_user(id: &str) -> crate::api::dto::SessionUser {
    crate::api::dto::SessionUser {
        id: id.to_string(),
        email: format!("{id}@example.com"),
        name: None,
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
        tabs: RwSignal::new(HashMap::new()),
        fetch,
    }
}

#[test]
fn receipt_csv_source_constants_match_expected_values() {
    use super::csv_page::ReceiptCsvSource;
    use crate::ui::csv_page::CsvSource;
    use leptos::prelude::*;

    let owner = Owner::new();
    owner.with(|| {
        let store = csv_source_store();
        for tab in ReceiptsTab::ALL {
            let source = ReceiptCsvSource { store, tab };
            assert_eq!(source.input_id(), tab.csv_input_id());
            assert_eq!(source.save_action(), "追加で保存");
            assert_eq!(source.save_mode_label(), "追加");
            assert_eq!(source.save_button_label(Some(3)), "3件を保存する");
            assert_eq!(source.save_button_label(None), "保存する");
        }
    });
}

#[test]
fn receipt_csv_source_input_disabled_reflects_store_state() {
    use super::csv_page::ReceiptCsvSource;
    use crate::ui::csv_page::CsvSource;
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
    use super::csv_page::ReceiptCsvSource;
    use crate::ui::csv_page::CsvSource;
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
    use super::csv_page::ReceiptCsvSource;
    use crate::support::csv_flow::CsvTabMeta;
    use crate::ui::csv_page::CsvSource;
    use leptos::prelude::*;

    let owner = Owner::new();
    owner.with(|| {
        let source = ReceiptCsvSource {
            store: csv_source_store(),
            tab: ReceiptsTab::Dividend,
        };

        let idle_state = CsvTabMeta::default();
        assert!(!source.delete_disabled(&idle_state), "idle -> not disabled");

        let saving_state = CsvTabMeta {
            saving: true,
            ..Default::default()
        };
        assert!(source.delete_disabled(&saving_state), "saving -> disabled");

        let deleting_state = CsvTabMeta {
            deleting: true,
            ..Default::default()
        };
        assert!(
            source.delete_disabled(&deleting_state),
            "deleting -> disabled"
        );
    });
}

#[test]
fn receipt_csv_source_delete_count_error_masks_stale_error_while_loading() {
    use super::csv_page::ReceiptCsvSource;
    use crate::features::receipts::{ReceiptTabData, TabState};
    use crate::ui::csv_page::CsvSource;
    use leptos::prelude::*;

    let owner = Owner::new();
    owner.with(|| {
        let store = csv_source_store();
        let tab = ReceiptsTab::Dividend;
        let source = ReceiptCsvSource { store, tab };
        let generation = store.session.generation.get_untracked();

        // 前回の裏再取得が失敗し、削除ページで始めた取り直しが進行中
        store.tabs.update(|map| {
            let entry = map.entry((generation, tab)).or_default();
            entry.list = Some(TabState::Loading);
            entry.refresh_error = Some("前回の失敗".to_string());
        });
        assert_eq!(
            source.delete_count_error(),
            None,
            "取り直し中は古い失敗を出さず取得中として扱う"
        );
        assert!(!source.delete_count_ready());

        // 取り直しが完了しても失敗が残るなら未確定としてエラーを出す
        store.tabs.update(|map| {
            let entry = map.entry((generation, tab)).or_default();
            entry.list = Some(TabState::Ready(ReceiptTabData {
                rows: vec![],
                total: Some(1),
                summary: None,
                truncated: false,
            }));
        });
        assert_eq!(source.delete_count_error().as_deref(), Some("前回の失敗"));
        assert!(!source.delete_count_ready());

        // 初回取得の失敗は Failed としてそのまま出す
        store.tabs.update(|map| {
            let entry = map.entry((generation, tab)).or_default();
            entry.list = Some(TabState::Failed("取得失敗".to_string()));
            entry.refresh_error = None;
        });
        assert_eq!(source.delete_count_error().as_deref(), Some("取得失敗"));
    });
}

#[test]
fn receipt_csv_source_save_disabled_requires_preview_rows() {
    use super::csv_page::ReceiptCsvSource;
    use crate::api::dto::CsvRowError;
    use crate::support::csv_flow::CsvTabMeta;
    use crate::ui::csv_page::CsvSource;
    use leptos::prelude::*;

    let owner = Owner::new();
    owner.with(|| {
        let source = ReceiptCsvSource {
            store: csv_source_store(),
            tab: ReceiptsTab::Dividend,
        };

        // ファイルを選んでも検証前・検証失敗(プレビュー無し)では保存できない
        let selected = CsvTabMeta {
            file_name: Some("a.csv".to_string()),
            ..Default::default()
        };
        assert!(
            source.save_disabled(&selected),
            "プレビュー無し -> 押せない"
        );

        // 有効行が0件(全行エラー)のプレビューでも押せない
        let all_error = CsvTabMeta {
            file_name: Some("a.csv".to_string()),
            preview: Some(crate::support::csv_flow::CsvPreviewMeta {
                valid_rows: 0,
                errors: vec![CsvRowError {
                    row: 1,
                    message: "x".to_string(),
                }],
                has_rows: false,
            }),
            ..Default::default()
        };
        assert!(source.save_disabled(&all_error), "有効行0件 -> 押せない");

        // 有効行があるプレビューでは押せる。処理中は押せない
        let ready = CsvTabMeta {
            file_name: Some("a.csv".to_string()),
            preview: Some(crate::support::csv_flow::CsvPreviewMeta {
                valid_rows: 2,
                errors: vec![],
                has_rows: true,
            }),
            ..Default::default()
        };
        assert!(!source.save_disabled(&ready), "有効行あり -> 押せる");
        let busy = CsvTabMeta {
            saving: true,
            ..ready
        };
        assert!(source.save_disabled(&busy), "保存中 -> 押せない");
    });
}

#[test]
fn receipt_csv_source_format_hint_names_expected_header() {
    use super::csv_page::ReceiptCsvSource;
    use crate::ui::csv_page::CsvSource;
    use leptos::prelude::*;

    let owner = Owner::new();
    owner.with(|| {
        let store = csv_source_store();
        let hints: Vec<&str> = ReceiptsTab::ALL
            .iter()
            .map(|tab| ReceiptCsvSource { store, tab: *tab }.csv_format_hint())
            .collect();
        assert!(
            hints.iter().all(|hint| hint.contains("見出し")),
            "各タブが想定する列を案内する"
        );
        assert!(
            hints.iter().all(|hint| hint.contains("CSV")),
            "どのファイルか分かる名前を含む"
        );
    });
}

#[test]
fn ensure_kind_resets_search_state_like_tab_switch() {
    use super::csv_page::ensure_kind;
    use crate::features::receipts::TabState;
    use crate::features::receipts::filter::ReceiptSearch;
    use leptos::prelude::*;

    let owner = Owner::new();
    owner.with(|| {
        let store = csv_source_store();
        // 世代リセットの検索クリアを通さず、タブ切替のリセットだけを見る
        let generation = store.session.generation.get_untracked();
        store.expanded_epoch.set(Some(generation));
        // 対象タブを取得済みにして ensure の fetch dispatch を起こさない
        store.tabs.update(|map| {
            map.entry((generation, ReceiptsTab::DomesticStock))
                .or_default()
                .list = Some(TabState::Ready(ReceiptTabData {
                rows: vec![],
                total: Some(0),
                summary: None,
                truncated: false,
            }));
        });
        store.search.set(ReceiptSearch {
            query: "9904".to_string(),
            ..Default::default()
        });
        store.expanded.update(|set| {
            set.insert("g0".to_string());
        });

        ensure_kind(store, ReceiptsTab::DomesticStock);

        assert_eq!(store.active_tab.get_untracked(), ReceiptsTab::DomesticStock);
        assert_eq!(store.search.get_untracked().query, "");
        assert!(store.expanded.with_untracked(|set| set.is_empty()));
    });
}

#[test]
fn condition_count_covers_structured_filters_only() {
    use super::action_bar::condition_count;
    use crate::features::receipts::filter::{ReceiptSearch, SearchKey};

    let mut search = ReceiptSearch::new(false);
    assert_eq!(condition_count(&search), 0);
    // 銘柄名・コードの検索は一覧側の入力欄で扱うため、帯の件数には入れない
    search.set_securities_query("TELE".to_string());
    assert_eq!(condition_count(&search), 0);
    search.select_quick(SearchKey::Products, "国内株式".to_string());
    search.select_quick(SearchKey::Accounts, "特定".to_string());
    assert_eq!(condition_count(&search), 2);
    search.set_date_query("2024".to_string());
    assert_eq!(condition_count(&search), 3);
    search.clear_key(SearchKey::Accounts, false);
    assert_eq!(condition_count(&search), 2);
}
