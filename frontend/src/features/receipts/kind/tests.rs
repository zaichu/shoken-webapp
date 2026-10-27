use super::*;
use crate::features::receipts::kind::product_matches;
use crate::features::receipts::kind::{ReceiptItem, ReceiptRow, Row};
use rust_decimal::Decimal;

fn saved_dividend_row() -> ReceiptRow {
    Row::Saved(ReceiptItem::Dividend(
        serde_json::from_value(serde_json::json!({
            "id": "id1",
            "settlement_date": "2024-06-15",
            "product": "NISA",
            "account": "特定",
            "security_code": "9910",
            "security_name": "TELE",
            "unit_price": 150,
            "shares": 100,
            "dividends_before_tax": 5000,
            "taxes": 1000,
            "net_amount_received": 4000,
            "created_at": "2024-06-15T00:00:00Z",
            "updated_at": "2024-06-15T00:00:00Z"
        }))
        .expect("dividend"),
    ))
}

fn saved_domestic_row() -> ReceiptRow {
    Row::Saved(ReceiptItem::DomesticStock(
        serde_json::from_value(serde_json::json!({
            "id": "id2",
            "trade_date": "2024-06-15",
            "settlement_date": "2024-06-15",
            "security_code": "9904",
            "security_name": "架空",
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
        .expect("domestic stock"),
    ))
}

fn saved_mutualfund_row() -> ReceiptRow {
    Row::Saved(ReceiptItem::MutualFund(
        serde_json::from_value(serde_json::json!({
            "id": "id3",
            "trade_date": "2024-06-15",
            "settlement_date": "2024-06-15",
            "fund_name": "ABC Fund",
            "dividends": "0",
            "account": "特定",
            "shares": 10000,
            "exchange_rate": 100,
            "cancellation_unit_price_yen": 15000,
            "cancellation_amount_yen": 1500000,
            "average_acquisition_price_yen": 14000,
            "realized_profit_and_loss": 100000,
            "taxes": 20000,
            "realized_profit_and_loss_after_tax": 80000,
            "created_at": "2024-06-15T00:00:00Z",
            "updated_at": "2024-06-15T00:00:00Z"
        }))
        .expect("mutualfund"),
    ))
}

#[test]
fn tab_columns_and_card_fields_match_row_cells() {
    for (tab, row) in [
        (ReceiptsTab::Dividend, saved_dividend_row()),
        (ReceiptsTab::DomesticStock, saved_domestic_row()),
        (ReceiptsTab::MutualFund, saved_mutualfund_row()),
    ] {
        let count = row.cells().len();
        assert_eq!(tab.columns().len(), count, "{tab:?}");
        let fields = tab.card_fields();
        for field in [fields.name, fields.date, fields.account] {
            assert!(field < count, "{tab:?}: {field}");
        }
    }
}

#[test]
fn tab_filters_keep_distinct_amount_and_category_rules() {
    for tab in ReceiptsTab::ALL {
        let config = tab.filter_config();
        assert_eq!(
            config.string_fields.unwrap().len(),
            match tab {
                ReceiptsTab::Dividend => 4,
                ReceiptsTab::DomesticStock => 3,
                ReceiptsTab::MutualFund => 2,
            }
        );
        assert_eq!(
            config.amount_fields.map(|fields| fields.len()),
            match tab {
                ReceiptsTab::Dividend | ReceiptsTab::DomesticStock => Some(5),
                ReceiptsTab::MutualFund => None,
            }
        );
        assert_eq!(tab.product_category(), tab == ReceiptsTab::Dividend);
        assert_eq!(tab.account_category(), tab != ReceiptsTab::MutualFund);
        assert!(
            config.year_search
                && config.year_month_search
                && config.date_search
                && config.date_range_search
        );
    }
}

#[test]
fn api_summaries_accept_only_the_matching_tab() {
    let values = [
        Decimal::new(100, 0),
        Decimal::new(20, 0),
        Decimal::new(80, 0),
    ];
    let summaries = [
        ReceiptSummary::Dividend(DividendSummary {
            total_dividends_before_tax: values[0],
            total_taxes: values[1],
            total_net_amount_received: values[2],
        }),
        ReceiptSummary::DomesticStock(DomesticStockSummary {
            total_realized_profit_and_loss: values[0],
            total_taxes: values[1],
            total_realized_profit_and_loss_after_tax: values[2],
        }),
        ReceiptSummary::MutualFund(MutualfundSummary {
            total_realized_profit_and_loss: values[0],
            total_taxes: values[1],
            total_realized_profit_and_loss_after_tax: values[2],
        }),
    ];
    for (index, tab) in ReceiptsTab::ALL.into_iter().enumerate() {
        for (summary_index, summary) in summaries.iter().enumerate() {
            assert_eq!(
                tab.api_summary_triple(summary),
                (index == summary_index).then_some(values)
            );
        }
    }
}

#[test]
fn dividend_totals_keeps_signed_amounts_and_empty_totals() {
    let row = |before_tax: i64, taxes: i64, net: i64| {
        Row::Saved(ReceiptItem::Dividend(
            serde_json::from_value(serde_json::json!({
                "id": "id1",
                "settlement_date": "2024-06-15",
                "product": "特定口座",
                "account": "特定",
                "security_code": "9910",
                "security_name": "TELE",
                "unit_price": 30.0,
                "shares": 100,
                "dividends_before_tax": before_tax,
                "taxes": taxes,
                "net_amount_received": net,
                "created_at": "2024-06-15T00:00:00Z",
                "updated_at": "2024-06-15T00:00:00Z"
            }))
            .expect("dividend"),
        ))
    };
    let rows = vec![row(15000, 3047, 11953), row(-5000, -1000, -4000)];
    assert_eq!(
        dividend_totals(&rows),
        DividendSummary {
            total_dividends_before_tax: Decimal::from(10000),
            total_taxes: Decimal::from(2047),
            total_net_amount_received: Decimal::from(7953),
        }
    );
    assert_eq!(
        dividend_totals(&[]),
        DividendSummary {
            total_dividends_before_tax: Decimal::ZERO,
            total_taxes: Decimal::ZERO,
            total_net_amount_received: Decimal::ZERO,
        }
    );
}

#[test]
fn receipt_item_delegates_all_accessors() {
    let item = ReceiptItem::Dividend(
        serde_json::from_value(serde_json::json!({
            "id": "id1",
            "settlement_date": "2024-06-15",
            "product": "NISA",
            "account": "特定",
            "security_code": "9910",
            "security_name": "TELE",
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
    assert_eq!(item.code(), "9910");
    assert_eq!(item.name(), "TELE");
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
    let row = saved_domestic_row();

    assert_eq!(row.date(), "2024-06-15");
    assert_eq!(row.code(), "9904");
    assert_eq!(row.name(), "架空");
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
fn receipt_row_dividend_saved_delegates_all_accessors() {
    let row = saved_dividend_row();

    assert_eq!(row.date(), "2024-06-15");
    assert_eq!(row.code(), "9910");
    assert_eq!(row.name(), "TELE");
    assert_eq!(row.account(), "特定");
    assert_eq!(row.product(), "NISA");
    assert!(row.is_specific());
    assert!(!row.cells().is_empty());
    assert!(!row.raw_key().is_empty());
    assert_eq!(row.search_amount(0), Decimal::from(150));
    let (a, b, c) = row.summary_amounts();
    assert_eq!(a, Decimal::from(5000));
    assert_eq!(b, Decimal::from(1000));
    assert_eq!(c, Decimal::from(4000));
    assert_eq!(row.realized_pnl(), Decimal::ZERO);
    assert_eq!(row.saved_id(), Some("id1"));
}

#[test]
fn mutualfund_item_delegates_all_accessors() {
    let item = ReceiptItem::MutualFund(
        serde_json::from_value(serde_json::json!({
            "id": "id3",
            "trade_date": "2024-06-15",
            "settlement_date": "2024-06-15",
            "fund_name": "ABC Fund",
            "dividends": "0",
            "account": "特定",
            "shares": 10000,
            "exchange_rate": 100,
            "cancellation_unit_price_yen": 15000,
            "cancellation_amount_yen": 1500000,
            "average_acquisition_price_yen": 14000,
            "realized_profit_and_loss": 100000,
            "taxes": 20000,
            "realized_profit_and_loss_after_tax": 80000,
            "created_at": "2024-06-15T00:00:00Z",
            "updated_at": "2024-06-15T00:00:00Z"
        }))
        .expect("deserialize"),
    );

    assert_eq!(item.date(), "2024-06-15");
    assert_eq!(item.code(), "");
    assert_eq!(item.name(), "ABC Fund");
    assert_eq!(item.account(), "特定");
    assert_eq!(item.product(), "");
    assert!(item.is_specific());
    assert!(!item.cells().is_empty());
    assert!(!item.raw_key().is_empty());
    assert_eq!(item.search_amount(0), Decimal::ZERO);
    let (a, b, c) = item.summary_amounts();
    assert_eq!(a, Decimal::from(100000));
    assert_eq!(b, Decimal::from(20000));
    assert_eq!(c, Decimal::from(80000));
    assert_eq!(item.realized_pnl(), Decimal::ZERO);
}

#[test]
fn product_matches_returns_true_for_matching_product() {
    let row = saved_dividend_row();
    assert!(product_matches(&row, "nisa"));
}
