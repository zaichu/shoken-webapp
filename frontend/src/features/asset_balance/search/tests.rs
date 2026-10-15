use super::filter_asset_balances;
use crate::api::dto::AssetBalance;
use crate::features::asset_balance::csv::AssetBalanceRow;
use crate::support::row::Row as ListRow;
use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Deserialize)]
struct Row {
    id: String,
    security_code: String,
    security_name: String,
}

#[derive(Deserialize)]
struct FilterCase {
    name: String,
    data: Vec<Row>,
    query: String,
    expected_ids: Vec<String>,
}

#[derive(Deserialize)]
struct Fixture {
    filter_cases: Vec<FilterCase>,
}

fn fixture() -> Fixture {
    serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/asset_balance/search.json"
    )))
    .expect("shared asset balance search fixture parses")
}

fn balance(row: &Row) -> AssetBalance {
    AssetBalance {
        id: row.id.clone().into(),
        security_code: row.security_code.parse().unwrap(),
        security_name: row.security_name.clone(),
        shares: Decimal::ZERO,
        average_purchase_price: Decimal::ZERO,
        total_purchase_amount: Decimal::ZERO,
        current_price: Decimal::ZERO,
        daily_change: Decimal::ZERO,
        price_as_of: None,
        created_at: String::new(),
        updated_at: String::new(),
    }
}

fn balances(rows: &[Row]) -> Vec<AssetBalanceRow> {
    rows.iter()
        .map(|row| ListRow::Saved(balance(row)))
        .collect()
}

fn row_id(row: &AssetBalanceRow) -> String {
    row.saved().map(|saved| saved.id.to_string()).unwrap()
}

impl FilterCase {
    fn expected_ids_all(&self) -> Vec<String> {
        self.data.iter().map(|row| row.id.clone()).collect()
    }
}

#[test]
fn shared_filter_cases_match() {
    let fixture = fixture();
    assert_eq!(fixture.filter_cases.len(), 14);
    for case in &fixture.filter_cases {
        let data = balances(&case.data);
        let actual: Vec<_> = filter_asset_balances(&data, &case.query)
            .iter()
            .map(|row| row_id(row))
            .collect();
        assert_eq!(actual, case.expected_ids, "{}", case.name);
    }
}

#[test]
fn cleared_query_restores_all_rows() {
    let fixture = fixture();
    let case = &fixture.filter_cases[2];
    let data = balances(&case.data);
    assert_eq!(filter_asset_balances(&data, &case.query).len(), 1);
    let actual: Vec<_> = filter_asset_balances(&data, "")
        .iter()
        .map(|row| row_id(row))
        .collect();
    assert_eq!(actual, case.expected_ids_all());
}
