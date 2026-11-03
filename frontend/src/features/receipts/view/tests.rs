use super::cards::*;
use super::main_content::{derive_filtered_count, show_dividend_section};
use super::summary::*;
use super::tabs::*;
use crate::api::dto::{DividendSummary, DomesticStockSummary, MutualfundSummary};
use crate::features::receipts::filter::ReceiptSearch;
use crate::features::receipts::filter::{column_order, filter_receipts};
use crate::features::receipts::kind::{
    ColumnTier, dividend_totals, group_label, is_date_group_key,
};
use crate::features::receipts::{
    ReceiptCell, ReceiptItem, ReceiptRow, ReceiptSummary, ReceiptTabData, ReceiptsTab,
};
use crate::testing::receipts::{dividends, domestic, funds};
use crate::ui::amount::AmountKind;
use crate::ui::card::StatTone;
use crate::ui::labels::{
    AFTER_TAX, BEFORE_TAX, DIVIDEND_AMOUNT, DIVIDEND_BEFORE_TAX, PAYMENT_DATE, REALIZED_BEFORE_TAX,
    REALIZED_PROFIT_LOSS, REDEMPTION_AMOUNT, SELLING_AMOUNT, TAX, TRADE_DATE,
};
use crate::ui::page_skeleton::FilteredCount;
use leptos::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

// 絞り込み結果の共有先(配当情報の切り替え・集計・期間・件数・表)は同じ入力から同じ値を出す
#[test]
fn dividend_section_switch_needs_dividend_tab_and_exact_code() {
    let rows = dividends();
    let filtered = filter_receipts(ReceiptsTab::Dividend, &rows, "9904");
    assert!(show_dividend_section(
        ReceiptsTab::Dividend,
        &filtered,
        "9904"
    ));

    assert!(!show_dividend_section(ReceiptsTab::Dividend, &rows, ""));
    assert!(!show_dividend_section(ReceiptsTab::Dividend, &rows, "架空"));

    let domestic_rows = filter_receipts(ReceiptsTab::DomesticStock, &domestic(), "9904");
    assert!(!domestic_rows.is_empty());
    assert!(!show_dividend_section(
        ReceiptsTab::DomesticStock,
        &domestic_rows,
        "9904"
    ));

    let fund_rows = filter_receipts(ReceiptsTab::MutualFund, &funds(), "Alpha");
    assert!(!fund_rows.is_empty());
    assert!(!show_dividend_section(
        ReceiptsTab::MutualFund,
        &fund_rows,
        "Alpha"
    ));
}

#[test]
fn dividend_totals_match_filtered_rows_when_section_shows() {
    let filtered = filter_receipts(ReceiptsTab::Dividend, &dividends(), "9904");
    assert!(show_dividend_section(
        ReceiptsTab::Dividend,
        &filtered,
        "9904"
    ));
    let totals = dividend_totals(&filtered);
    assert_eq!(
        (
            totals.total_dividends_before_tax,
            totals.total_taxes,
            totals.total_net_amount_received
        ),
        (dec!(500), dec!(100), dec!(400))
    );
}

#[test]
fn filtered_count_keeps_api_total_and_filtering_flag() {
    let rows = dividends();
    let with_total = ReceiptTabData {
        rows: rows.clone(),
        total: Some(100),
        summary: None,
        truncated: false,
    };
    assert_eq!(
        derive_filtered_count(&with_total, 1, true),
        FilteredCount {
            filtered: 1,
            total: 100,
            filtering: true,
        }
    );
    let without_total = ReceiptTabData {
        rows: rows.clone(),
        total: None,
        summary: None,
        truncated: false,
    };
    assert_eq!(
        derive_filtered_count(&without_total, rows.len(), false),
        FilteredCount {
            filtered: rows.len(),
            total: rows.len(),
            filtering: false,
        }
    );
}

#[test]
fn receipt_sheet_draft_apply_matches_direct_filter() {
    let owner = Owner::new();
    owner.with(|| {
        let search = RwSignal::new(ReceiptSearch::new(true));
        let mut draft = ReceiptSearch::new(true);
        draft.set_securities_query("9904".to_string());
        super::action_bar::apply_receipt_sheet_draft(search, &draft);
        let rows = dividends();
        let query = search.with(|s| s.query.clone());
        let applied = filter_receipts(ReceiptsTab::Dividend, &rows, &query);
        let direct = filter_receipts(ReceiptsTab::Dividend, &rows, &draft.query);
        assert_eq!(applied.len(), direct.len());
        assert!(!applied.is_empty());
    });
}

#[test]
fn receipt_dialog_clear_resets_search() {
    let owner = Owner::new();
    owner.with(|| {
        let draft = RwSignal::new({
            let mut search = ReceiptSearch::new(true);
            search.set_securities_query("9904".to_string());
            search.select_quick(
                crate::features::receipts::filter::SearchKey::Accounts,
                "SBI証券".to_string(),
            );
            search
        });
        super::action_bar::clear_receipt_dialog_draft(draft, true);
        assert!(draft.with(|s| s.query.is_empty()));
    });
}

#[test]
fn receipt_sort_order_round_trips() {
    use crate::features::receipts::filter::ReceiptSortOrder;
    assert_eq!(
        ReceiptSortOrder::from_value(ReceiptSortOrder::Newest.value()),
        Some(ReceiptSortOrder::Newest)
    );
    assert_eq!(
        ReceiptSortOrder::from_value(ReceiptSortOrder::Oldest.value()),
        Some(ReceiptSortOrder::Oldest)
    );
    assert_eq!(ReceiptSortOrder::from_value("other"), None);
}

#[test]
fn receipt_sheet_date_label_matches_tab() {
    assert_eq!(
        super::action_bar::receipt_date_label(ReceiptsTab::Dividend),
        PAYMENT_DATE
    );
    assert_eq!(
        super::action_bar::receipt_date_label(ReceiptsTab::DomesticStock),
        TRADE_DATE
    );
    assert_eq!(
        super::action_bar::receipt_date_label(ReceiptsTab::MutualFund),
        TRADE_DATE
    );
}

#[test]
fn period_note_short_spans_filtered_rows() {
    let filtered = filter_receipts(ReceiptsTab::Dividend, &dividends(), "9910");
    assert_eq!(filtered.len(), 2);
    assert_eq!(
        period_note_short(&filtered),
        Some("2024/06 〜 2026/06".to_string())
    );
    let single = filter_receipts(ReceiptsTab::Dividend, &dividends(), "9904 2026");
    assert_eq!(single.len(), 1);
    assert_eq!(
        period_note_short(&single),
        Some("2026/06 〜 2026/06".to_string())
    );
}

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

// 帯に出す小さな期間は絞り込み後の明細の最初と最後の年月を出す。行が無いときは出さない
#[test]
fn period_note_short_spans_first_to_last_year_month() {
    assert_eq!(period_note_short(&[]), None);
    assert_eq!(
        period_note_short(&dividends()),
        Some("2024/06 〜 2026/06".to_string())
    );
    assert_eq!(
        period_note_short(&domestic()),
        Some("2024/03 〜 2024/03".to_string())
    );
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
    let groups = ReceiptsTab::Dividend.table_groups(
        &filtered,
        &rows,
        "9910 2024",
        crate::features::receipts::filter::ReceiptSortOrder::Newest,
    );
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].label, "TELE");
    assert_eq!(groups[0].summary, ["¥500", "¥100", "¥400"]);
    let filtered = filter_receipts(ReceiptsTab::Dividend, &rows, "9910");
    let groups = ReceiptsTab::Dividend.table_groups(
        &filtered,
        &rows,
        "9910",
        crate::features::receipts::filter::ReceiptSortOrder::Newest,
    );
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
    let groups = ReceiptsTab::Dividend.table_groups(
        &rows,
        &rows,
        "",
        crate::features::receipts::filter::ReceiptSortOrder::Newest,
    );
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
        let groups = ReceiptsTab::Dividend.table_groups(
            &filtered,
            &rows,
            query,
            crate::features::receipts::filter::ReceiptSortOrder::Newest,
        );
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].label, expected);
    }
}
#[test]
fn mutual_fund_name_groups_and_domestic_monthly_groups() {
    let rows = funds();
    assert_eq!(
        ReceiptsTab::MutualFund.table_groups(
            &rows,
            &rows,
            "\"Alpha Fund A\"",
            crate::features::receipts::filter::ReceiptSortOrder::Newest,
        )[0]
        .label,
        "Alpha Fund A"
    );
    let rows = domestic();
    let groups = ReceiptsTab::DomesticStock.table_groups(
        &rows,
        &rows,
        "9904",
        crate::features::receipts::filter::ReceiptSortOrder::Newest,
    );
    assert_eq!(groups[0].label, "2024年3月");
    assert_eq!(groups[0].summary, ["¥1,000", "¥888", "¥112"]);
}

#[test]
fn domestic_monthly_summary_groups_by_month() {
    let mut rows = domestic();
    let mut second = match &rows[0] {
        ReceiptItem::DomesticStock(row) => row.clone(),
        _ => unreachable!(),
    };
    second.id = "second".to_string().into();
    rows.push(ReceiptItem::DomesticStock(second.clone()));
    second.id = "next-day".to_string().into();
    second.trade_date = "2024-03-02".into();
    rows.push(ReceiptItem::DomesticStock(second));
    let groups = ReceiptsTab::DomesticStock.table_groups(
        &rows,
        &rows,
        "",
        crate::features::receipts::filter::ReceiptSortOrder::Newest,
    );
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].label, "2024年3月");
    assert_eq!(groups[0].rows.len(), 3);
    assert_eq!(groups[0].summary, ["¥3,000", "¥2,664", "¥336"]);
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
    for (cells, index) in [(dividends()[0].cells(), 1), (domestic()[0].cells(), 1)] {
        assert!(matches!(cells[index], ReceiptCell::SecurityCode(_)));
        // 銘柄コードのすぐ右が銘柄名
        assert!(matches!(
            cells[index + 1],
            ReceiptCell::InstrumentName { .. }
        ));
    }
    assert!(
        !funds()[0]
            .cells()
            .iter()
            .any(|cell| matches!(cell, ReceiptCell::SecurityCode(_)))
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
    // 税額は符号を反転するため、保存済みの符号付き表示からは再計算しない。既知の値で確かめる
    let (gross_text, gross_negative, gross_positive, gross_profit) = detail_value(&card, "配当金");
    assert_eq!(gross_text, "+¥500");
    assert!(!gross_negative);
    assert!(gross_positive);
    assert!(gross_profit);
    let (tax_text, tax_negative, tax_positive, _) = detail_value(&card, "税額");
    assert_eq!(tax_text, "-¥100");
    assert!(tax_negative);
    assert!(!tax_positive);
}

fn detail_negative(card: &CardRowData, label: &str) -> bool {
    card.details
        .iter()
        .find(|detail| detail.label == label)
        .is_some_and(|detail| matches!(detail.value, CardDetailValue::Text { negative: true, .. }))
}

fn detail_value(card: &CardRowData, label: &str) -> (String, bool, bool, bool) {
    card.details
        .iter()
        .find(|detail| detail.label == label)
        .map(|detail| match &detail.value {
            CardDetailValue::Text {
                text,
                negative,
                positive,
                is_profit,
            } => (text.clone(), *negative, *positive, *is_profit),
            CardDetailValue::CopyName { display, .. } => (display.clone(), false, false, false),
        })
        .unwrap_or_default()
}

fn card_for(tab: ReceiptsTab, row: ReceiptRow) -> CardRowData {
    let rows = vec![row];
    let cells = rows[0].cells();
    let order = column_order(tab, &rows, "");
    card_row_data(tab, &cells, &order, tab.card_fields())
}

#[test]
fn tax_outflow_uses_inverted_sign() {
    let mut stock = match domestic().remove(0) {
        ReceiptItem::DomesticStock(row) => row,
        _ => unreachable!(),
    };
    stock.realized_profit_and_loss = rust_decimal::Decimal::from(-1000);
    stock.taxes = rust_decimal::Decimal::from(-203);
    stock.realized_profit_and_loss_after_tax = rust_decimal::Decimal::from(-797);
    let card = card_for(
        ReceiptsTab::DomesticStock,
        ReceiptItem::DomesticStock(stock.clone()),
    );
    assert!(detail_negative(&card, "実現損益"));
    assert!(!detail_negative(&card, "税額"));
    let (refund_text, _, refund_positive, _) = detail_value(&card, "税額");
    assert_eq!(refund_text, "+¥203");
    assert!(refund_positive);

    stock.taxes = rust_decimal::Decimal::from(203);
    let card = card_for(
        ReceiptsTab::DomesticStock,
        ReceiptItem::DomesticStock(stock),
    );
    assert!(detail_negative(&card, "税額"));
    assert_eq!(detail_value(&card, "税額").0, "-¥203");

    let mut dividend = match dividends().remove(0) {
        ReceiptItem::Dividend(row) => row,
        _ => unreachable!(),
    };
    dividend.taxes = rust_decimal::Decimal::from(-100);
    dividend.net_amount_received = rust_decimal::Decimal::from(-50);
    let card = card_for(ReceiptsTab::Dividend, ReceiptItem::Dividend(dividend));
    assert!(!detail_negative(&card, "税額"));
    assert!(detail_negative(&card, "税引後"));
}

#[test]
fn dividend_receipt_counts_as_inflow() {
    assert!(is_profit_label(ReceiptsTab::Dividend, "税引後"));
    assert!(is_profit_label(ReceiptsTab::Dividend, "配当金"));
    assert!(is_profit_label(ReceiptsTab::Dividend, "税引前"));
    assert!(!is_profit_label(ReceiptsTab::Dividend, "税額"));
    assert!(is_profit_label(ReceiptsTab::DomesticStock, "税引後"));
    assert!(is_profit_label(ReceiptsTab::DomesticStock, "税引前"));
    assert!(is_profit_label(ReceiptsTab::MutualFund, "税引後"));
    assert!(!is_profit_label(ReceiptsTab::DomesticStock, "税額"));
    assert_eq!(
        amount_kind_for_label(ReceiptsTab::Dividend, "税額", "¥100"),
        AmountKind::Delta
    );
    assert_eq!(
        amount_kind_for_label(ReceiptsTab::Dividend, "税額", "¥0"),
        AmountKind::Plain
    );
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
    assert!(is_negative_labeled_value(
        ReceiptsTab::DomesticStock,
        "税額",
        "¥1,234"
    ));
    assert!(!is_negative_labeled_value(
        ReceiptsTab::DomesticStock,
        "税額",
        "¥0"
    ));
    assert!(is_negative_labeled_value(
        ReceiptsTab::Dividend,
        "税引後",
        "-¥1,234"
    ));
}

#[test]
fn tax_signed_text_inverts_outflow() {
    assert_eq!(tax_signed_text("¥1,234"), "-¥1,234");
    assert_eq!(tax_signed_text("-¥1,234"), "+¥1,234");
    assert_eq!(tax_signed_text("¥0"), "¥0");
    assert_eq!(tax_signed_text("—"), "—");
    assert_eq!(
        signed_text_for_profit(ReceiptsTab::Dividend, "税額", "¥100"),
        "-¥100"
    );
    assert_eq!(
        signed_text_for_profit(ReceiptsTab::Dividend, "配当金", "¥500"),
        "+¥500"
    );
    assert_eq!(
        signed_text_for_profit(ReceiptsTab::Dividend, "税引前", "¥500"),
        "+¥500"
    );
}

#[test]
fn card_header_carries_code_badge_value_and_copy_name() {
    let rows = dividends();
    let cells = rows[0].cells();
    let order = column_order(ReceiptsTab::Dividend, &rows, "");
    let card = card_row_data(
        ReceiptsTab::Dividend,
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
    let rows = vec![ReceiptItem::Dividend(row)];
    let cells = rows[0].cells();
    let order = column_order(ReceiptsTab::Dividend, &rows, "");
    let card = card_row_data(
        ReceiptsTab::Dividend,
        &cells,
        &order,
        ReceiptsTab::Dividend.card_fields(),
    );
    assert_eq!(card.code.as_deref(), Some(""));
}

#[test]
fn table_groups_carry_group_key_and_row_ids() {
    let rows = dividends();
    let groups = ReceiptsTab::Dividend.table_groups(
        &rows,
        &rows,
        "",
        crate::features::receipts::filter::ReceiptSortOrder::Newest,
    );
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
        ReceiptItem::Dividend(row) => row.clone(),
        _ => panic!("dividend row"),
    };
    same_date.security_name = "別名".into();
    rows.push(ReceiptItem::Dividend(same_date));
    let groups = ReceiptsTab::Dividend.table_groups(
        &rows,
        &rows,
        "9910",
        crate::features::receipts::filter::ReceiptSortOrder::Newest,
    );
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
    let groups = ReceiptsTab::DomesticStock.table_groups(
        &domestic_rows,
        &domestic_rows,
        "",
        crate::features::receipts::filter::ReceiptSortOrder::Newest,
    );
    assert_eq!(groups[0].rows.len(), 1);
    let fund_rows = funds();
    let groups = ReceiptsTab::MutualFund.table_groups(
        &fund_rows,
        &fund_rows,
        "",
        crate::features::receipts::filter::ReceiptSortOrder::Newest,
    );
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
    assert_eq!(order[1], 3);
    let cells = rows[0].cells();
    let card = card_row_data(
        ReceiptsTab::Dividend,
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

#[test]
fn receipt_dialog_follows_specified_order_and_uses_row_values() {
    for tab in ReceiptsTab::ALL {
        let rows = match tab {
            ReceiptsTab::Dividend => dividends(),
            ReceiptsTab::DomesticStock => domestic(),
            ReceiptsTab::MutualFund => funds(),
        };
        let cells = rows[0].cells();
        let order = column_order(tab, &rows, "");
        let card = card_row_data(tab, &cells, &order, tab.card_fields());
        let (left, right) = receipt_detail_cells(tab, &card);
        let left_labels: Vec<&str> = left.iter().map(|cell| cell.label.as_str()).collect();
        let right_labels: Vec<&str> = right.iter().map(|cell| cell.label.as_str()).collect();
        match tab {
            ReceiptsTab::Dividend => {
                assert_eq!(left_labels, ["入金日", "数量", "1株配当"]);
                assert_eq!(right_labels, ["税引前", "税額", "税引後"]);
            }
            ReceiptsTab::DomesticStock => {
                assert_eq!(left_labels, ["約定日", "数量", "売却額", "取得価額"]);
                assert_eq!(right_labels, ["税引前", "税額", "税引後"]);
            }
            ReceiptsTab::MutualFund => {
                assert_eq!(left_labels, ["約定日", "数量", "解約額", "取得価額"]);
                assert_eq!(right_labels, ["税引前", "税額", "税引後"]);
            }
        }
        assert!(right[2].bold, "{tab:?}: 税引後は太字");
        assert!(right[2].delta, "{tab:?}: 税引後は損益として符号と色");
        assert!(right[0].delta, "{tab:?}: 税引前は損益として符号と色");
        assert!(!right[0].bold, "{tab:?}: 税引前は太字にしない");
        let (hero, _, _, _) = detail_value(&card, "税引後");
        assert!(!hero.is_empty(), "{tab:?}: 行の主役が無い");
        assert_eq!(right[2].value, hero, "{tab:?}: ダイアログは行と同じ基準");
    }
}

// 1件だけの特定口座の行では帯・月見出し・行の税引後が一致する
#[test]
fn single_specific_row_keeps_band_group_and_row_after_tax_equal() {
    use crate::api::dto::DomesticStock;
    let row: DomesticStock = serde_json::from_value(serde_json::json!({"id":"single", "trade_date":"2024-03-18", "settlement_date":"2024-03-18", "security_code":"9905", "security_name":"架空ゲーム", "account":"特定口座", "shares":10, "asked_price":5500, "proceeds":55000, "purchase_price":5000, "realized_profit_and_loss":5000, "taxes":1015, "realized_profit_and_loss_after_tax":3985, "created_at":"", "updated_at":""})).unwrap();
    let rows = vec![ReceiptItem::DomesticStock(row)];
    let query = "9905";
    let filtered = filter_receipts(ReceiptsTab::DomesticStock, &rows, query);
    assert_eq!(filtered.len(), 1);
    let data = ReceiptTabData {
        rows: filtered.clone(),
        total: None,
        summary: None,
        truncated: false,
    };
    assert_eq!(
        header_summary(ReceiptsTab::DomesticStock, &data, query)
            .iter()
            .map(|v| v.1)
            .collect::<Vec<_>>(),
        [dec!(5000), dec!(1015), dec!(3985)]
    );
    let groups = ReceiptsTab::DomesticStock.table_groups(
        &filtered,
        &rows,
        query,
        crate::features::receipts::filter::ReceiptSortOrder::Newest,
    );
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].summary, ["¥5,000", "¥1,015", "¥3,985"]);
    let card = card_for(ReceiptsTab::DomesticStock, filtered[0].clone());
    assert_eq!(
        single_row_headline(std::slice::from_ref(&card)),
        vec![(AFTER_TAX, "+¥3,985".to_string())]
    );
    assert_eq!(detail_value(&card, "税引後").0, "+¥3,985");
    let (left, right) = receipt_detail_cells(ReceiptsTab::DomesticStock, &card);
    assert_eq!(left[0].label, "約定日");
    assert_eq!(left[2].label, "売却額");
    assert_eq!(right[0].label, "税引前");
    assert_eq!(right[0].value, "+¥5,000");
    assert_eq!(right[2].label, "税引後");
    assert!(right[2].bold);
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

#[test]
fn profit_sign_uses_plus_minus_and_zero_has_no_sign() {
    assert!(is_positive_text("¥1,234"));
    assert!(is_positive_text("+¥1,234"));
    assert!(!is_positive_text("-¥1,234"));
    assert!(!is_positive_text("¥0"));
    assert!(!is_positive_text("—"));
    assert!(is_negative_text("-¥1,234"));
    assert!(!is_negative_text("¥1,234"));
    assert!(!is_negative_text("-¥0"));
}

#[test]
fn unit_suffixed_amounts_keep_sign_detection() {
    assert!(is_positive_text("10.0千円"));
    assert!(is_negative_text("-10.0千円"));
    assert!(!is_positive_text("0.0千円"));
    assert!(!is_negative_text("0.0千円"));
    assert!(is_positive_text("1.0万円"));
    assert!(is_negative_text("-1.0万円"));
    assert!(is_negative_text("-0.0千円"));
    assert!(!is_positive_text("-0.0千円"));
    assert!(!is_positive_text("—"));
    assert!(!is_negative_text("—"));
}

#[test]
fn profit_zero_keeps_delta_without_sign() {
    let mut stock = match domestic().remove(0) {
        ReceiptItem::DomesticStock(row) => row,
        _ => unreachable!(),
    };
    stock.realized_profit_and_loss = dec!(0);
    stock.taxes = dec!(0);
    stock.realized_profit_and_loss_after_tax = dec!(0);
    let card = card_for(
        ReceiptsTab::DomesticStock,
        ReceiptItem::DomesticStock(stock),
    );
    let (text, negative, positive, is_profit) = detail_value(&card, "実現損益");
    assert_eq!(text, "¥0");
    assert!(!negative);
    assert!(!positive);
    assert!(is_profit);
}

#[test]
fn profit_positive_gets_plus_and_green_only_for_profit_labels() {
    let mut stock = match domestic().remove(0) {
        ReceiptItem::DomesticStock(row) => row,
        _ => unreachable!(),
    };
    stock.realized_profit_and_loss = dec!(1000);
    stock.taxes = dec!(203);
    stock.realized_profit_and_loss_after_tax = dec!(797);
    let card = card_for(
        ReceiptsTab::DomesticStock,
        ReceiptItem::DomesticStock(stock),
    );
    let (text, negative, positive, is_profit) = detail_value(&card, "実現損益");
    assert_eq!(text, "+¥1,000");
    assert!(!negative);
    assert!(positive);
    assert!(is_profit);
    let (after_text, after_negative, after_positive, after_profit) = detail_value(&card, "税引後");
    assert_eq!(after_text, "+¥797");
    assert!(!after_negative);
    assert!(after_positive);
    assert!(after_profit);
    let (tax_text, tax_negative, tax_positive, tax_profit) = detail_value(&card, "税額");
    assert_eq!(tax_text, "-¥203");
    assert!(tax_negative);
    assert!(!tax_positive);
    assert!(tax_profit);
}

#[test]
fn tax_zero_uses_plain_and_refund_uses_plus() {
    let mut stock = match domestic().remove(0) {
        ReceiptItem::DomesticStock(row) => row,
        _ => unreachable!(),
    };
    stock.realized_profit_and_loss = dec!(1000);
    stock.taxes = dec!(0);
    stock.realized_profit_and_loss_after_tax = dec!(1000);
    let card = card_for(
        ReceiptsTab::DomesticStock,
        ReceiptItem::DomesticStock(stock.clone()),
    );
    let (zero_text, zero_negative, zero_positive, zero_profit) = detail_value(&card, "税額");
    assert_eq!(zero_text, "¥0");
    assert!(!zero_negative);
    assert!(!zero_positive);
    assert!(!zero_profit);
    assert_eq!(
        amount_kind_for_label(ReceiptsTab::DomesticStock, "税額", "¥0"),
        AmountKind::Plain
    );

    stock.taxes = dec!(-100);
    stock.realized_profit_and_loss_after_tax = dec!(1100);
    let card = card_for(
        ReceiptsTab::DomesticStock,
        ReceiptItem::DomesticStock(stock),
    );
    let (refund_text, refund_negative, refund_positive, refund_profit) =
        detail_value(&card, "税額");
    assert_eq!(refund_text, "+¥100");
    assert!(!refund_negative);
    assert!(refund_positive);
    assert!(refund_profit);
}

#[test]
fn before_tax_loss_uses_minus_and_delta() {
    let mut stock = match domestic().remove(0) {
        ReceiptItem::DomesticStock(row) => row,
        _ => unreachable!(),
    };
    stock.realized_profit_and_loss = dec!(-1000);
    stock.taxes = dec!(0);
    stock.realized_profit_and_loss_after_tax = dec!(-1000);
    let card = card_for(
        ReceiptsTab::DomesticStock,
        ReceiptItem::DomesticStock(stock),
    );
    let (text, negative, positive, is_profit) = detail_value(&card, "実現損益");
    assert_eq!(text, "-¥1,000");
    assert!(negative);
    assert!(!positive);
    assert!(is_profit);
    let (left, right) = receipt_detail_cells(ReceiptsTab::DomesticStock, &card);
    let _ = left;
    let before = right
        .iter()
        .find(|cell| cell.label == "税引前")
        .expect("税引前がある");
    assert_eq!(before.value, "-¥1,000");
    assert!(before.delta);
    assert!(before.negative);
}

#[test]
fn dividend_receipt_gets_plus_as_inflow() {
    let mut dividend = match dividends().remove(0) {
        ReceiptItem::Dividend(row) => row,
        _ => unreachable!(),
    };
    dividend.dividends_before_tax = dec!(500);
    dividend.net_amount_received = dec!(400);
    dividend.taxes = dec!(100);
    let card = card_for(ReceiptsTab::Dividend, ReceiptItem::Dividend(dividend));
    let (gross_text, gross_negative, gross_positive, gross_profit) = detail_value(&card, "配当金");
    assert_eq!(gross_text, "+¥500");
    assert!(!gross_negative);
    assert!(gross_positive);
    assert!(gross_profit);
    let (net_text, net_negative, net_positive, net_profit) = detail_value(&card, "税引後");
    assert_eq!(net_text, "+¥400");
    assert!(!net_negative);
    assert!(net_positive);
    assert!(net_profit);
    let (tax_text, tax_negative, tax_positive, tax_profit) = detail_value(&card, "税額");
    assert_eq!(tax_text, "-¥100");
    assert!(tax_negative);
    assert!(!tax_positive);
    assert!(tax_profit);
}

#[test]
fn summary_band_keeps_primary_last_and_breakdown_order() {
    for tab in ReceiptsTab::ALL {
        let labels: Vec<&str> = tab
            .header_items()
            .into_iter()
            .map(|(label, _)| label)
            .collect();
        assert_eq!(labels.len(), 3);
        let primary = labels[2];
        let breakdown = &labels[..2];
        match tab {
            ReceiptsTab::Dividend => {
                assert_eq!(primary, "税引後");
                assert_eq!(breakdown, ["配当金", "税額"]);
            }
            ReceiptsTab::DomesticStock | ReceiptsTab::MutualFund => {
                assert_eq!(primary, "税引後");
                assert_eq!(breakdown[0], "実現損益");
                assert_eq!(breakdown[1], "税額");
            }
        }
    }
}

#[test]
fn summary_breakdown_labels_show_tax_basis() {
    assert_eq!(breakdown_label(DIVIDEND_AMOUNT), DIVIDEND_BEFORE_TAX);
    assert_eq!(breakdown_label(REALIZED_PROFIT_LOSS), REALIZED_BEFORE_TAX);
    assert_eq!(breakdown_label(TAX), TAX);
    assert_eq!(breakdown_label(AFTER_TAX), AFTER_TAX);
}

#[test]
fn summary_band_pc_and_mobile_share_tax_basis_labels() {
    for tab in ReceiptsTab::ALL {
        let items: Vec<(&str, bool)> = tab.header_items().into_iter().collect();
        let pc: Vec<&str> = items[..2]
            .iter()
            .map(|(label, _)| breakdown_label(label))
            .collect();
        match tab {
            ReceiptsTab::Dividend => assert_eq!(pc, [DIVIDEND_BEFORE_TAX, TAX]),
            ReceiptsTab::DomesticStock | ReceiptsTab::MutualFund => {
                assert_eq!(pc, [REALIZED_BEFORE_TAX, TAX])
            }
        }
    }
}

#[test]
fn band_breakdown_signs_match_detail_dialog() {
    use crate::ui::summary::SummaryTone;
    let (text, negative, positive, tone) = band_value(DIVIDEND_AMOUNT, dec!(2000));
    assert_eq!(text, "+¥2,000");
    assert!(!negative);
    assert!(positive);
    assert_eq!(tone, SummaryTone::Gain);
    let (text, negative, positive, tone) = band_value(REALIZED_PROFIT_LOSS, dec!(-500));
    assert_eq!(text, "-¥500");
    assert!(negative);
    assert!(!positive);
    assert_eq!(tone, SummaryTone::Loss);
    let (text, negative, positive, tone) = band_value(TAX, dec!(406));
    assert_eq!(text, "-¥406");
    assert!(negative);
    assert!(!positive);
    assert_eq!(tone, SummaryTone::Loss);
    let (text, negative, positive, tone) = band_value(TAX, dec!(-100));
    assert_eq!(text, "+¥100");
    assert!(!negative);
    assert!(positive);
    assert_eq!(tone, SummaryTone::Gain);
    let (text, negative, positive, tone) = band_value(TAX, Decimal::ZERO);
    assert_eq!(text, "¥0");
    assert!(!negative);
    assert!(!positive);
    assert_eq!(tone, SummaryTone::Neutral);
}

#[test]
fn row_group_band_share_after_tax_total() {
    // 行の主役・月見出し・帯はすべて税引後。行を足すと帯に合う
    let mut domestic_row = domestic().remove(0);
    if let ReceiptItem::DomesticStock(row) = &mut domestic_row {
        row.taxes = dec!(203);
        row.realized_profit_and_loss_after_tax = dec!(797);
    }
    let domestic_rows = vec![domestic_row];
    let domestic_card = card_for(ReceiptsTab::DomesticStock, domestic_rows[0].clone());
    assert_eq!(detail_value(&domestic_card, "税引後").0, "+¥797");
    let fund_card = card_for(ReceiptsTab::MutualFund, funds().remove(0));
    assert_eq!(detail_value(&fund_card, "税引後").0, "+¥900");
    let dividend_card = card_for(ReceiptsTab::Dividend, dividends().remove(0));
    assert_eq!(detail_value(&dividend_card, "税引後").0, "+¥400");
    for (tab, rows) in [
        (ReceiptsTab::Dividend, dividends()),
        (ReceiptsTab::DomesticStock, domestic_rows),
        (ReceiptsTab::MutualFund, funds()),
    ] {
        let band_after_tax = tab.client_summary_triple(&rows)[2];
        let row_sum: Decimal = rows.iter().map(|row| row.summary_amounts().2).sum();
        assert_eq!(band_after_tax, row_sum);
    }
}

#[test]
fn month_headline_uses_after_tax_total() {
    // 税引前と税引後が違う集計でも月見出しは税引後を選ぶ
    let stock = vec![
        ("実現損益", "+¥1,000".to_string()),
        ("税額", "¥203".to_string()),
        ("税引後", "+¥797".to_string()),
    ];
    assert_eq!(group_headline_value(&stock), "+¥797");
    let dividend = vec![
        ("配当金", "¥500".to_string()),
        ("税額", "¥100".to_string()),
        ("税引後", "¥400".to_string()),
    ];
    assert_eq!(group_headline_value(&dividend), "¥400");
}

#[test]
fn group_headline_amount_uses_delta_kind() {
    // 月見出しの金額は受取額・損益とも Delta の種別で出す
    let dividend = vec![
        ("配当金", "¥500".to_string()),
        ("税額", "¥100".to_string()),
        ("税引後", "¥400".to_string()),
    ];
    let item = group_headline_summary(ReceiptsTab::Dividend, &dividend).expect("見出しがある");
    assert_eq!(item.label, "受取");
    assert_eq!(item.value, "+¥400");
    assert_eq!(item.kind, AmountKind::Delta);
    assert!(item.positive);
    let stock = vec![
        ("実現損益", "+¥1,000".to_string()),
        ("税額", "¥203".to_string()),
        ("税引後", "+¥797".to_string()),
    ];
    let item = group_headline_summary(ReceiptsTab::DomesticStock, &stock).expect("見出しがある");
    assert_eq!(item.label, "損益");
    assert_eq!(item.value, "+¥797");
    assert_eq!(item.kind, AmountKind::Delta);
    assert!(item.positive);
    assert!(group_headline_summary(ReceiptsTab::Dividend, &[]).is_none());
}

#[test]
fn single_day_group_headline_uses_row_after_tax() {
    // 集計が無い単発の日はその行の税引後を見出しに出す
    let card = card_for(ReceiptsTab::DomesticStock, domestic().remove(0));
    let headline = single_row_headline(std::slice::from_ref(&card));
    assert_eq!(headline, [(AFTER_TAX, "+¥112".to_string())]);
    assert!(single_row_headline(&[]).is_empty());
}

#[test]
fn detail_dialog_keeps_aux_amounts_removed_from_row() {
    for tab in ReceiptsTab::ALL {
        let mut rows = match tab {
            ReceiptsTab::Dividend => dividends(),
            ReceiptsTab::DomesticStock => domestic(),
            ReceiptsTab::MutualFund => funds(),
        };
        let card = card_for(tab, rows.remove(0));
        let (left, right) = receipt_detail_cells(tab, &card);
        let (label, expected) = match tab {
            ReceiptsTab::Dividend => (BEFORE_TAX, detail_value(&card, DIVIDEND_AMOUNT).0),
            ReceiptsTab::DomesticStock => (SELLING_AMOUNT, detail_value(&card, SELLING_AMOUNT).0),
            ReceiptsTab::MutualFund => {
                (REDEMPTION_AMOUNT, detail_value(&card, REDEMPTION_AMOUNT).0)
            }
        };
        assert!(!expected.is_empty(), "{tab:?}: 行の補助の金額が無い");
        let shown = left
            .iter()
            .chain(right.iter())
            .find(|cell| cell.label == label)
            .unwrap_or_else(|| panic!("{tab:?}: ダイアログに{label}が無い"));
        assert_eq!(
            shown.value, expected,
            "{tab:?}: ダイアログは行と同じ補助の金額"
        );
    }
}
