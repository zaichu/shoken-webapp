use super::*;
use crate::api::dto::CsvRowError;
use rust_decimal_macros::dec;

fn preview_response(rows: Vec<serde_json::Value>) -> CsvPreviewResponse {
    CsvPreviewResponse {
        total_rows: 2,
        valid_rows: 1,
        errors: vec![CsvRowError {
            row: 2,
            message: "入金日の形式が不正です".to_string(),
        }],
        rows,
    }
}

#[test]
fn preview_response_maps_to_typed_rows_per_tab() {
    let dividend = preview_response(vec![serde_json::json!({
        "settlement_date": "2024-03-01",
        "product": "特定口座",
        "account": "SBI証券",
        "security_code": "7203",
        "security_name": "トヨタ自動車",
        "unit_price": 30.0,
        "shares": 100,
        "dividends_before_tax": 3000,
        "taxes": 609,
        "net_amount_received": 2391
    })]);
    let preview = to_preview(ReceiptsTab::Dividend, dividend);
    assert_eq!((preview.total_rows, preview.valid_rows), (2, 1));
    assert_eq!(preview.errors.len(), 1);
    let CsvPreviewRow::Dividend(row) = &preview.rows[0] else {
        panic!("dividend row expected")
    };
    assert_eq!(row.security_name, "トヨタ自動車");
    assert_eq!(row.unit_price, dec!(30.0));
    assert_eq!(row.shares, dec!(100));

    let domestic = preview_response(vec![serde_json::json!({
        "trade_date": "2024-01-15",
        "settlement_date": "2024-01-17",
        "security_code": "1301",
        "security_name": "極洋",
        "account": "特定",
        "shares": 100,
        "asked_price": 1500.5,
        "proceeds": 150050,
        "purchase_price": 1400.25,
        "realized_profit_and_loss": 10000.1,
        "taxes": 2031.5,
        "realized_profit_and_loss_after_tax": 7968.6
    })]);
    let preview = to_preview(ReceiptsTab::DomesticStock, domestic);
    let CsvPreviewRow::DomesticStock(row) = &preview.rows[0] else {
        panic!("domestic stock row expected")
    };
    assert_eq!(row.security_code, "1301");
    assert_eq!(row.proceeds, dec!(150050));

    let mutualfund = preview_response(vec![serde_json::json!({
        "trade_date": "2024-01-15",
        "settlement_date": "2024-01-17",
        "fund_name": "eMAXIS Slim 全世界株式",
        "dividends": "再投資型",
        "account": "楽天証券",
        "shares": 10000,
        "exchange_rate": 150.25,
        "cancellation_unit_price_yen": 12345,
        "cancellation_amount_yen": 120000,
        "average_acquisition_price_yen": 11000.5,
        "realized_profit_and_loss": 12000,
        "taxes": 2437,
        "realized_profit_and_loss_after_tax": 9563
    })]);
    let preview = to_preview(ReceiptsTab::MutualFund, mutualfund);
    let CsvPreviewRow::MutualFund(row) = &preview.rows[0] else {
        panic!("mutualfund row expected")
    };
    assert_eq!(row.fund_name, "eMAXIS Slim 全世界株式");
    assert_eq!(row.dividends.as_deref(), Some("再投資型"));
    assert_eq!(row.cancellation_amount_yen, dec!(120000));
}

#[test]
fn preview_row_with_missing_fields_falls_back_like_react() {
    let response = preview_response(vec![
        serde_json::Value::Null,
        serde_json::json!({"security_name": "トヨタ自動車"}),
    ]);
    let preview = to_preview(ReceiptsTab::Dividend, response);
    assert_eq!(preview.rows.len(), 2);
    let CsvPreviewRow::Dividend(row) = &preview.rows[1] else {
        panic!("dividend row expected")
    };
    assert_eq!(row.security_name, "トヨタ自動車");
    assert_eq!(row.unit_price, dec!(0));
}

#[test]
fn preview_rows_convert_to_receipt_items_like_react_transform() {
    let dividend = CsvPreviewRow::Dividend(DividendCsvRow {
        settlement_date: "2024-03-01".to_string(),
        product: "特定口座".to_string(),
        security_name: "トヨタ自動車".to_string(),
        net_amount_received: dec!(2391),
        ..Default::default()
    });
    let ReceiptItem::Dividend(item) = ReceiptItem::from(dividend) else {
        panic!("dividend item expected")
    };
    assert_eq!(item.security_name, "トヨタ自動車");
    assert_eq!(item.net_amount_received, dec!(2391));
    assert!(item.id.is_empty());
    assert!(item.created_at.is_empty());
    assert!(item.updated_at.is_empty());

    let domestic = CsvPreviewRow::DomesticStock(DomesticStockCsvRow {
        trade_date: "2024-02-01".to_string(),
        security_name: "任天堂".to_string(),
        realized_profit_and_loss_after_tax: dec!(3985),
        ..Default::default()
    });
    let ReceiptItem::DomesticStock(item) = ReceiptItem::from(domestic) else {
        panic!("domestic stock item expected")
    };
    assert_eq!(item.security_name, "任天堂");
    assert_eq!(item.realized_profit_and_loss_after_tax, dec!(3985));

    let fund = CsvPreviewRow::MutualFund(MutualfundCsvRow {
        fund_name: "eMAXIS Slim 全世界株式".to_string(),
        dividends: None,
        ..Default::default()
    });
    let ReceiptItem::MutualFund(item) = ReceiptItem::from(fund) else {
        panic!("mutual fund item expected")
    };
    assert_eq!(item.fund_name, "eMAXIS Slim 全世界株式");
    assert_eq!(item.dividends.as_deref(), Some(""));
}

#[test]
fn preview_rows_show_unparseable_code_and_account_verbatim() {
    // newtype の検証に通らない値も、"0" や "-" へ差し替えず元の文字列を表示する
    let domestic = CsvPreviewRow::DomesticStock(DomesticStockCsvRow {
        security_code: "7203-1".to_string(),
        account: String::new(),
        ..Default::default()
    });
    let ReceiptItem::DomesticStock(item) = ReceiptItem::from(domestic) else {
        panic!("domestic stock item expected")
    };
    assert_eq!(item.security_code.as_str(), "7203-1");
    assert_eq!(item.account.as_str(), "");
}

fn arb_decimal() -> impl proptest::strategy::Strategy<Value = Decimal> {
    use proptest::strategy::Strategy;
    (-9_999_999_999_999i64..9_999_999_999_999i64).prop_map(|mantissa| Decimal::new(mantissa, 2))
}

proptest::proptest! {
    #[test]
    fn prop_dividend_to_receipt_item_copies_fields(
        date in "[ -~]{0,12}",
        product in ".*",
        account in ".{1,100}",
        code in ".*",
        name in ".*",
        unit_price in arb_decimal(),
        shares in arb_decimal(),
        before_tax in arb_decimal(),
        taxes in arb_decimal(),
        net in arb_decimal(),
    ) {
        let row = DividendCsvRow {
            settlement_date: date,
            product,
            account,
            security_code: code,
            security_name: name,
            unit_price,
            shares,
            dividends_before_tax: before_tax,
            taxes,
            net_amount_received: net,
        };
        let expected = row.clone();
        let ReceiptItem::Dividend(item) =
            ReceiptItem::from(CsvPreviewRow::Dividend(row))
        else {
            panic!("dividend item expected")
        };
        proptest::prop_assert_eq!(item.settlement_date, expected.settlement_date);
        proptest::prop_assert_eq!(item.product, expected.product);
        proptest::prop_assert_eq!(item.account.as_str(), expected.account);
        proptest::prop_assert_eq!(item.security_code, expected.security_code);
        proptest::prop_assert_eq!(item.security_name, expected.security_name);
        proptest::prop_assert_eq!(item.unit_price, expected.unit_price);
        proptest::prop_assert_eq!(item.shares, expected.shares);
        proptest::prop_assert_eq!(item.dividends_before_tax, expected.dividends_before_tax);
        proptest::prop_assert_eq!(item.taxes, expected.taxes);
        proptest::prop_assert_eq!(item.net_amount_received, expected.net_amount_received);
    }

    #[test]
    fn prop_to_preview_preserves_row_count(
        rows in proptest::collection::vec(proptest::bool::ANY, 0..8usize),
    ) {
        // bool はどの行型にもデシリアライズできずデフォルト行になる
        let row_count = rows.len();
        let response = CsvPreviewResponse {
            total_rows: row_count,
            valid_rows: 0,
            errors: vec![],
            rows: rows.into_iter().map(serde_json::Value::Bool).collect(),
        };
        for tab in [
            ReceiptsTab::Dividend,
            ReceiptsTab::DomesticStock,
            ReceiptsTab::MutualFund,
        ] {
            let preview = to_preview(tab, response.clone());
            proptest::prop_assert_eq!(preview.rows.len(), row_count);
            match tab {
                ReceiptsTab::Dividend => {
                    for row in &preview.rows {
                        proptest::prop_assert_eq!(
                            row,
                            &CsvPreviewRow::Dividend(DividendCsvRow::default())
                        );
                    }
                }
                ReceiptsTab::DomesticStock => {
                    for row in &preview.rows {
                        proptest::prop_assert_eq!(
                            row,
                            &CsvPreviewRow::DomesticStock(DomesticStockCsvRow::default())
                        );
                    }
                }
                ReceiptsTab::MutualFund => {
                    for row in &preview.rows {
                        proptest::prop_assert_eq!(
                            row,
                            &CsvPreviewRow::MutualFund(MutualfundCsvRow::default())
                        );
                    }
                }
            }
        }
    }
}
