use super::*;
use crate::api::dto::AssetBalance;
use crate::support::row::Row::Saved;
use rust_decimal_macros::dec;

#[test]
fn api_paths_match_backend_routes() {
    assert_eq!(LIST_PATH, "/api/v1/asset-balances");
    assert_eq!(PREVIEW_PATH, "/api/v1/asset-balance-import-validations");
    assert_eq!(IMPORT_PATH, "/api/v1/asset-balance-imports");
}

#[test]
fn asset_balance_row_saved_delegates_current_price() {
    let balance = AssetBalance {
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
        prev_close: Some(dec!(2550)),
        created_at: "2024-01-01T00:00:00Z".to_string(),
        updated_at: "2024-01-01T00:00:00Z".to_string(),
    };
    let row: AssetBalanceRow = Saved(balance);

    assert_eq!(row.current_price(), dec!(2600));
}

#[test]
fn asset_balance_saved_row_current_price_field() {
    let row = AssetBalance {
        id: "id1".to_string().into(),
        security_code: "9904".parse().unwrap(),
        security_name: "架空自動車".to_string(),
        account: None,
        shares: dec!(100),
        average_purchase_price: dec!(2500),
        total_purchase_amount: dec!(250000),
        current_price: dec!(2900),
        daily_change: dec!(50),
        price_as_of: Some("2024-01-16".to_string()),
        prev_close: Some(dec!(2850)),
        created_at: "2024-01-01T00:00:00Z".to_string(),
        updated_at: "2024-01-01T00:00:00Z".to_string(),
    };

    assert_eq!(row.current_price(), dec!(2900));
}
