use super::{ChartPlan, SortInput, SortKey, SortOrder, chart_plan, sort_holdings};
use crate::features::asset_balance::holdings::HoldingDividend;
use crate::features::asset_balance::model::ValuationInput;
use serde::Deserialize;

const PERCENTAGE_TOLERANCE: f64 = 1e-9;

#[derive(Deserialize)]
struct FixtureItem {
    id: String,
    purchase: f64,
}

#[derive(Deserialize)]
struct OrderCase {
    name: String,
    items: Vec<FixtureItem>,
    expected_ids: Vec<String>,
    expected_percentages: Vec<Option<f64>>,
}

#[derive(Deserialize)]
struct Fixture {
    order_cases: Vec<OrderCase>,
}

fn fixture() -> Fixture {
    serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/asset_balance/portfolio.json"
    )))
    .expect("shared asset balance portfolio fixture parses")
}

fn plan_for(items: &[FixtureItem]) -> ChartPlan {
    let values: Vec<f64> = items.iter().map(|item| item.purchase).collect();
    chart_plan(&values)
}

fn sorted_ids<'a>(items: &'a [FixtureItem], plan: &ChartPlan) -> Vec<&'a str> {
    plan.order
        .iter()
        .map(|&index| items[index].id.as_str())
        .collect()
}

fn assert_percentage(actual: Option<f64>, expected: &Option<f64>, case: &str, index: usize) {
    match (actual, expected) {
        (None, None) => {}
        (Some(actual), Some(expected)) => {
            let diff = (actual - expected).abs();
            assert!(
                diff <= PERCENTAGE_TOLERANCE,
                "{case} percentages[{index}]: actual {actual} expected {expected}"
            );
        }
        (actual, expected) => {
            panic!("{case} percentages[{index}]: actual {actual:?} expected {expected:?}")
        }
    }
}

#[test]
fn shared_order_cases_match() {
    let fixture = fixture();
    assert_eq!(fixture.order_cases.len(), 7);
    for case in &fixture.order_cases {
        let plan = plan_for(&case.items);
        assert_eq!(
            sorted_ids(&case.items, &plan),
            case.expected_ids,
            "{}",
            case.name
        );
        assert_eq!(
            plan.percentages.len(),
            case.expected_percentages.len(),
            "{}",
            case.name
        );
        for (index, (actual, expected)) in plan
            .percentages
            .iter()
            .zip(&case.expected_percentages)
            .enumerate()
        {
            assert_percentage(*actual, expected, &case.name, index);
        }
    }
}

fn sort_input(code: &str, market: f64, purchase: f64, annual: Option<f64>) -> SortInput<'static> {
    let dividend = Box::leak(Box::new(HoldingDividend {
        per_share: annual.map(|v| v / 100.0),
        annual,
        yield_value: annual.map(|v| v / purchase * 100.0),
        status: None,
    }));
    SortInput {
        code: Box::leak(code.to_string().into_boxed_str()),
        valuation: ValuationInput {
            current_price: market,
            shares: 1.0,
            total_purchase_amount: purchase,
        },
        dividend: &*dividend,
    }
}

#[test]
fn sort_holdings_orders_by_market_value_desc() {
    let inputs = vec![
        sort_input("9904", 100.0, 80.0, Some(10.0)),
        sort_input("9914", 200.0, 150.0, Some(20.0)),
        sort_input("9999", 50.0, 40.0, Some(5.0)),
    ];
    let order = sort_holdings(inputs, SortKey::MarketValue, SortOrder::Desc);
    assert_eq!(order, vec![1, 0, 2]);
}

#[test]
fn sort_holdings_orders_by_profit_loss_asc() {
    let inputs = vec![
        sort_input("9904", 100.0, 80.0, Some(10.0)),
        sort_input("9914", 200.0, 150.0, Some(20.0)),
        sort_input("9999", 50.0, 40.0, Some(5.0)),
    ];
    let order = sort_holdings(inputs, SortKey::ProfitLoss, SortOrder::Asc);
    assert_eq!(order, vec![2, 0, 1]);
}

#[test]
fn sort_holdings_orders_by_dividend_desc() {
    let inputs = vec![
        sort_input("9904", 100.0, 80.0, Some(10.0)),
        sort_input("9914", 200.0, 150.0, Some(20.0)),
        sort_input("9999", 50.0, 40.0, Some(5.0)),
    ];
    let order = sort_holdings(inputs, SortKey::Dividend, SortOrder::Desc);
    assert_eq!(order, vec![1, 0, 2]);
}

#[test]
fn sort_holdings_orders_by_yield_asc() {
    let inputs = vec![
        sort_input("9904", 100.0, 80.0, Some(10.0)),
        sort_input("9914", 200.0, 150.0, Some(20.0)),
        sort_input("9999", 50.0, 40.0, Some(5.0)),
    ];
    let order = sort_holdings(inputs, SortKey::Yield, SortOrder::Asc);
    // 9904 と 9999 は利回り同値のためコード順
    assert_eq!(order, vec![0, 2, 1]);
}

#[test]
fn sort_holdings_places_uncomputable_at_end_regardless_of_order() {
    let inputs = vec![
        sort_input("9904", 100.0, 80.0, Some(10.0)),
        sort_input("9914", 0.0, 150.0, Some(20.0)),
        sort_input("9999", 50.0, 40.0, Some(5.0)),
    ];
    let desc = sort_holdings(inputs.clone(), SortKey::MarketValue, SortOrder::Desc);
    assert_eq!(desc, vec![0, 2, 1]);
    let asc = sort_holdings(inputs, SortKey::MarketValue, SortOrder::Asc);
    assert_eq!(asc, vec![2, 0, 1]);
}

#[test]
fn sort_holdings_breaks_ties_by_code() {
    let inputs = vec![
        sort_input("9904", 100.0, 80.0, Some(10.0)),
        sort_input("9914", 100.0, 80.0, Some(10.0)),
        sort_input("9999", 100.0, 80.0, Some(10.0)),
    ];
    let order = sort_holdings(inputs, SortKey::MarketValue, SortOrder::Desc);
    assert_eq!(order, vec![0, 1, 2]);
}

#[test]
fn sort_holdings_allows_other_keys_while_dividend_pending() {
    let mut pending = sort_input("0001", 100.0, 80.0, None);
    let pending_dividend = Box::leak(Box::new(HoldingDividend {
        per_share: None,
        annual: None,
        yield_value: None,
        status: Some("pending".to_string()),
    }));
    pending.dividend = &*pending_dividend;
    let inputs = vec![
        pending,
        sort_input("9904", 200.0, 150.0, Some(20.0)),
        sort_input("9999", 50.0, 40.0, Some(5.0)),
    ];
    let order = sort_holdings(inputs, SortKey::MarketValue, SortOrder::Desc);
    assert_eq!(order, vec![1, 0, 2]);
}
