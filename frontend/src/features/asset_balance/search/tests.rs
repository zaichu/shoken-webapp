use super::{filter_asset_balances, stock_accounts};
use crate::api::dto::AssetBalance;
use crate::features::asset_balance::csv::AssetBalanceRow;
use crate::features::asset_balance::fund::AssetSearch;
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
        account: None,
        shares: Decimal::ZERO,
        average_purchase_price: Decimal::ZERO,
        total_purchase_amount: Decimal::ZERO,
        current_price: Decimal::ZERO,
        daily_change: Decimal::ZERO,
        price_as_of: None,
        prev_close: None,
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
        let search = AssetSearch {
            text: case.query.clone(),
            ..Default::default()
        };
        let actual: Vec<_> = filter_asset_balances(&data, &search)
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
    let search = |text: &str| AssetSearch {
        text: text.to_string(),
        ..Default::default()
    };
    assert_eq!(filter_asset_balances(&data, &search(&case.query)).len(), 1);
    let actual: Vec<_> = filter_asset_balances(&data, &search(""))
        .iter()
        .map(|row| row_id(row))
        .collect();
    assert_eq!(actual, case.expected_ids_all());
}

#[test]
fn account_filter_and_options() {
    let mut named = balance(&Row {
        id: "with-account".to_string(),
        security_code: "9904".to_string(),
        security_name: "架空自動車".to_string(),
    });
    named.account = Some("特定".parse().unwrap());
    let rows = vec![
        ListRow::Saved(named),
        ListRow::Saved(balance(&Row {
            id: "no-account".to_string(),
            security_code: "9901".to_string(),
            security_name: "架空資源開発".to_string(),
        })),
    ];

    assert_eq!(stock_accounts(&rows), ["特定".to_string()]);

    // 口座を選んだときは口座なしの行は除く
    let search = AssetSearch {
        accounts: vec!["特定".to_string()],
        ..Default::default()
    };
    let actual: Vec<_> = filter_asset_balances(&rows, &search)
        .iter()
        .map(|row| row_id(row))
        .collect();
    assert_eq!(actual, ["with-account"]);

    // テキストは口座名でも部分一致する
    let search = AssetSearch {
        text: "特定".to_string(),
        ..Default::default()
    };
    let actual: Vec<_> = filter_asset_balances(&rows, &search)
        .iter()
        .map(|row| row_id(row))
        .collect();
    assert_eq!(actual, ["with-account"]);
}
