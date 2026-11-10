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
    assert!(!STOCK_LINKS.is_empty());
    for (name, template) in STOCK_LINKS {
        assert!(!name.is_empty());
        assert!(template.contains("{code}"));
        assert!(template.starts_with("https://"));
    }
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
fn yield_tile_wide_only_for_odd_detail_counts() {
    assert!(super::yield_tile_wide(true, true, true));
    assert!(!super::yield_tile_wide(true, true, false));
    assert!(super::yield_tile_wide(true, false, false));
    assert!(super::yield_tile_wide(false, true, false));
}
