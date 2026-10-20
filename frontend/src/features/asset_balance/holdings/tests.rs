use super::*;
use crate::api::dto::AssetBalance;
use crate::features::asset_balance::csv::{AssetBalanceCsvRow, AssetBalanceRow};
use crate::support::row::Row;
use rust_decimal_macros::dec;

fn saved(prev_close: Option<rust_decimal::Decimal>) -> AssetBalanceRow {
    Row::Saved(AssetBalance {
        id: "id1".to_string().into(),
        security_code: "9904".parse().unwrap(),
        security_name: "架空自動車".to_string(),
        shares: dec!(100),
        average_purchase_price: dec!(2500),
        total_purchase_amount: dec!(250000),
        current_price: dec!(2600),
        daily_change: dec!(50),
        price_as_of: Some("2024-01-16".to_string()),
        prev_close,
        created_at: String::new(),
        updated_at: String::new(),
    })
}

fn view(day_change: Option<f64>, shares: f64) -> HoldingView {
    HoldingView {
        id: None,
        code: "9904".to_string(),
        name: "架空自動車".to_string(),
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
fn day_change_uses_csv_value_when_no_shared_price() {
    // price_as_of がない(CSV 取込値のまま)行は csv_daily_change を使う
    let mut row = saved(None);
    if let Row::Saved(ref mut balance) = row {
        balance.price_as_of = None;
    }
    let view = holding_view(&row);
    assert_eq!(view.day_change, Some(50.0));
}

#[test]
fn day_change_none_when_csv_preview_has_no_value() {
    let row: AssetBalanceRow = Row::Preview(AssetBalanceCsvRow {
        security_code: "9904".to_string(),
        shares: dec!(100),
        ..Default::default()
    });
    assert_eq!(holding_view(&row).day_change, None);
    let row: AssetBalanceRow = Row::Preview(AssetBalanceCsvRow {
        security_code: "9904".to_string(),
        daily_change: Some(dec!(-20)),
        ..Default::default()
    });
    assert_eq!(holding_view(&row).day_change, Some(-20.0));
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
