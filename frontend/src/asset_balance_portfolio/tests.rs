use super::{chart_display, chart_plan, ChartPlan};
use serde::Deserialize;

const PERCENTAGE_TOLERANCE: f64 = 1e-9;

#[derive(Deserialize)]
struct FixtureItem {
    id: String,
    purchase: f64,
    market: Option<f64>,
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
    grid_class: String,
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
    serde_json::from_str(include_str!(
        "../../tests/fixtures/asset_balance/portfolio.json"
    ))
    .expect("shared asset balance portfolio fixture parses")
}

fn plan_for(items: &[FixtureItem]) -> ChartPlan {
    let values: Vec<f64> = items.iter().map(|item| item.purchase).collect();
    let markets: Vec<Option<f64>> = items.iter().map(|item| item.market).collect();
    chart_plan(&values, &markets)
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
    assert_eq!(fixture.order_cases.len(), 6);
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
    assert_eq!(fixture.display_cases.len(), 7);
    for case in &fixture.display_cases {
        let plan = plan_for(&case.items);
        for expectation in &case.expectations {
            let label = format!("{} show_all={}", case.name, expectation.show_all);
            let display = chart_display(plan.order.len(), &plan.percentages, expectation.show_all);
            let visible: Vec<&str> = plan
                .order
                .iter()
                .take(display.visible_count)
                .map(|&index| case.items[index].id.as_str())
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
                display.toggle_label, expectation.toggle_label,
                "{label} toggle_label"
            );
            assert_eq!(
                display.grid_class, expectation.grid_class,
                "{label} grid_class"
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
