use super::{ChartPlan, SortInput, SortKey, SortOrder, chart_display, chart_plan, sort_holdings};
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

#[derive(Debug, Deserialize)]
struct ExpectedOthers {
    count: usize,
    percentage: f64,
}

#[derive(Deserialize)]
struct DisplayExpectation {
    show_all: bool,
    visible_ids: Vec<String>,
    others: Option<ExpectedOthers>,
    toggle_label: Option<String>,
}

#[derive(Deserialize)]
struct DisplayCase {
    name: String,
    items: Vec<FixtureItem>,
    expectations: Vec<DisplayExpectation>,
}

#[derive(Deserialize)]
struct Fixture {
    order_cases: Vec<OrderCase>,
    display_cases: Vec<DisplayCase>,
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

#[test]
fn shared_display_cases_match() {
    let fixture = fixture();
    assert_eq!(fixture.display_cases.len(), 8);
    for case in &fixture.display_cases {
        let plan = plan_for(&case.items);
        for expectation in &case.expectations {
            let label = format!("{} show_all={}", case.name, expectation.show_all);
            let display = chart_display(plan.order.len(), &plan.percentages, expectation.show_all);
            let visible: Vec<&str> = plan
                .order
                .iter()
                .zip(&plan.percentages)
                .enumerate()
                .filter(|(position, (_, percentage))| {
                    *position < display.visible_count || percentage.is_none()
                })
                .map(|(_, (&index, _))| case.items[index].id.as_str())
                .collect();
            assert_eq!(visible, expectation.visible_ids, "{label} visible_ids");
            match (&display.others, &expectation.others) {
                (None, None) => {}
                (Some(actual), Some(expected)) => {
                    assert_eq!(actual.count, expected.count, "{label} others.count");
                    let diff = (actual.percentage - expected.percentage).abs();
                    assert!(
                        diff <= PERCENTAGE_TOLERANCE,
                        "{label} others.percentage: actual {} expected {}",
                        actual.percentage,
                        expected.percentage
                    );
                }
                (actual, expected) => {
                    panic!("{label} others: actual {actual:?} expected {expected:?}")
                }
            }
            assert_eq!(
                display.toggle_label.is_some(),
                expectation.toggle_label.is_some(),
                "{label} toggle presence"
            );
        }
    }
}

#[test]
fn collapsed_others_appear_only_when_their_sum_is_positive() {
    let mut percentages: Vec<Option<f64>> = vec![Some(50.0); 20];
    percentages.push(Some(0.0));
    assert!(chart_display(21, &percentages, false).others.is_none());
    percentages[20] = None;
    assert!(chart_display(21, &percentages, false).others.is_none());
    percentages[20] = Some(1.5);
    let others = chart_display(21, &percentages, false).others.unwrap();
    assert_eq!(others.count, 1);
    assert_eq!(others.percentage, 1.5);
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
        sort_input("7203", 100.0, 80.0, Some(10.0)),
        sort_input("6758", 200.0, 150.0, Some(20.0)),
        sort_input("9999", 50.0, 40.0, Some(5.0)),
    ];
    let order = sort_holdings(inputs, SortKey::MarketValue, SortOrder::Desc);
    assert_eq!(order, vec![1, 0, 2]);
}

#[test]
fn sort_holdings_orders_by_profit_loss_asc() {
    let inputs = vec![
        sort_input("7203", 100.0, 80.0, Some(10.0)),
        sort_input("6758", 200.0, 150.0, Some(20.0)),
        sort_input("9999", 50.0, 40.0, Some(5.0)),
    ];
    let order = sort_holdings(inputs, SortKey::ProfitLoss, SortOrder::Asc);
    assert_eq!(order, vec![2, 0, 1]);
}

#[test]
fn sort_holdings_orders_by_dividend_desc() {
    let inputs = vec![
        sort_input("7203", 100.0, 80.0, Some(10.0)),
        sort_input("6758", 200.0, 150.0, Some(20.0)),
        sort_input("9999", 50.0, 40.0, Some(5.0)),
    ];
    let order = sort_holdings(inputs, SortKey::Dividend, SortOrder::Desc);
    assert_eq!(order, vec![1, 0, 2]);
}

#[test]
fn sort_holdings_orders_by_yield_asc() {
    let inputs = vec![
        sort_input("7203", 100.0, 80.0, Some(10.0)),
        sort_input("6758", 200.0, 150.0, Some(20.0)),
        sort_input("9999", 50.0, 40.0, Some(5.0)),
    ];
    let order = sort_holdings(inputs, SortKey::Yield, SortOrder::Asc);
    assert_eq!(order, vec![2, 0, 1]);
}

#[test]
fn sort_holdings_places_uncomputable_at_end_regardless_of_order() {
    let inputs = vec![
        sort_input("7203", 100.0, 80.0, Some(10.0)),
        sort_input("6758", 0.0, 150.0, Some(20.0)),
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
        sort_input("7203", 100.0, 80.0, Some(10.0)),
        sort_input("6758", 100.0, 80.0, Some(10.0)),
        sort_input("9999", 100.0, 80.0, Some(10.0)),
    ];
    let order = sort_holdings(inputs, SortKey::MarketValue, SortOrder::Desc);
    assert_eq!(order, vec![1, 0, 2]);
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
        sort_input("7203", 200.0, 150.0, Some(20.0)),
        sort_input("9999", 50.0, 40.0, Some(5.0)),
    ];
    let order = sort_holdings(inputs, SortKey::MarketValue, SortOrder::Desc);
    assert_eq!(order, vec![1, 2, 0]);
}
