use super::*;
use crate::api::dto::AssetBalance;
use crate::features::asset_balance::csv::AssetBalanceRow;
use crate::testing::asset_balance::balance_row;
use rust_decimal_macros::dec;

#[test]
fn known_zero_dividend_has_zero_yield() {
    let mut maps = DividendMaps::default();
    maps.status.insert("9904".to_string(), "zero".to_string());
    for average_price in [2500.0, 0.0] {
        let dividend = holding_dividend("9904", 100.0, average_price, &maps);
        assert_eq!(dividend.per_share, Some(0.0));
        assert_eq!(dividend.annual, Some(0.0));
        assert_eq!(dividend.yield_value, Some(0.0));
    }
}

#[test]
fn missing_dividend_keeps_yield_unknown() {
    for status in [None, Some("pending"), Some("error")] {
        let mut maps = DividendMaps::default();
        if let Some(status) = status {
            maps.status.insert("9904".to_string(), status.to_string());
        }
        let dividend = holding_dividend("9904", 100.0, 2500.0, &maps);
        assert!(dividend.per_share.is_none());
        assert!(dividend.annual.is_none());
        assert!(dividend.yield_value.is_none());
    }
}

fn saved(prev_close: Option<rust_decimal::Decimal>) -> AssetBalanceRow {
    AssetBalance {
        id: "id1".to_string().into(),
        security_code: "9904".parse().unwrap(),
        security_name: "架空自動車".to_string(),
        account: None,
        shares: dec!(100),
        average_purchase_price: dec!(2500),
        total_purchase_amount: dec!(250000),
        current_price: dec!(2600),
        daily_change: dec!(50),
        price_as_of: Some("2024-01-16".to_string()),
        prev_close,
        created_at: String::new(),
        updated_at: String::new(),
    }
}

fn view(day_change: Option<f64>, shares: f64) -> HoldingView {
    HoldingView {
        id: None,
        code: "9904".to_string(),
        name: "架空自動車".to_string(),
        account: None,
        shares,
        average_price: 2500.0,
        purchase: 250_000.0,
        current_price: 2600.0,
        day_change,
        price_as_of: Some("2024-01-16".to_string()),
    }
}

#[test]
fn day_change_maps_per_share_value() {
    let view = holding_view(&saved(Some(dec!(2550))));
    assert_eq!(view.day_change, Some(50.0));
}

#[test]
fn day_change_is_none_without_prev_close() {
    // 上場直後・取得失敗で前日終値が取れない銘柄は「—」・合計から除く
    let view = holding_view(&saved(None));
    assert_eq!(view.day_change, None);
}

#[test]
fn day_change_uses_row_value_without_price_date() {
    // 基準日を持たない行は前日終値の有無を見ず取込値をそのまま使う
    let mut balance = balance_row(9904);
    balance.price_as_of = None;
    let view = holding_view(&balance);
    assert_eq!(view.day_change, Some(50.0));
}

#[test]
fn day_change_keeps_present_value_and_none_without_it() {
    // 値がある行はそのまま使い、基準日があっても前日終値がなければ出さない
    let mut with_value = balance_row(9904);
    with_value.daily_change = dec!(-20);
    assert_eq!(holding_view(&with_value).day_change, Some(-20.0));

    let mut without_prev_close = balance_row(9904);
    without_prev_close.prev_close = None;
    without_prev_close.daily_change = dec!(30);
    assert_eq!(holding_view(&without_prev_close).day_change, None);
}

#[test]
fn daily_change_total_is_weighted_by_shares() {
    let views = vec![
        view(Some(50.0), 100.0),
        view(Some(-20.0), 50.0),
        view(Some(30.0), 1.0),
    ];
    // 50×100 + (-20)×50 + 30×1 = 4030
    assert_eq!(stock_daily_change_total(&views), Some(4030.0));
}

#[test]
fn daily_change_total_skips_rows_without_prev_close() {
    let views = vec![view(Some(50.0), 100.0), view(None, 50.0)];
    assert_eq!(stock_daily_change_total(&views), Some(5000.0));
}

#[test]
fn daily_change_total_is_none_when_all_rows_lack_prev_close() {
    let views = vec![view(None, 100.0), view(None, 50.0)];
    assert_eq!(stock_daily_change_total(&views), None);
    assert_eq!(stock_daily_change_total(&[]), None);
}

#[test]
fn stock_href_points_to_search_page_with_code() {
    assert_eq!(holding_stock_href("9904"), "/search?code=9904");
}

fn grouped_view(code: &str, account: Option<&str>, shares: f64, purchase: f64) -> HoldingView {
    HoldingView {
        id: None,
        code: code.to_string(),
        name: "架空自動車".to_string(),
        account: account.map(str::to_string),
        shares,
        average_price: 2500.0,
        purchase,
        current_price: 2600.0,
        day_change: None,
        price_as_of: Some("2024-01-16".to_string()),
    }
}

#[test]
fn group_holdings_collects_same_code_keeping_first_order() {
    let views = vec![
        grouped_view("9904", Some("特定"), 100.0, 250_000.0),
        grouped_view("9914", Some("特定"), 50.0, 150_000.0),
        grouped_view("9904", Some("NISA成長"), 50.0, 130_000.0),
    ];
    let groups = group_holdings_by_code(&views);
    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0].code, "9904");
    assert_eq!(groups[0].rows.len(), 2);
    assert_eq!(groups[0].total_shares, 150.0);
    assert_eq!(groups[0].total_purchase, 380_000.0);
    assert_eq!(groups[0].average_price(), 380_000.0 / 150.0);
    assert_eq!(groups[1].code, "9914");
    assert_eq!(groups[1].rows.len(), 1);
}
