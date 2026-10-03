use super::*;
use crate::features::receipts::kind::product_matches;
use crate::features::receipts::kind::{
    CsvPreviewRow, DividendCsvRow, DomesticStockCsvRow, MutualfundCsvRow, ReceiptItem, ReceiptRow,
    Row,
};
use rust_decimal::Decimal;

#[test]
fn dividend_csv_row_cells_not_empty() {
    let row = DividendCsvRow {
        settlement_date: "2024-06-15".to_string(),
        product: "NISA".to_string(),
        account: "特定".to_string(),
        security_code: "9432".to_string(),
        security_name: "NTT".to_string(),
        unit_price: Decimal::new(15000, 2),
        shares: Decimal::new(100, 0),
        dividends_before_tax: Decimal::new(5000, 2),
        taxes: Decimal::new(1000, 2),
        net_amount_received: Decimal::new(4000, 2),
    };
    let cells = row.cells();
    assert!(!cells.is_empty());
    assert_eq!(cells.len(), 10);
}

#[test]
fn csv_preview_row_delegates_all_accessors() {
    let row = CsvPreviewRow::Dividend(DividendCsvRow {
        settlement_date: "2024-06-15".to_string(),
        product: "NISA".to_string(),
        account: "特定".to_string(),
        security_code: "9432".to_string(),
        security_name: "NTT".to_string(),
        unit_price: Decimal::new(15000, 2),
        shares: Decimal::new(100, 0),
        dividends_before_tax: Decimal::new(5000, 2),
        taxes: Decimal::new(1000, 2),
        net_amount_received: Decimal::new(4000, 2),
    });

    assert_eq!(row.date(), "2024-06-15");
    assert_eq!(row.code(), "9432");
    assert_eq!(row.name(), "NTT");
    assert_eq!(row.account(), "特定");
    assert_eq!(row.product(), "NISA");
    assert!(row.is_specific());
    assert!(!row.cells().is_empty());
    assert!(!row.raw_key().is_empty());
    assert_eq!(row.search_amount(0), Decimal::new(15000, 2));
    let (a, b, c) = row.summary_amounts();
    assert_eq!(a, Decimal::new(5000, 2));
    assert_eq!(b, Decimal::new(1000, 2));
    assert_eq!(c, Decimal::new(4000, 2));
    assert_eq!(row.realized_pnl(), Decimal::ZERO);
}

#[test]
fn csv_preview_row_domestic_stock_delegates_all_accessors() {
    let row = CsvPreviewRow::DomesticStock(DomesticStockCsvRow {
        trade_date: "2024-06-15".to_string(),
        settlement_date: "2024-06-15".to_string(),
        security_code: "7203".to_string(),
        security_name: "トヨタ".to_string(),
        account: "NISA".to_string(),
        shares: Decimal::new(100, 0),
        asked_price: Decimal::new(3000, 2),
        proceeds: Decimal::new(300000, 2),
        purchase_price: Decimal::new(2500, 2),
        realized_profit_and_loss: Decimal::new(50000, 2),
        taxes: Decimal::new(10000, 2),
        realized_profit_and_loss_after_tax: Decimal::new(40000, 2),
    });

    assert_eq!(row.date(), "2024-06-15");
    assert_eq!(row.code(), "7203");
    assert_eq!(row.name(), "トヨタ");
    assert_eq!(row.account(), "NISA");
    assert_eq!(row.product(), "");
    assert!(!row.is_specific());
    assert!(!row.cells().is_empty());
    assert!(!row.raw_key().is_empty());
    assert_eq!(row.search_amount(0), Decimal::new(100, 0));
    let (a, b, c) = row.summary_amounts();
    assert_eq!(a, Decimal::new(50000, 2));
    assert_eq!(b, Decimal::new(10000, 2));
    assert_eq!(c, Decimal::new(40000, 2));
    assert_eq!(row.realized_pnl(), Decimal::new(50000, 2));
}

#[test]
fn csv_preview_row_mutualfund_delegates_all_accessors() {
    let row = CsvPreviewRow::MutualFund(MutualfundCsvRow {
        trade_date: "2024-06-15".to_string(),
        settlement_date: "2024-06-15".to_string(),
        fund_name: "ABC Fund".to_string(),
        dividends: Some("0".to_string()),
        account: "特定".to_string(),
        shares: Decimal::new(10000, 2),
        exchange_rate: Decimal::new(100, 0),
        cancellation_unit_price_yen: Decimal::new(15000, 2),
        cancellation_amount_yen: Decimal::new(1500000, 2),
        average_acquisition_price_yen: Decimal::new(14000, 2),
        realized_profit_and_loss: Decimal::new(100000, 2),
        taxes: Decimal::new(20000, 2),
        realized_profit_and_loss_after_tax: Decimal::new(80000, 2),
    });

    assert_eq!(row.date(), "2024-06-15");
    assert_eq!(row.code(), "");
    assert_eq!(row.name(), "ABC Fund");
    assert_eq!(row.account(), "特定");
    assert_eq!(row.product(), "");
    assert!(row.is_specific());
    assert!(!row.cells().is_empty());
    assert!(!row.raw_key().is_empty());
    assert_eq!(row.search_amount(0), Decimal::ZERO);
    let (a, b, c) = row.summary_amounts();
    assert_eq!(a, Decimal::new(100000, 2));
    assert_eq!(b, Decimal::new(20000, 2));
    assert_eq!(c, Decimal::new(80000, 2));
    assert_eq!(row.realized_pnl(), Decimal::ZERO);
}

#[test]
fn receipt_item_delegates_all_accessors() {
    let item = ReceiptItem::Dividend(
        serde_json::from_value(serde_json::json!({
            "id": "id1",
            "settlement_date": "2024-06-15",
            "product": "NISA",
            "account": "特定",
            "security_code": "9432",
            "security_name": "NTT",
            "unit_price": 150.00,
            "shares": 100,
            "dividends_before_tax": 50.00,
            "taxes": 10.00,
            "net_amount_received": 40.00,
            "created_at": "2024-06-15T00:00:00Z",
            "updated_at": "2024-06-15T00:00:00Z"
        }))
        .expect("deserialize"),
    );

    assert_eq!(item.date(), "2024-06-15");
    assert_eq!(item.code(), "9432");
    assert_eq!(item.name(), "NTT");
    assert_eq!(item.account(), "特定");
    assert_eq!(item.product(), "NISA");
    assert!(item.is_specific());
    assert!(!item.cells().is_empty());
    assert!(!item.raw_key().is_empty());
    assert_eq!(item.search_amount(0), Decimal::new(15000, 2));
    let (a, b, c) = item.summary_amounts();
    assert_eq!(a, Decimal::new(5000, 2));
    assert_eq!(b, Decimal::new(1000, 2));
    assert_eq!(c, Decimal::new(4000, 2));
    assert_eq!(item.realized_pnl(), Decimal::ZERO);
}

#[test]
fn receipt_row_saved_delegates_all_accessors() {
    let item = ReceiptItem::DomesticStock(
        serde_json::from_value(serde_json::json!({
            "id": "id2",
            "trade_date": "2024-06-15",
            "settlement_date": "2024-06-15",
            "security_code": "7203",
            "security_name": "トヨタ",
            "account": "NISA",
            "shares": 100,
            "asked_price": 3000.00,
            "proceeds": 300000.00,
            "purchase_price": 2500.00,
            "realized_profit_and_loss": 50000.00,
            "taxes": 10000.00,
            "realized_profit_and_loss_after_tax": 40000.00,
            "created_at": "2024-06-15T00:00:00Z",
            "updated_at": "2024-06-15T00:00:00Z"
        }))
        .expect("deserialize"),
    );
    let row: ReceiptRow = Row::Saved(item);

    assert_eq!(row.date(), "2024-06-15");
    assert_eq!(row.code(), "7203");
    assert_eq!(row.name(), "トヨタ");
    assert_eq!(row.account(), "NISA");
    assert_eq!(row.product(), "");
    assert!(!row.is_specific());
    assert!(!row.cells().is_empty());
    assert!(!row.raw_key().is_empty());
    assert_eq!(row.search_amount(0), Decimal::new(100, 0));
    let (a, b, c) = row.summary_amounts();
    assert_eq!(a, Decimal::new(5000000, 2));
    assert_eq!(b, Decimal::new(1000000, 2));
    assert_eq!(c, Decimal::new(4000000, 2));
    assert_eq!(row.realized_pnl(), Decimal::new(5000000, 2));
    assert_eq!(row.saved_id(), Some("id2"));
}

#[test]
fn receipt_row_preview_delegates_all_accessors() {
    let row: ReceiptRow = Row::Preview(CsvPreviewRow::Dividend(DividendCsvRow {
        settlement_date: "2024-06-15".to_string(),
        product: "NISA".to_string(),
        account: "特定".to_string(),
        security_code: "9432".to_string(),
        security_name: "NTT".to_string(),
        unit_price: Decimal::new(15000, 2),
        shares: Decimal::new(100, 0),
        dividends_before_tax: Decimal::new(5000, 2),
        taxes: Decimal::new(1000, 2),
        net_amount_received: Decimal::new(4000, 2),
    }));

    assert_eq!(row.date(), "2024-06-15");
    assert_eq!(row.code(), "9432");
    assert_eq!(row.name(), "NTT");
    assert_eq!(row.account(), "特定");
    assert_eq!(row.product(), "NISA");
    assert!(row.is_specific());
    assert!(!row.cells().is_empty());
    assert!(!row.raw_key().is_empty());
    assert_eq!(row.search_amount(0), Decimal::new(15000, 2));
    let (a, b, c) = row.summary_amounts();
    assert_eq!(a, Decimal::new(5000, 2));
    assert_eq!(b, Decimal::new(1000, 2));
    assert_eq!(c, Decimal::new(4000, 2));
    assert_eq!(row.realized_pnl(), Decimal::ZERO);
    assert_eq!(row.saved_id(), None);
}

#[test]
fn domestic_stock_csv_row_cells_not_empty() {
    let row = DomesticStockCsvRow {
        trade_date: "2024-06-15".to_string(),
        settlement_date: "2024-06-15".to_string(),
        security_code: "9432".to_string(),
        security_name: "NTT".to_string(),
        account: "特定".to_string(),
        shares: Decimal::new(100, 0),
        asked_price: Decimal::new(15000, 2),
        proceeds: Decimal::new(1500000, 2),
        purchase_price: Decimal::new(14000, 2),
        realized_profit_and_loss: Decimal::new(100000, 2),
        taxes: Decimal::new(20000, 2),
        realized_profit_and_loss_after_tax: Decimal::new(80000, 2),
    };
    let cells = row.cells();
    assert!(!cells.is_empty());
    assert_eq!(cells.len(), 11);
}

#[test]
fn mutualfund_csv_row_cells_not_empty() {
    let row = MutualfundCsvRow {
        trade_date: "2024-06-15".to_string(),
        settlement_date: "2024-06-15".to_string(),
        fund_name: "ABC Fund".to_string(),
        dividends: Some("0".to_string()),
        account: "特定".to_string(),
        shares: Decimal::new(10000, 2),
        exchange_rate: Decimal::new(100, 0),
        cancellation_unit_price_yen: Decimal::new(15000, 2),
        cancellation_amount_yen: Decimal::new(1500000, 2),
        average_acquisition_price_yen: Decimal::new(14000, 2),
        realized_profit_and_loss: Decimal::new(100000, 2),
        taxes: Decimal::new(20000, 2),
        realized_profit_and_loss_after_tax: Decimal::new(80000, 2),
    };
    let cells = row.cells();
    assert!(!cells.is_empty());
    assert_eq!(cells.len(), 10);
}

#[test]
fn product_matches_returns_true_for_matching_product() {
    let row = ReceiptRow::Preview(CsvPreviewRow::Dividend(DividendCsvRow {
        settlement_date: "2024-06-15".to_string(),
        product: "NISA".to_string(),
        account: "特定".to_string(),
        security_code: "9432".to_string(),
        security_name: "NTT".to_string(),
        unit_price: Decimal::new(15000, 2),
        shares: Decimal::new(100, 0),
        dividends_before_tax: Decimal::new(5000, 2),
        taxes: Decimal::new(1000, 2),
        net_amount_received: Decimal::new(4000, 2),
    }));
    assert!(product_matches(&row, "nisa"));
}
