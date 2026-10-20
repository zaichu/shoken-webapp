use super::*;
use crate::features::receipts::ReceiptItem;
use crate::support::row::Row::Saved;
use crate::testing::receipts::{dividends, domestic, funds};

#[test]
fn dividend_filter_uses_exact_fields_and_all_amount_fields() {
    let rows = dividends();
    for query in [
        "9910 2024",
        "架空電信電話",
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
    assert_eq!(
        filter_receipts(ReceiptsTab::Dividend, &rows, "991"),
        rows[..2],
        "991"
    );
}
#[test]
fn domestic_filter_matches_trade_date_and_amount_fields() {
    let rows = domestic();
    for query in [
        "9904",
        "架空自動車",
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
    for query in ["2024-03-05", "888", "112"] {
        assert!(
            filter_receipts(ReceiptsTab::DomesticStock, &rows, query).is_empty(),
            "{query}"
        );
    }
    assert_eq!(
        filter_receipts(ReceiptsTab::DomesticStock, &rows, "架空"),
        rows,
        "架空"
    );
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
    let query = "2024-03-05";
    assert!(
        filter_receipts(ReceiptsTab::MutualFund, &rows, query).is_empty(),
        "{query}"
    );
    assert_eq!(
        filter_receipts(ReceiptsTab::MutualFund, &rows, "Alpha"),
        rows,
        "Alpha"
    );
}
#[test]
fn full_width_names_match_half_width_query() {
    let rows = dividends();
    assert_eq!(
        filter_receipts(ReceiptsTab::Dividend, &rows, "TELE"),
        rows[1..2]
    );
    assert_eq!(
        filter_receipts(ReceiptsTab::Dividend, &rows, "ＴＥＬＥ"),
        rows[1..2]
    );
}

#[test]
fn fund_options_merge_names_differing_only_by_width() {
    let mut rows = funds();
    let mut twin = match &rows[0] {
        Saved(ReceiptItem::MutualFund(row)) => row.clone(),
        _ => panic!("fund row"),
    };
    twin.id = "fund2".to_string().into();
    twin.fund_name = "Ａｌｐｈａ Ｆｕｎｄ Ａ".into();
    rows.push(Saved(ReceiptItem::MutualFund(twin)));
    let categories = search_categories(ReceiptsTab::MutualFund, &rows);
    assert_eq!(
        categories
            .securities
            .iter()
            .map(|option| (option.value.as_str(), option.label.as_str()))
            .collect::<Vec<_>>(),
        [("Alpha Fund A", "Alpha Fund A")]
    );
    let query = "\"Alpha Fund A\"";
    assert_eq!(
        filter_receipts(ReceiptsTab::MutualFund, &rows, query).len(),
        2
    );
    assert_eq!(
        ReceiptsTab::MutualFund
            .table_groups(&rows, &rows, query)
            .len(),
        1
    );
}

#[test]
fn promoted_column_matches_full_width_query() {
    let rows = dividends();
    assert_eq!(
        promoted_column(ReceiptsTab::Dividend, &rows, "ｎｉｓａ"),
        Some(2)
    );
    let order = column_order(ReceiptsTab::Dividend, &rows, "ｎｉｓａ");
    assert_eq!(&order[..2], &[0, 2]);
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
fn categories_use_latest_name_sorted_dates_and_tab_fields() {
    let result = search_categories(ReceiptsTab::Dividend, &dividends());
    assert!(result.dates);
    assert_eq!(
        result
            .securities
            .iter()
            .map(|o| o.label.as_str())
            .collect::<Vec<_>>(),
        ["9904: 架空自動車", "9910: TELE"]
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
fn combined_query_uses_fixed_order_and_quotes_whitespace() {
    let queries = SelectedQueries {
        date: " 2026-06 ".into(),
        securities: "9911: ABCD".into(),
        years: "2025".into(),
        products: "国内 株式".into(),
        accounts: "特定・一般".into(),
    };
    assert_eq!(
        build_combined_query(&queries),
        "2026-06 \"9911: ABCD\" 2025 \"国内 株式\" 特定・一般"
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
fn date_and_year_queries_keep_search_keys_separate() {
    let mut state = ReceiptSearch::new(true);
    state.select_year("2026".into());
    assert_eq!(state.query, "2026");
    assert!(state.selected_queries.years.is_empty());

    state.select_quick(SearchKey::Years, "2025".into());
    assert_eq!(state.query, "2026 2025");
}

#[test]
fn range_query_supports_empty_and_open_ranges() {
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

    state.select_quick(SearchKey::Securities, "9911".into());
    state.select_quick(SearchKey::Products, "国内株式".into());
    state.date_inputs.month_value = "2026-06".into();
    state.set_date_query("2026-06".into());
    assert_eq!(state.query, "2026-06 9911 国内株式");

    state.select_quick(SearchKey::Products, "国内株式".into());
    assert_eq!(state.query, "2026-06 9911");
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
fn range_inputs_and_clear_reset_search_state() {
    let mut state = ReceiptSearch::new(true);
    state.change_date_segment(DateSegment::Range);
    state.date_inputs.range_start = "2026-06-01".into();
    state.update_range_query();
    assert_eq!(state.query, "2026-06-01..");
    state.date_inputs.range_end = "2026-06-30".into();
    state.update_range_query();
    assert_eq!(state.query, "2026-06-01..2026-06-30");

    state.clear(false);
    assert!(state.query.is_empty());
    assert_eq!(state.selected_queries, SelectedQueries::default());
    assert_eq!(state.date_inputs, DateInputs::default());
    assert_eq!(state.date_segment, DateSegment::Month);
}
#[test]
fn selected_queries_is_empty_only_when_all_fields_blank() {
    assert!(SelectedQueries::default().is_empty());
    for key in [
        SearchKey::Securities,
        SearchKey::Years,
        SearchKey::Products,
        SearchKey::Accounts,
        SearchKey::Date,
    ] {
        let mut queries = SelectedQueries::default();
        queries.set(key, "x".into());
        assert!(!queries.is_empty(), "{key:?}");
    }
}

#[test]
fn default_search_state_is_default() {
    assert!(ReceiptSearch::new(true).is_default());
    let mut state = ReceiptSearch::new(true);
    state
        .selected_queries
        .set(SearchKey::Accounts, "特定".into());
    state.rebuild_query();
    assert!(!state.is_default());
}

#[test]
fn date_all_separates_no_period_from_the_year_segment() {
    let mut state = ReceiptSearch::new(true);
    assert!(state.date_all, "初期状態は期間なしで絞り込まない");

    // 既定のセグメントが年のまま「年」を選んでも「すべて」と区別できる
    state.change_date_segment(DateSegment::Year);
    assert!(!state.date_all);

    state.clear_key(SearchKey::Date, true);
    assert!(state.date_all);
    assert_eq!(state.date_segment, DateSegment::Year);
    assert!(state.query.is_empty());

    state.select_year("2026".into());
    assert!(!state.date_all);
    assert_eq!(state.query, "2026");
}

#[test]
fn set_date_updates_input_query_and_selected() {
    let mut state = ReceiptSearch::new(true);
    state.date_inputs.date_value = "2026-06-15".into();
    state.set_date_query("2026-06-15".into());
    assert_eq!(state.date_inputs.date_value, "2026-06-15");
    assert_eq!(state.selected_queries.date, "2026-06-15");
    assert_eq!(state.query, "2026-06-15");
}

#[test]
fn month_segment_keeps_selected_month_when_year_changes() {
    let mut state = ReceiptSearch::new(true);
    state.change_date_segment(DateSegment::Month);
    state.select_year("2023".into());
    state.date_inputs.month_value = "2023-06".into();
    state.set_date_query("2023-06".into());
    state.date_inputs.month_value = "06".to_string();
    assert_eq!(state.query, "2023-06");

    state.select_year("2024".into());
    assert_eq!(state.query, "2024-06");
    assert_eq!(state.date_inputs.month_value, "06");

    // 年を外すと月だけでは期間を作れないので日付条件は空になる
    state.select_year(String::new());
    assert_eq!(state.query, "");
    assert_eq!(state.date_inputs.month_value, "06");
}

#[test]
fn year_segment_ignores_leftover_month_input() {
    let mut state = ReceiptSearch::new(true);
    state.date_inputs.month_value = "06".to_string();
    state.select_year("2024".into());
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
