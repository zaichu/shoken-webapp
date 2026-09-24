#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::{CsvPreviewResponse, CsvRowError};
    use rust_decimal_macros::dec;

    fn preview_response(rows: Vec<serde_json::Value>) -> CsvPreviewResponse {
        CsvPreviewResponse {
            total_rows: 2,
            valid_rows: 1,
            errors: vec![CsvRowError {
                row: 2,
                message: "保有数量［株］の形式が不正です".to_string(),
            }],
            rows,
        }
    }

    #[test]
    fn preview_response_maps_to_asset_balance_rows() {
        let response = preview_response(vec![serde_json::json!({
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
        })]);
        let preview = to_preview(response);
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
        assert_eq!(row.daily_change, dec!(50));
        assert_eq!(row.market_value, dec!(260000));
        assert_eq!(row.profit_loss_rate, dec!(4.0));
    }

    #[test]
    fn preview_row_with_missing_fields_falls_back() {
        let response = preview_response(vec![
            serde_json::Value::Null,
            serde_json::json!({"security_name": "ソニーグループ"}),
        ]);
        let preview = to_preview(response);
        assert_eq!(preview.rows.len(), 2);
        assert_eq!(preview.rows[0], AssetBalanceCsvRow::default());
        assert_eq!(preview.rows[1].security_name, "ソニーグループ");
        assert_eq!(preview.rows[1].shares, dec!(0));
        assert_eq!(preview.rows[1].market_value, dec!(0));
    }

    #[test]
    fn csv_row_converts_to_asset_balance_with_empty_identity() {
        let row = AssetBalanceCsvRow {
            security_code: "7203".to_string(),
            security_name: "トヨタ自動車".to_string(),
            shares: dec!(100),
            average_purchase_price: dec!(2500),
            ..Default::default()
        };
        let balance = row.to_asset_balance();
        assert!(balance.id.is_empty());
        assert!(balance.created_at.is_empty());
        assert!(balance.updated_at.is_empty());
        assert_eq!(balance.security_code, "7203");
        assert_eq!(balance.security_name, "トヨタ自動車");
        assert_eq!(balance.shares, dec!(100));
        assert_eq!(balance.average_purchase_price, dec!(2500));
    }

    #[test]
    fn api_paths_match_backend_routes() {
        assert_eq!(LIST_PATH, "/api/v1/asset-balances");
        assert_eq!(PREVIEW_PATH, "/api/v1/asset-balance-import-validations");
        assert_eq!(IMPORT_PATH, "/api/v1/asset-balance-imports");
    }
}
