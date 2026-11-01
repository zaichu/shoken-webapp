use super::{
    ChartPlan, CompositionBasis, CompositionGroup, SortInput, SortKey, SortOrder, chart_plan,
    composition_color, composition_entries, composition_label_fits, composition_percentages,
    composition_rows, composition_slice_arc, sort_holdings,
};
use crate::features::asset_balance::holdings::{HoldingDividend, HoldingView};
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
fn composition_group_and_basis_labels_are_distinct() {
    let labels: Vec<&str> = CompositionGroup::ALL
        .iter()
        .map(|group| group.label())
        .collect();
    for (index, label) in labels.iter().enumerate() {
        assert!(!label.is_empty());
        assert!(
            !labels[..index].contains(label),
            "区分のラベルが重複している: {label}"
        );
    }
    assert_eq!(CompositionBasis::MarketValue.value(), "market_value");
    assert_eq!(CompositionBasis::Dividend.value(), "dividend");
    assert_ne!(
        CompositionBasis::MarketValue.label(),
        CompositionBasis::Dividend.label()
    );
}

fn composition_view(
    code: &str,
    name: &str,
    account: Option<&str>,
    id: Option<&str>,
) -> HoldingView {
    HoldingView {
        id: id.map(str::to_string),
        code: code.to_string(),
        name: name.to_string(),
        account: account.map(str::to_string),
        shares: 100.0,
        average_price: 100.0,
        purchase: 100.0,
        current_price: 100.0,
        day_change: None,
        price_as_of: None,
    }
}

#[test]
fn composition_entries_keep_rows_for_holding_and_aggregate_for_product_and_account() {
    let views = vec![
        composition_view("9904", "架空自動車", Some("特定"), Some("1")),
        composition_view("9904", "架空自動車", Some("NISA"), Some("2")),
        composition_view("9914", "架空電機", Some("特定"), Some("3")),
        composition_view("9920", "口座なし銘柄", None, Some("4")),
    ];
    let basis = vec![Some(300.0), Some(100.0), Some(200.0), Some(50.0)];

    let holding = composition_entries(&views, CompositionGroup::Holding, &basis);
    assert_eq!(holding.len(), 4);
    assert_eq!(holding[0].name, "架空自動車");
    assert_eq!(holding[0].sub, "9904");
    assert_eq!(holding[0].value, Some(300.0));
    assert_eq!(holding[0].holdings, 1);
    assert_eq!(holding[0].href.as_deref(), Some("/assetbalance/holdings/1"));

    let product = composition_entries(&views, CompositionGroup::Product, &basis);
    assert_eq!(product.len(), 3);
    assert_eq!(product[0].name, "架空自動車");
    assert_eq!(product[0].value, Some(400.0));
    assert_eq!(product[0].holdings, 2);
    assert_eq!(product[0].href, None);
    assert_eq!(product[1].name, "架空電機");

    let account = composition_entries(&views, CompositionGroup::Account, &basis);
    assert_eq!(account.len(), 3);
    assert_eq!(account[0].name, "特定");
    assert_eq!(account[0].value, Some(500.0));
    assert_eq!(account[0].holdings, 2);
    assert!(account.iter().any(|entry| entry.name == "—"));
}

#[test]
fn composition_entries_put_uncomputable_last_and_sum_only_available_values() {
    let views = vec![
        composition_view("9904", "算出不可", Some("特定"), Some("1")),
        composition_view("9914", "一部なし", Some("特定"), Some("2")),
        composition_view("9914", "一部なし", Some("NISA"), Some("3")),
    ];
    let basis = vec![None, Some(200.0), None];
    let entries = composition_entries(&views, CompositionGroup::Product, &basis);
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].name, "一部なし");
    assert_eq!(entries[0].value, Some(200.0));
    assert_eq!(entries[1].value, None);
}

#[test]
fn composition_percentages_follow_the_existing_zero_and_total_rules() {
    let (total, percentages) = composition_percentages(&[Some(150.0), Some(50.0), Some(0.0), None]);
    assert_eq!(total, 200.0);
    assert_eq!(percentages.len(), 4);
    assert!(percentages[0].is_some_and(|value| (value - 75.0).abs() < 1e-9));
    assert!(percentages[1].is_some_and(|value| (value - 25.0).abs() < 1e-9));
    assert_eq!(percentages[2], None);
    assert_eq!(percentages[3], None);

    let (total, percentages) = composition_percentages(&[Some(0.0), None]);
    assert_eq!(total, 0.0);
    assert_eq!(percentages, vec![None, None]);
}

#[test]
fn composition_rows_split_major_and_other_at_nine_percent() {
    let views = vec![
        composition_view("9904", "A", Some("特定"), Some("1")),
        composition_view("9914", "B", Some("特定"), Some("2")),
        composition_view("9920", "C", Some("特定"), Some("3")),
        composition_view("9930", "D", Some("特定"), Some("4")),
        composition_view("9940", "E", Some("特定"), Some("5")),
    ];
    let basis = vec![Some(50.0), Some(30.0), Some(9.0), Some(8.9), Some(2.1)];
    let entries = composition_entries(&views, CompositionGroup::Holding, &basis);
    let (_, percentages) =
        composition_percentages(&entries.iter().map(|entry| entry.value).collect::<Vec<_>>());
    let rows = composition_rows(&entries, &percentages);
    assert_eq!(rows.len(), 4);
    assert!(!rows[0].is_other);
    assert_eq!(rows[0].name, "A");
    assert!(rows[0].computable);
    assert_eq!(rows[2].name, "C");
    assert_eq!(rows[2].color, composition_color(2));
    let other = &rows[3];
    assert!(other.is_other);
    assert_eq!(other.name, "その他 2銘柄");
    assert!((other.percentage - 11.0).abs() < 1e-9);
    assert!(other.computable);
    assert_eq!(other.href, None);
}

#[test]
fn composition_rows_keep_uncomputable_minors_in_other() {
    let views = vec![
        composition_view("9904", "A", Some("特定"), Some("1")),
        composition_view("9914", "B", Some("特定"), Some("2")),
    ];
    let entries = composition_entries(&views, CompositionGroup::Holding, &[Some(100.0), None]);
    let (_, percentages) =
        composition_percentages(&entries.iter().map(|entry| entry.value).collect::<Vec<_>>());
    let rows = composition_rows(&entries, &percentages);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].name, "A");
    let other = &rows[1];
    assert!(other.is_other);
    assert_eq!(other.name, "その他 1銘柄");
    assert!(!other.computable);
    assert_eq!(other.value, None);
}

#[test]
fn composition_label_fits_only_when_text_stays_inside_the_slice() {
    assert!(composition_label_fits("架空自動車 100.0%", 100.0));
    assert!(composition_label_fits("架空グローバル先端素… 58.0%", 58.0));
    assert!(!composition_label_fits("その他 1銘柄 4.8%", 4.8));
    assert!(composition_label_fits("A 9.5%", 9.5));
    assert!(!composition_label_fits("架空エレクトロニクス 9.5%", 9.5));
}

#[test]
fn composition_slice_arc_reverses_below_the_middle() {
    let (_, top) = composition_slice_arc(80.0, 10.0);
    assert!(!top, "上半分は時計回りに読む");
    let (_, bottom) = composition_slice_arc(20.0, 20.0);
    assert!(bottom, "真下は逆向きに読む");
    let (path, full_reversed) = composition_slice_arc(0.0, 100.0);
    assert!(full_reversed, "1周は真下を中心にするため逆向きに読む");
    assert_eq!(
        path.matches("A 80 80").count(),
        2,
        "1周しても弧が欠けない: {path}"
    );
}

#[test]
fn composition_color_keeps_neighbors_different() {
    for position in 0..20 {
        assert_ne!(
            composition_color(position),
            composition_color(position + 1),
            "{position}"
        );
        assert!(composition_color(position) < 5, "{position}");
    }
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
