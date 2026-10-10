use super::*;
use crate::api::dto::CsvPreviewResponse;
use crate::support::row::Row;
use rust_decimal_macros::dec;

#[test]
fn preview_response_deserializes_wire_rows() {
    // backend は CreateAssetBalanceRequest を直列化して rows に入れる。
    // ワイヤー形そのままの JSON が CsvPreviewResponse<AssetBalanceCsvRow> に落ちることを固定する
    let preview: CsvPreviewResponse<AssetBalanceCsvRow> =
        serde_json::from_value(serde_json::json!({
            "total_rows": 2,
            "valid_rows": 1,
            "errors": [{"row": 2, "message": "保有数量［株］の形式が不正です"}],
            "rows": [{
                "security_code": "7203",
                "security_name": "トヨタ自動車",
                "shares": 100,
                "executing_shares": 0,
                "average_purchase_price": 2500,
                "total_purchase_amount": 250000,
                "current_price": 2600,
                "daily_change": 50,
                "market_value": 260000,
                "profit_loss_rate": 4.0
            }]
        }))
        .expect("preview response");
    assert_eq!((preview.total_rows, preview.valid_rows), (2, 1));
    assert_eq!(preview.errors.len(), 1);
    let row = &preview.rows[0];
    assert_eq!(row.security_code, "7203");
    assert_eq!(row.security_name, "トヨタ自動車");
    assert_eq!(row.shares, dec!(100));
    assert_eq!(row.executing_shares, dec!(0));
    assert_eq!(row.average_purchase_price, dec!(2500));
    assert_eq!(row.total_purchase_amount, dec!(250000));
    assert_eq!(row.current_price, dec!(2600));
}

#[test]
fn preview_row_with_missing_fields_uses_defaults() {
    // フィールド欠落は serde(default) で穴埋めする。行自体が null など
    // オブジェクトでない場合はレスポンス全体がパースエラーになる
    let row: AssetBalanceCsvRow =
        serde_json::from_value(serde_json::json!({"security_name": "ソニーグループ"}))
            .expect("lenient row");
    assert_eq!(row.security_name, "ソニーグループ");
    assert_eq!(row.shares, dec!(0));
    assert!(serde_json::from_value::<AssetBalanceCsvRow>(serde_json::Value::Null).is_err());
}

#[test]
fn preview_row_exposes_csv_fields_without_saved_identity() {
    let row = Row::Preview(AssetBalanceCsvRow {
        security_code: "7203".to_string(),
        security_name: "トヨタ自動車".to_string(),
        shares: dec!(100),
        average_purchase_price: dec!(2500),
        ..Default::default()
    });
    assert!(row.is_preview());
    assert!(row.saved().is_none());
    assert_eq!(row.security_code(), "7203");
    assert_eq!(row.security_name(), "トヨタ自動車");
    assert_eq!(row.shares(), dec!(100));
    assert_eq!(row.average_purchase_price(), dec!(2500));
}

#[test]
fn csv_row_shows_unparseable_security_code_verbatim() {
    // 検証に通らない銘柄コードも "0" へ差し替えず元の文字列を表示する
    let row = Row::Preview(AssetBalanceCsvRow {
        security_code: "7203-1".to_string(),
        ..Default::default()
    });
    assert_eq!(row.security_code(), "7203-1");
}

#[test]
fn api_paths_match_backend_routes() {
    assert_eq!(LIST_PATH, "/api/v1/asset-balances");
    assert_eq!(PREVIEW_PATH, "/api/v1/asset-balance-import-validations");
    assert_eq!(IMPORT_PATH, "/api/v1/asset-balance-imports");
}

#[test]
fn asset_balance_row_saved_delegates_current_price() {
    use crate::api::dto::AssetBalance;
    use crate::support::row::Row::Saved;

    let balance = AssetBalance {
        id: "id1".to_string().into(),
        security_code: "7203".parse().unwrap(),
        security_name: "トヨタ自動車".to_string(),
        shares: dec!(100),
        executing_shares: dec!(0),
        average_purchase_price: dec!(2500),
        total_purchase_amount: dec!(250000),
        current_price: dec!(2600),
        daily_change: dec!(50),
        created_at: "2024-01-01T00:00:00Z".to_string(),
        updated_at: "2024-01-01T00:00:00Z".to_string(),
    };
    let row: AssetBalanceRow = Saved(balance);

    assert_eq!(row.current_price(), dec!(2600));
}

#[test]
fn asset_balance_row_preview_delegates_current_price() {
    use crate::support::row::Row::Preview;

    let row: AssetBalanceRow = Preview(AssetBalanceCsvRow {
        security_code: "7203".to_string(),
        security_name: "トヨタ自動車".to_string(),
        shares: dec!(100),
        executing_shares: dec!(0),
        average_purchase_price: dec!(2500),
        total_purchase_amount: dec!(250000),
        current_price: dec!(2700),
    });

    assert_eq!(row.current_price(), dec!(2700));
}

#[test]
fn asset_balance_csv_row_current_price_field() {
    let row = AssetBalanceCsvRow {
        security_code: "7203".to_string(),
        security_name: "トヨタ自動車".to_string(),
        shares: dec!(100),
        executing_shares: dec!(0),
        average_purchase_price: dec!(2500),
        total_purchase_amount: dec!(250000),
        current_price: dec!(2800),
    };

    assert_eq!(row.current_price(), dec!(2800));
}

#[test]
fn asset_balance_saved_row_current_price_field() {
    use crate::api::dto::AssetBalance;

    let row = AssetBalance {
        id: "id1".to_string().into(),
        security_code: "7203".parse().unwrap(),
        security_name: "トヨタ自動車".to_string(),
        shares: dec!(100),
        executing_shares: dec!(0),
        average_purchase_price: dec!(2500),
        total_purchase_amount: dec!(250000),
        current_price: dec!(2900),
        daily_change: dec!(50),
        created_at: "2024-01-01T00:00:00Z".to_string(),
        updated_at: "2024-01-01T00:00:00Z".to_string(),
    };

    assert_eq!(row.current_price(), dec!(2900));
}
