use super::*;
use crate::api::dto::AssetBalance;
use crate::features::asset_balance::csv::AssetBalanceRow;
use crate::support::row::Row;
use crate::testing::asset_balance::balance_row;
use rust_decimal_macros::dec;

fn saved(prev_close: Option<rust_decimal::Decimal>) -> AssetBalanceRow {
    Row::Saved(AssetBalance {
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
    })
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
    let view = holding_view(&Row::Saved(balance));
    assert_eq!(view.day_change, Some(50.0));
}

#[test]
fn day_change_keeps_present_value_and_none_without_it() {
    // 値がある行はそのまま使い、基準日があっても前日終値がなければ出さない
    let mut with_value = balance_row(9904);
    with_value.daily_change = dec!(-20);
    assert_eq!(
        holding_view(&Row::Saved(with_value)).day_change,
        Some(-20.0)
    );

    let mut without_prev_close = balance_row(9904);
    without_prev_close.prev_close = None;
    without_prev_close.daily_change = dec!(30);
    assert_eq!(
        holding_view(&Row::Saved(without_prev_close)).day_change,
        None
    );
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
