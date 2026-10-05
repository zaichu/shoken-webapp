use crate::api::dto::{Dividend, DomesticStock, Mutualfund};
use crate::features::receipts::{ReceiptItem, ReceiptRow};
use crate::support::row::Row::Saved;

pub(crate) fn dividends() -> Vec<ReceiptRow> {
    let base: Dividend = serde_json::from_value(serde_json::json!({"id":"old", "settlement_date":"2024-06-21", "product":"国内株式", "account":"特定", "security_code":"9432", "security_name":"日本電信電話", "unit_price":5, "shares":100, "dividends_before_tax":500, "taxes":100, "net_amount_received":400, "created_at":"", "updated_at":""})).unwrap();
    let mut latest = base.clone();
    latest.id = "new".to_string().into();
    latest.settlement_date = "2026-06-01".into();
    latest.security_name = "ＮＴＴ".into();
    let mut other = latest.clone();
    other.id = "other".to_string().into();
    other.security_code = "7203".into();
    other.security_name = "トヨタ自動車".into();
    other.account = "NISA".parse().unwrap();
    vec![
        Saved(ReceiptItem::Dividend(base)),
        Saved(ReceiptItem::Dividend(latest)),
        Saved(ReceiptItem::Dividend(other)),
    ]
}
pub(crate) fn domestic() -> Vec<ReceiptRow> {
    let row: DomesticStock = serde_json::from_value(serde_json::json!({"id":"stock", "trade_date":"2024-03-01", "settlement_date":"2024-03-05", "security_code":"7203", "security_name":"トヨタ自動車", "account":"特定", "shares":100, "asked_price":2000, "proceeds":200000, "purchase_price":199000, "realized_profit_and_loss":1000, "taxes":888, "realized_profit_and_loss_after_tax":112, "created_at":"", "updated_at":""})).unwrap();
    vec![Saved(ReceiptItem::DomesticStock(row))]
}
pub(crate) fn funds() -> Vec<ReceiptRow> {
    let row: Mutualfund = serde_json::from_value(serde_json::json!({"id":"fund", "trade_date":"2024-03-01", "settlement_date":"2024-03-05", "fund_name":"Alpha Fund A", "account":"NISA", "shares":100, "exchange_rate":1, "cancellation_unit_price_yen":100, "cancellation_amount_yen":1000, "average_acquisition_price_yen":100, "realized_profit_and_loss":900, "taxes":0, "realized_profit_and_loss_after_tax":900, "created_at":"", "updated_at":""})).unwrap();
    vec![Saved(ReceiptItem::MutualFund(row))]
}
