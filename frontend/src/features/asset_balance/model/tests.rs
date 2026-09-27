use super::*;
use rust_decimal_macros::dec;
use serde::Deserialize;
use serde_json::json;

const RATE_TOLERANCE: f64 = 1e-9;

#[derive(Deserialize)]
struct FixtureDocument {
    valuation_cases: Vec<ValuationCase>,
    summary_cases: Vec<SummaryCase>,
    composition_cases: Vec<CompositionCase>,
    kpi_cases: Vec<KpiCase>,
}

#[derive(Deserialize)]
struct ValuationCase {
    name: String,
    market_value: Value,
    purchase_amount: Value,
    expected: AmountRate,
}

#[derive(Deserialize)]
struct AmountRate {
    amount: Value,
    rate: Value,
}

#[derive(Deserialize)]
struct SummaryCase {
    name: String,
    items: Vec<SummaryItem>,
    expected: SummaryExpected,
}

#[derive(Deserialize)]
struct SummaryItem {
    market_value: Value,
    total_purchase_amount: Value,
}

#[derive(Deserialize)]
struct SummaryExpected {
    #[serde(rename = "marketValue")]
    market_value: Value,
    amount: Value,
    rate: Value,
    incomplete: bool,
}

#[derive(Deserialize)]
struct CompositionCase {
    name: String,
    values: Vec<f64>,
    expected_percentages: Vec<Value>,
}

#[derive(Deserialize)]
struct KpiCase {
    name: String,
    holdings: Vec<KpiHoldingFixture>,
    dividends_per_share: HashMap<String, Value>,
    expected: KpiExpected,
}

#[derive(Deserialize)]
struct KpiHoldingFixture {
    security_code: String,
    shares: Value,
    total_purchase_amount: Value,
}

#[derive(Deserialize)]
struct KpiExpected {
    total_purchase_amount: Value,
    total_annual_dividends: Value,
    dividend_yield: Value,
    holdings_count: usize,
}

fn fixture() -> FixtureDocument {
    serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/asset_balance/valuation.json"
    )))
    .expect("shared asset balance fixture parses")
}

fn expected_number(value: &Value) -> Option<f64> {
    match value {
        Value::Null => None,
        Value::Number(number) => number.as_f64(),
        unexpected => panic!("fixture expected must be a number or null: {unexpected}"),
    }
}

fn assert_optional_amount(actual: Option<f64>, expected: &Value, case: &str, field: &str) {
    assert_eq!(actual, expected_number(expected), "{case} {field}");
}

fn assert_optional_rate(actual: Option<f64>, expected: &Value, case: &str, field: &str) {
    match (actual, expected_number(expected)) {
        (None, None) => {}
        (Some(actual), Some(expected)) => {
            let diff = (actual - expected).abs();
            assert!(
                diff <= RATE_TOLERANCE,
                "{case} {field}: actual {actual} expected {expected} (diff {diff})"
            );
        }
        (actual, expected) => panic!("{case} {field}: actual {actual:?} expected {expected:?}"),
    }
}

fn json_number_or_zero(value: &Value) -> f64 {
    to_finite_amount(value).unwrap_or(0.0)
}

#[test]
fn shared_valuation_cases_match() {
    let fixture = fixture();
    assert_eq!(fixture.valuation_cases.len(), 9);
    for case in &fixture.valuation_cases {
        let result = calculate_valuation(&case.market_value, &case.purchase_amount);
        assert_optional_amount(result.amount, &case.expected.amount, &case.name, "amount");
        assert_optional_rate(result.rate, &case.expected.rate, &case.name, "rate");
    }
}

#[test]
fn shared_summary_cases_match() {
    let fixture = fixture();
    assert_eq!(fixture.summary_cases.len(), 8);
    for case in &fixture.summary_cases {
        let items: Vec<ValuationItem> = case
            .items
            .iter()
            .map(|item| ValuationItem {
                market_value: item.market_value.clone(),
                total_purchase_amount: item.total_purchase_amount.clone(),
            })
            .collect();
        let summary = summarize_valuation(&items);
        assert_eq!(
            summary.incomplete, case.expected.incomplete,
            "{} incomplete",
            case.name
        );
        assert_optional_amount(
            summary.market_value,
            &case.expected.market_value,
            &case.name,
            "marketValue",
        );
        assert_optional_amount(summary.amount, &case.expected.amount, &case.name, "amount");
        assert_optional_rate(summary.rate, &case.expected.rate, &case.name, "rate");
    }
}

#[test]
fn shared_composition_cases_match() {
    let fixture = fixture();
    assert_eq!(fixture.composition_cases.len(), 7);
    for case in &fixture.composition_cases {
        let actual = composition_percentages(&case.values);
        assert_eq!(
            actual.len(),
            case.expected_percentages.len(),
            "{}",
            case.name
        );
        for (index, (actual, expected)) in
            actual.iter().zip(&case.expected_percentages).enumerate()
        {
            assert_optional_rate(
                Some(*actual),
                expected,
                &case.name,
                &format!("percentages[{index}]"),
            );
        }
    }
}

#[test]
fn shared_kpi_cases_match() {
    let fixture = fixture();
    assert_eq!(fixture.kpi_cases.len(), 6);
    for case in &fixture.kpi_cases {
        let holdings: Vec<KpiHolding> = case
            .holdings
            .iter()
            .map(|item| KpiHolding {
                security_code: item.security_code.clone(),
                shares: json_number_or_zero(&item.shares),
                total_purchase_amount: json_number_or_zero(&item.total_purchase_amount),
            })
            .collect();
        let dividends: HashMap<String, f64> = case
            .dividends_per_share
            .iter()
            .map(|(code, value)| (code.clone(), value.as_f64().expect("dividend per share")))
            .collect();
        let kpi = calculate_portfolio_kpi(&holdings, &dividends, None);
        assert_optional_amount(
            Some(kpi.total_purchase_amount),
            &case.expected.total_purchase_amount,
            &case.name,
            "total_purchase_amount",
        );
        assert_optional_amount(
            kpi.total_annual_dividends,
            &case.expected.total_annual_dividends,
            &case.name,
            "total_annual_dividends",
        );
        assert_optional_rate(
            kpi.dividend_yield,
            &case.expected.dividend_yield,
            &case.name,
            "dividend_yield",
        );
        assert_eq!(
            kpi.holdings_count, case.expected.holdings_count,
            "{}",
            case.name
        );
        assert_eq!(kpi.holdings_count, case.holdings.len(), "{}", case.name);
    }
}

fn summary_items() -> Vec<ValuationItem> {
    vec![
        ValuationItem {
            market_value: json!(260000),
            total_purchase_amount: json!(250000),
        },
        ValuationItem {
            market_value: json!(650000),
            total_purchase_amount: json!(600000),
        },
    ]
}

fn kpi_holdings() -> Vec<KpiHolding> {
    vec![
        KpiHolding {
            security_code: "7203".to_string(),
            shares: 100.0,
            total_purchase_amount: 250000.0,
        },
        KpiHolding {
            security_code: "6758".to_string(),
            shares: 50.0,
            total_purchase_amount: 600000.0,
        },
    ]
}

fn full_dividends() -> HashMap<String, f64> {
    HashMap::from([("7203".to_string(), 50.0), ("6758".to_string(), 240.0)])
}

#[test]
fn summary_override_matches_component_cases() {
    let no_summary = summarize_valuation_with_summary(&summary_items(), None);
    assert_eq!(no_summary.market_value, Some(910000.0));
    assert_eq!(no_summary.amount, Some(60000.0));
    assert!(!no_summary.incomplete);

    let small = SummaryOverride {
        total_purchase_amount: dec!(1000),
        total_market_value: dec!(1100),
    };
    let prioritized = summarize_valuation_with_summary(&summary_items(), Some(&small));
    assert_eq!(prioritized.market_value, Some(1100.0));
    assert_eq!(prioritized.amount, Some(100.0));
    assert_optional_rate(prioritized.rate, &json!(10.0), "summary", "rate");
    assert!(!prioritized.incomplete);

    let large = SummaryOverride {
        total_purchase_amount: dec!(1234567),
        total_market_value: dec!(1300000),
    };
    let reviewed = summarize_valuation_with_summary(&summary_items(), Some(&large));
    assert_eq!(reviewed.market_value, Some(1300000.0));
    assert_eq!(reviewed.amount, Some(65433.0));
    assert_optional_rate(reviewed.rate, &json!(5.300076869056115), "summary", "rate");
    assert!(!reviewed.incomplete);
}

#[test]
fn summary_override_incomplete_detail_is_incomplete() {
    let incomplete_items = vec![ValuationItem {
        market_value: Value::Null,
        total_purchase_amount: json!(20),
    }];
    let large = SummaryOverride {
        total_purchase_amount: dec!(1234567),
        total_market_value: dec!(1300000),
    };
    let propagated = summarize_valuation_with_summary(&incomplete_items, Some(&large));
    assert!(propagated.incomplete);
    assert_eq!(propagated.market_value, None);
}

#[test]
fn summarize_valuation_totals_are_exact() {
    let items = vec![
        ValuationItem {
            market_value: json!(1100.10),
            total_purchase_amount: json!(1000.05),
        },
        ValuationItem {
            market_value: json!(2200.20),
            total_purchase_amount: json!(2000.15),
        },
    ];
    let summary = summarize_valuation(&items);
    assert_eq!(summary.market_value, Some(3300.30));
    assert_eq!(summary.amount, Some(300.10));
    assert!(!summary.incomplete);

    let items = vec![
        ValuationItem {
            market_value: json!(1000000.1),
            total_purchase_amount: json!(999999.95),
        },
        ValuationItem {
            market_value: json!(2000000.2),
            total_purchase_amount: json!(1999999.95),
        },
    ];
    let summary = summarize_valuation(&items);
    assert_eq!(summary.amount, Some(0.4));

    let totals = SummaryOverride::sum([
        (dec!(1100.10), dec!(1000.05)),
        (dec!(2200.20), dec!(2000.15)),
    ])
    .expect("totals fit in Decimal");
    assert_eq!(
        (totals.total_market_value, totals.total_purchase_amount),
        (dec!(3300.30), dec!(3000.20))
    );
    assert!(SummaryOverride::sum([(Decimal::MAX, dec!(0)), (Decimal::MAX, dec!(0))]).is_none());

    let summary = summarize_valuation(&[]);
    assert_eq!(summary.amount, Some(0.0));
    assert_eq!(summary.market_value, Some(0.0));
}

#[test]
fn summarize_valuation_overflow_falls_back_to_f64() {
    let items: Vec<ValuationItem> = (0..8)
        .map(|_| ValuationItem {
            market_value: json!(1e28),
            total_purchase_amount: json!(0.0),
        })
        .collect();
    let summary = summarize_valuation(&items);
    assert!(!summary.incomplete);
    assert_eq!(summary.market_value, Some(8e28));
    assert_eq!(summary.amount, Some(8e28));
    assert_eq!(summary.rate, None);

    let items = vec![ValuationItem {
        market_value: json!(1e30),
        total_purchase_amount: json!(1e29),
    }];
    let summary = summarize_valuation(&items);
    assert!(!summary.incomplete);
    assert_eq!(summary.market_value, Some(1e30));
    assert_eq!(summary.amount, Some(9e29));
    let rate = summary.rate.expect("nonzero purchase must produce a rate");
    assert!((rate - 900.0).abs() < 1e-6);
}

#[test]
fn calculate_valuation_rate_overflow_does_not_panic() {
    let result = calculate_valuation(&json!(1e28), &json!(1e-10));
    assert_eq!(result.amount, Some(1e28));
    assert!(result.rate.is_some_and(|rate| rate > 1e30));

    let result = calculate_valuation(&json!(1e30), &json!(1e29));
    assert_eq!(result.amount, Some(9e29));
    let rate = result.rate.expect("nonzero purchase must produce a rate");
    assert!((rate - 900.0).abs() < 1e-6);
}

#[test]
fn kpi_summary_override_matches_component_cases() {
    let without_summary = calculate_portfolio_kpi(&kpi_holdings(), &full_dividends(), None);
    assert_eq!(without_summary.total_purchase_amount, 850000.0);
    assert_eq!(without_summary.total_annual_dividends, Some(17000.0));
    assert_optional_rate(without_summary.dividend_yield, &json!(2.0), "kpi", "yield");

    let with_summary =
        calculate_portfolio_kpi(&kpi_holdings(), &full_dividends(), Some(1234567.0));
    assert_eq!(with_summary.total_purchase_amount, 1234567.0);
    assert_eq!(with_summary.total_annual_dividends, Some(17000.0));
    assert_optional_rate(
        with_summary.dividend_yield,
        &json!(1.3770010052107338),
        "kpi",
        "yield",
    );
    assert_eq!(with_summary.holdings_count, 2);
}

#[test]
fn to_fixed_matches_js_boundary_cases() {
    assert_eq!(to_fixed(0.125, 2), 0.13);
    assert_eq!(to_fixed(2.675, 2), 2.67);
    assert_eq!(to_fixed(1.005, 2), 1.0);
    assert_eq!(to_fixed(201.0 / 20000.0 * 100.0, 2), 1.0);
    assert_eq!(to_fixed(-0.125, 2), -0.13);
    assert_eq!(to_fixed(-1.005, 2), -1.0);
    assert_eq!(to_fixed(12.5, 0), 13.0);
    assert_eq!(to_fixed(-12.5, 0), -13.0);
    assert_eq!(to_fixed(0.1 + 0.2, 10), 0.3);
    // 非正規化数は仮数部と指数を別分岐で取り出す
    assert_eq!(to_fixed(f64::from_bits(1), 2), 0.0);
    assert!(to_fixed(-f64::from_bits(1), 2).is_sign_negative());
    // 仮数の最上位ビットが立つ値は暗黙の 1 ビット合成で値が変わる
    assert_eq!(to_fixed(0.75, 0), 1.0);
    assert_eq!(to_fixed(-0.75, 0), -1.0);
    let negative_zero = to_fixed(-0.001, 2);
    assert_eq!(negative_zero, 0.0);
    assert!(negative_zero.is_sign_negative());
    assert!(!to_fixed(-0.0, 2).is_sign_negative());
}

#[test]
fn intl_fixed_matches_number_format_boundary_cases() {
    // toFixed とは異なり、最短10進表記を10進の値として半分以上で切り上げる
    assert_eq!(intl_fixed(1.005, 2), 1.01);
    assert_eq!(intl_fixed(2.675, 2), 2.68);
    assert_eq!(intl_fixed(-1.005, 2), -1.01);
    assert_eq!(intl_fixed(-2.675, 2), -2.68);
    assert_eq!(intl_fixed(0.995, 2), 1.0);
    assert_eq!(intl_fixed(1.004, 2), 1.0);
    assert_eq!(intl_fixed(0.125, 2), 0.13);
    assert_eq!(intl_fixed(1.5, 2), 1.5);
    assert_eq!(intl_fixed(100.0, 2), 100.0);
    let negative_zero = intl_fixed(-0.001, 2);
    assert_eq!(negative_zero, 0.0);
    assert!(negative_zero.is_sign_negative());
    // Decimal に載らない巨大値は to_fixed にフォールバックする
    assert_eq!(intl_fixed(1e30, 2), 1e30);
    assert_eq!(intl_fixed(f64::INFINITY, 2), f64::INFINITY);
}

#[test]
fn safe_add_matches_react_rounding() {
    assert_eq!(safe_add(0.1, 0.2), 0.3);
    assert_eq!(safe_add(10.0, 20.0), 30.0);
    assert_eq!(safe_add(safe_add(0.0, 0.00000000004), 0.00000000004), 0.0);
    assert_eq!(safe_add(250000.0, 600000.0), 850000.0);
}

#[test]
fn huge_values_do_not_panic() {
    assert_eq!(to_finite_amount(&json!(1e30)), Some(1e30));
    let precision =
        calculate_valuation(&json!(9007199254740993u64), &json!(9007199254740992u64));
    assert_eq!(precision.amount, Some(0.0));
    let overflow = safe_add(1e308, 1e308);
    assert!(overflow.is_infinite());
    assert_eq!(to_fixed(f64::INFINITY, 2), f64::INFINITY);
    assert!(calculate_composition_percentage(1e308, 1e308).is_finite());
}

#[test]
fn chart_filter_matches_component_conditions() {
    assert!(should_include_chart_item(Some(1.0), None));
    assert!(should_include_chart_item(Some(250000.0), Some(260000.0)));
    assert!(!should_include_chart_item(Some(0.0), Some(0.0)));
    assert!(!should_include_chart_item(Some(0.0), None));
    assert!(!should_include_chart_item(None, None));
    assert!(should_include_chart_item(Some(0.0), Some(100000.0)));
    assert!(should_include_chart_item(None, Some(50000.0)));
}

#[test]
fn chart_percentages_match_component_conditions() {
    assert!(chart_percentages(&[], &[]).is_empty());
    assert!(chart_percentages(&[0.0, 0.0], &[Some(0.0), Some(0.0)]).is_empty());
    assert_eq!(chart_percentages(&[0.0], &[Some(50000.0)]), vec![None]);
    let percentages = chart_percentages(&[250000.0, 600000.0], &[Some(1.0), Some(1.0)]);
    assert_eq!(percentages.len(), 2);
    let total = 850000.0;
    assert_eq!(
        percentages,
        vec![
            Some(250000.0 / total * 100.0),
            Some(600000.0 / total * 100.0),
        ]
    );
}

#[test]
fn missing_markers_match_react_cases() {
    for marker in [
        "",
        "-",
        "—",
        "ー",
        "--",
        "n/a",
        "N/A",
        "null",
        "undefined",
        "  -  ",
    ] {
        assert_eq!(
            to_finite_amount(&Value::String(marker.to_string())),
            None,
            "{marker}"
        );
    }
    assert_eq!(
        to_finite_amount(&Value::String("1,180,000".to_string())),
        Some(1180000.0)
    );
    assert_eq!(to_finite_amount(&Value::Null), None);
}

#[test]
fn security_normalization_matches_react_cases() {
    assert_eq!(normalize_security_code(" 7974: 任天堂 "), "7974");
    assert_eq!(normalize_security_code("7203: トヨタ自動車"), "7203");
    assert_eq!(normalize_security_code("brk.b"), "BRK.B");
    assert_eq!(normalize_security_code("  "), "");
    assert_eq!(normalize_security_code(""), "");
    assert_eq!(normalize_display_name("ＫＤＤＩ"), "KDDI");
    assert_eq!(normalize_display_name("トヨタ自動車"), "トヨタ自動車");
}

proptest::proptest! {
    #[test]
    fn prop_group_thousands_positions(digits in "[0-9]{1,15}") {
        let grouped = group_thousands(&digits);
        proptest::prop_assert_eq!(grouped.replace(',', ""), digits);
        for (position, character) in grouped.chars().enumerate() {
            if character == ',' {
                proptest::prop_assert_eq!((grouped.len() - position) % 4, 0);
            }
        }
    }

    #[test]
    fn prop_to_fixed_symmetric_and_quantized(
        mantissa in -9_999_999_999_999i64..9_999_999_999_999i64,
        divisor in 1u32..=6u32,
        decimals in 0u32..=5u32,
    ) {
        let value = mantissa as f64 / 10f64.powi(divisor as i32);
        let result = to_fixed(value, decimals);
        proptest::prop_assert_eq!(to_fixed(-value, decimals), -result);
        let unit = 10f64.powi(-(decimals as i32));
        proptest::prop_assert!((result - value).abs() <= 0.51 * unit);
        let scaled = result / unit;
        proptest::prop_assert!(
            (scaled - scaled.round()).abs() <= 1e-9 * scaled.abs() + 1e-9
        );
    }

    #[test]
    fn prop_intl_fixed_symmetric_and_quantized(
        mantissa in -9_999_999_999_999i64..9_999_999_999_999i64,
        divisor in 1u32..=6u32,
        decimals in 0u32..=5u32,
    ) {
        let value = mantissa as f64 / 10f64.powi(divisor as i32);
        let result = intl_fixed(value, decimals);
        proptest::prop_assert_eq!(intl_fixed(-value, decimals), -result);
        let unit = 10f64.powi(-(decimals as i32));
        proptest::prop_assert!((result - value).abs() <= 0.6 * unit);
    }

    #[test]
    fn prop_composition_percentages_sum_to_100(
        values in proptest::collection::vec(0.0001f64..1_000_000.0f64, 1..8usize),
    ) {
        let percentages = composition_percentages(&values);
        proptest::prop_assert_eq!(percentages.len(), values.len());
        let sum: f64 = percentages.iter().sum();
        proptest::prop_assert!(
            (sum - 100.0).abs() <= 0.006 * percentages.len() as f64 + 1e-9,
            "sum={sum}"
        );
    }

    #[test]
    fn prop_to_finite_amount(
        value in -1e15f64..1e15f64,
        marker in proptest::sample::select(vec![
            "", "-", "—", "ー", "--", "n/a", "N/A", "null", "undefined",
        ]),
    ) {
        proptest::prop_assert_eq!(to_finite_amount(&json!(value)), Some(value));
        let text = value.to_string();
        proptest::prop_assert_eq!(
            to_finite_amount(&json!(text)),
            Some(text.parse::<f64>().unwrap())
        );
        proptest::prop_assert_eq!(to_finite_amount(&json!(marker)), None);
        proptest::prop_assert_eq!(to_finite_amount(&json!(true)), None);
        proptest::prop_assert_eq!(to_finite_amount(&json!([1, 2])), None);
        proptest::prop_assert_eq!(to_finite_amount(&Value::Null), None);
        // "1e999" / "inf" / "NaN" は f64 にはパースされるが非有限なので欠損扱い
        for text in ["1e999", "-1e999", "inf", "-inf", "NaN"] {
            proptest::prop_assert_eq!(to_finite_amount(&json!(text)), None, "text={}", text);
        }
    }

    #[test]
    fn prop_calculate_valuation(
        market in proptest::option::of(-1e15f64..1e15f64),
        purchase in proptest::option::of(-1e15f64..1e15f64),
    ) {
        let market_json = market.map_or_else(|| json!("-"), |v| json!(v));
        let purchase_json = purchase.map_or_else(|| json!("-"), |v| json!(v));
        let result = calculate_valuation(&market_json, &purchase_json);
        match (market, purchase) {
            (Some(m), Some(p)) => {
                let amount = result
                    .amount
                    .expect("finite inputs must produce an amount");
                let diff = m - p;
                proptest::prop_assert_eq!(amount.signum(), diff.signum());
                proptest::prop_assert!(
                    (amount - diff).abs() <= 1e-9 * m.abs().max(p.abs()).max(1.0)
                );
                if p == 0.0 {
                    proptest::prop_assert_eq!(result.rate, None);
                } else {
                    let rate = result
                        .rate
                        .expect("nonzero purchase must produce a rate");
                    proptest::prop_assert_eq!(
                        rate.signum(),
                        amount.signum() * p.signum()
                    );
                    if rate.is_finite() {
                        let reconstructed = rate * p / 100.0;
                        proptest::prop_assert!(
                            (reconstructed - amount).abs()
                                <= 1e-6 * amount.abs().max(1.0),
                            "rate={rate} p={p} amount={amount}"
                        );
                    }
                }
            }
            _ => {
                proptest::prop_assert_eq!(result.amount, None);
                proptest::prop_assert_eq!(result.rate, None);
            }
        }
    }

    #[test]
    fn prop_should_include_chart_item(
        purchase in proptest::option::of(-1e6f64..1e6f64),
        market in proptest::option::of(-1e6f64..1e6f64),
    ) {
        let expected = matches!(purchase, Some(p) if p > 0.0)
            || matches!(market, Some(m) if m != 0.0);
        proptest::prop_assert_eq!(should_include_chart_item(purchase, market), expected);
    }

    #[test]
    fn prop_chart_percentages(
        items in proptest::collection::vec(
            (0.0f64..1e6f64, proptest::option::of(0.0f64..1e6f64)),
            0..8usize
        ),
    ) {
        let values: Vec<f64> = items.iter().map(|item| item.0).collect();
        let markets: Vec<Option<f64>> = items.iter().map(|item| item.1).collect();
        let result = chart_percentages(&values, &markets);
        let total: f64 = values.iter().sum();
        if total == 0.0 {
            let has_valuation = markets
                .iter()
                .any(|market| matches!(market, Some(v) if *v != 0.0));
            if has_valuation {
                proptest::prop_assert_eq!(result, vec![None; values.len()]);
            } else {
                proptest::prop_assert!(result.is_empty());
            }
        } else {
            let expected: Vec<Option<f64>> =
                values.iter().map(|v| Some(*v / total * 100.0)).collect();
            proptest::prop_assert_eq!(result, expected);
        }
    }

    #[test]
    fn prop_normalize_security_code(input in ".*") {
        let output = normalize_security_code(&input);
        let expected: String = input
            .trim()
            .split([':', '：'])
            .next()
            .unwrap_or_default()
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>()
            .to_uppercase();
        proptest::prop_assert_eq!(output.clone(), expected);
        proptest::prop_assert!(!output.chars().any(|c| c.is_whitespace()));
        proptest::prop_assert_eq!(output.clone(), output.to_uppercase());
    }

    #[test]
    fn prop_kpi_empty_dividend_map(
        holdings in proptest::collection::vec(
            ("[0-9]{4}", 0.0f64..1e5f64, 0.0f64..1e8f64),
            0..8usize
        ),
    ) {
        let holdings: Vec<KpiHolding> = holdings
            .into_iter()
            .map(|(security_code, shares, total_purchase_amount)| KpiHolding {
                security_code,
                shares,
                total_purchase_amount,
            })
            .collect();
        let kpi = calculate_portfolio_kpi(&holdings, &HashMap::new(), None);
        proptest::prop_assert_eq!(kpi.holdings_count, holdings.len());
        proptest::prop_assert_eq!(kpi.total_annual_dividends, None);
        proptest::prop_assert_eq!(kpi.dividend_yield, None);
        let expected_purchase = holdings
            .iter()
            .fold(0.0, |sum, item| safe_add(sum, item.total_purchase_amount));
        proptest::prop_assert_eq!(kpi.total_purchase_amount, expected_purchase);
    }
}
