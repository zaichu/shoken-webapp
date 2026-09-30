use super::*;
use crate::features::receipts::kind::product_matches;
use crate::features::receipts::kind::{
    CsvPreviewRow, DividendCsvRow, DomesticStockCsvRow, MutualfundCsvRow, ReceiptRow,
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
