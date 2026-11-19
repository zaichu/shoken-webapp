use super::*;
use crate::features::asset_balance::HoldingGroup;
use crate::features::asset_balance::HoldingView;

fn tile_group(rows: Vec<HoldingView>) -> HoldingGroup {
    let total_shares = rows.iter().map(|row| row.shares).sum();
    let total_purchase = rows.iter().map(|row| row.purchase).sum();
    HoldingGroup {
        code: "9904".to_string(),
        name: "架空自動車".to_string(),
        current_price: 2600.0,
        price_as_of: Some("2026-01-01".to_string()),
        rows,
        total_shares,
        total_purchase,
    }
}

fn tile_row(account: &str, shares: f64, purchase: f64) -> HoldingView {
    HoldingView {
        id: None,
        code: "9904".to_string(),
        name: "架空自動車".to_string(),
        account: Some(account.to_string()),
        shares,
        average_price: purchase / shares,
        purchase,
        current_price: 2600.0,
        day_change: None,
        price_as_of: Some("2026-01-01".to_string()),
    }
}

fn zero_maps(code: &str) -> DividendMaps {
    let mut maps = DividendMaps {
        responded: true,
        ..DividendMaps::default()
    };
    maps.status.insert(code.to_string(), "zero".to_string());
    maps
}

fn ok_maps(code: &str, per_share: f64) -> DividendMaps {
    let mut maps = DividendMaps {
        responded: true,
        ..DividendMaps::default()
    };
    maps.status.insert(code.to_string(), "ok".to_string());
    maps.per_share.insert(code.to_string(), per_share);
    maps
}

#[test]
fn holding_total_tiles_collapse_zero_dividends_into_single_item() {
    let group = tile_group(vec![tile_row("特定", 100.0, 250_000.0)]);
    let zero = HoldingTilesData::total(&group, &zero_maps("9904"));
    assert!(zero.no_dividend);
    assert_eq!(zero.annual, labels::NO_DIVIDEND);
    let ok = HoldingTilesData::total(&group, &ok_maps("9904", 90.0));
    assert!(!ok.no_dividend);
    assert_ne!(ok.annual, labels::NO_DIVIDEND);
}

#[test]
fn holding_account_tiles_collapse_zero_dividends_into_single_item() {
    let group = tile_group(vec![
        tile_row("特定", 100.0, 250_000.0),
        tile_row("NISA成長", 50.0, 130_000.0),
    ]);
    for row in &group.rows {
        let zero = HoldingTilesData::account(row, &group, &zero_maps("9904"));
        assert!(zero.no_dividend);
        assert_eq!(zero.annual, labels::NO_DIVIDEND);
        let ok = HoldingTilesData::account(row, &group, &ok_maps("9904", 90.0));
        assert!(!ok.no_dividend);
        assert_ne!(ok.annual, labels::NO_DIVIDEND);
    }
}

#[test]
fn holding_total_tiles_sum_accounts_without_unit_price() {
    let group = tile_group(vec![
        tile_row("特定", 100.0, 250_000.0),
        tile_row("NISA成長", 50.0, 130_000.0),
    ]);
    let data = HoldingTilesData::total(&group, &DividendMaps::default());
    assert!(data.market.contains("390,000"));
    assert!(data.pnl.contains("10,000"));
    assert!(data.quantity.contains("150"));
    assert!(data.purchase.contains("380,000"));
    assert!(data.average_price.is_none());
    assert!(data.account.is_none());
    assert!(data.show_common);
}

#[test]
fn holding_account_tiles_hide_common_items() {
    let group = tile_group(vec![
        tile_row("特定", 100.0, 250_000.0),
        tile_row("NISA成長", 50.0, 130_000.0),
    ]);
    let total = HoldingTilesData::total(&group, &DividendMaps::default());
    assert!(total.show_common);
    for row in &group.rows {
        let account = HoldingTilesData::account(row, &group, &DividendMaps::default());
        assert!(!account.show_common);
    }
}

#[test]
fn holding_single_account_tiles_keep_unit_price_and_badge() {
    let group = tile_group(vec![tile_row("特定", 100.0, 250_000.0)]);
    let data = HoldingTilesData::total(&group, &DividendMaps::default());
    assert!(data.average_price.is_some());
    assert_eq!(data.account.as_deref(), Some("特定"));
    assert!(data.show_common);

    let account = HoldingTilesData::account(&group.rows[0], &group, &DividendMaps::default());
    assert!(account.average_price.is_some());
    assert_eq!(account.account.as_deref(), Some("特定"));
    assert!(account.quantity.contains("100"));
}

#[test]
fn stock_links_contain_code_placeholder() {
    assert_ne!(STOCK_LINKS, []);
    for (name, template) in STOCK_LINKS {
        assert_ne!(*name, "");
        assert!(template.contains("{code}"));
        assert!(template.starts_with("https://"));
    }
}

#[test]
fn stock_links_lead_with_main_three_and_keep_all() {
    assert_eq!(STOCK_LINKS.len(), 11);
    let names: Vec<&str> = STOCK_LINKS.iter().map(|(name, _)| *name).collect();
    assert_eq!(
        &names[..3],
        &[
            labels::STOCK_LINK_RAKUTEN,
            labels::STOCK_LINK_KABUTAN,
            labels::STOCK_LINK_YAHOO,
        ]
    );
    assert_eq!(names.len() - 3, 8);
}

#[test]
fn empty_search_description_differs_for_initial_and_no_results() {
    assert_eq!(
        empty_search_description("銘柄を検索"),
        "銘柄コード（例：7203）または銘柄名を入力して検索してください。"
    );
    assert_eq!(
        empty_search_description("該当する銘柄が見つかりませんでした"),
        "銘柄コードまたは銘柄名を確認してください。"
    );
}

#[test]
fn dividend_tile_spans_avoid_single_item_rows() {
    // 2列(スマホ)は奇数、4列(PC)は4で割って1余るときだけ全幅にする
    assert_eq!(super::dividend_tile_spans(9), (true, true));
    assert_eq!(super::dividend_tile_spans(8), (false, false));
    assert_eq!(super::dividend_tile_spans(7), (true, false));
    assert_eq!(super::dividend_tile_spans(5), (true, true));
    assert_eq!(super::dividend_tile_spans(6), (false, false));
    assert_eq!(super::dividend_tile_spans(4), (false, false));
}

#[test]
fn detail_item_count_matches_rendered_tiles() {
    assert_eq!(super::detail_item_count(true, true, true, false), 9);
    assert_eq!(super::detail_item_count(true, true, false, false), 8);
    assert_eq!(super::detail_item_count(true, false, false, false), 7);
    assert_eq!(super::detail_item_count(false, true, false, false), 5);
}

#[test]
fn yield_tile_wide_counts_collapsed_dividend_as_single_item() {
    // 合計は3項目→1項目で2減、口座ごとは2項目→1項目で1減
    assert_eq!(super::detail_item_count(true, true, true, true), 7);
    assert_eq!(super::detail_item_count(true, true, false, true), 6);
    assert_eq!(super::detail_item_count(true, false, false, true), 5);
    assert_eq!(super::detail_item_count(false, true, false, true), 4);
}

#[test]
fn search_body_state_prefers_search_error_first() {
    assert_eq!(
        super::resolve_search_body_state(true, true, true, true, true),
        super::SearchBodyState::SearchFailed
    );
    assert_eq!(
        super::resolve_search_body_state(true, false, false, false, false),
        super::SearchBodyState::SearchFailed
    );
}

#[test]
fn search_body_state_prefers_balance_error_over_result() {
    assert_eq!(
        super::resolve_search_body_state(false, true, true, false, false),
        super::SearchBodyState::BalanceFailed
    );
    assert_eq!(
        super::resolve_search_body_state(false, true, false, true, false),
        super::SearchBodyState::BalanceFailed
    );
}

#[test]
fn search_body_state_orders_found_not_found_initial_loading() {
    assert_eq!(
        super::resolve_search_body_state(false, false, true, true, true),
        super::SearchBodyState::Found
    );
    assert_eq!(
        super::resolve_search_body_state(false, false, false, true, true),
        super::SearchBodyState::NotFound
    );
    assert_eq!(
        super::resolve_search_body_state(false, false, false, false, false),
        super::SearchBodyState::Initial
    );
    assert_eq!(
        super::resolve_search_body_state(false, false, false, false, true),
        super::SearchBodyState::Loading
    );
}
