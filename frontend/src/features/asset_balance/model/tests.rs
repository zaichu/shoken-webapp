use super::*;
use serde::Deserialize;
use serde_json::{json, Value};

const RATE_TOLERANCE: f64 = 1e-9;

/// 欠損値として扱う文字列の集合（比較は小文字化後）。
const MISSING_MARKERS: &[&str] = &["", "-", "—", "ー", "--", "n/a", "null", "undefined"];

/// 任意の値を有限数に正規化する。欠損は `None` を返す。
/// 数値文字列（カンマ区切り可）は数値として扱う。
fn to_finite_amount(value: &Value) -> Option<f64> {
    match value {
        Value::Null => None,
        Value::Number(number) => number.as_f64(),
        Value::String(raw) => {
            let trimmed = raw.replace(',', "").trim().to_string();
            if trimmed.is_empty() {
                return None;
            }
            if MISSING_MARKERS.contains(&trimmed.to_lowercase().as_str()) {
                return None;
            }
            match trimmed.parse::<f64>() {
                Ok(parsed) if parsed.is_finite() => Some(parsed),
                _ => None,
            }
        }
        _ => None,
    }
}

fn f64_to_decimal_exact(value: f64) -> Option<Decimal> {
    Decimal::from_str_exact(&value.to_string()).ok()
}

fn calculate_valuation(market_value: Option<f64>, purchase_amount: Option<f64>) -> ValuationResult {
    let (Some(market), Some(purchase)) = (market_value, purchase_amount) else {
        return ValuationResult {
            amount: None,
            rate: None,
        };
    };
    let (amount, rate) = valuation_parts(
        f64_to_decimal_exact(market),
        f64_to_decimal_exact(purchase),
        market,
        purchase,
    );
    ValuationResult {
        amount: Some(amount),
        rate,
    }
}

/// 構成比（%）。`formatters.ts` の `calculatePercentage(value, total, 2)` に対応する。
/// 共通 fixture の契約用。画面表示の構成比は PieChart 準拠の `chart_percentages` を使う。
fn calculate_composition_percentage(value: f64, total: f64) -> f64 {
    if total == 0.0 {
        0.0
    } else {
        to_fixed(value / total * 100.0, 2)
    }
}

/// 構成比の一覧。合計を分母に各要素の割合を求める。
/// 分母の合計は素朴な加算で求める。
/// 共通 fixture の契約用。画面表示の構成比は PieChart 準拠の `chart_percentages` を使う。
fn composition_percentages(values: &[f64]) -> Vec<f64> {
    let total: f64 = values.iter().sum();
    values
        .iter()
        .map(|value| calculate_composition_percentage(*value, total))
        .collect()
}

#[derive(Deserialize)]
struct FixtureDocument {
    valuation_cases: Vec<ValuationCase>,
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
        let result = calculate_valuation(
            to_finite_amount(&case.market_value),
            to_finite_amount(&case.purchase_amount),
        );
        assert_optional_amount(result.amount, &case.expected.amount, &case.name, "amount");
        assert_optional_rate(result.rate, &case.expected.rate, &case.name, "rate");
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
        for (index, (actual, expected)) in actual.iter().zip(&case.expected_percentages).enumerate()
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
fn calculate_valuation_rate_overflow_does_not_panic() {
    let result = calculate_valuation(Some(1e28), Some(1e-10));
    assert_eq!(result.amount, Some(1e28));
    assert!(result.rate.is_some_and(|rate| rate > 1e30));

    let result = calculate_valuation(Some(1e30), Some(1e29));
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

    let with_summary = calculate_portfolio_kpi(&kpi_holdings(), &full_dividends(), Some(1234567.0));
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
fn safe_add_rounds_floating_point_noise() {
    assert_eq!(safe_add(0.1, 0.2), 0.3);
    assert_eq!(safe_add(10.0, 20.0), 30.0);
    assert_eq!(safe_add(safe_add(0.0, 0.00000000004), 0.00000000004), 0.0);
    assert_eq!(safe_add(250000.0, 600000.0), 850000.0);
}

#[test]
fn huge_values_do_not_panic() {
    assert_eq!(to_finite_amount(&json!(1e30)), Some(1e30));
    let precision = calculate_valuation(Some(9007199254740993.0), Some(9007199254740992.0));
    assert_eq!(precision.amount, Some(0.0));
    let overflow = safe_add(1e308, 1e308);
    assert!(overflow.is_infinite());
    assert_eq!(to_fixed(f64::INFINITY, 2), f64::INFINITY);
    assert!(calculate_composition_percentage(1e308, 1e308).is_finite());
}

#[test]
fn chart_percentages_match_component_conditions() {
    assert!(chart_percentages(&[]).is_empty());
    assert!(chart_percentages(&[0.0, 0.0]).is_empty());
    let percentages = chart_percentages(&[250000.0, 600000.0]);
    assert_eq!(percentages.len(), 2);
    let total = 850000.0;
    assert_eq!(
        percentages,
        vec![
            Some(250000.0 / total * 100.0),
            Some(600000.0 / total * 100.0),
        ]
    );
    // 取得額にマイナスが混ざっても帯の幅が負や 100% 超にならないよう絶対値で割る
    let mixed = chart_percentages(&[-100000.0, 300000.0]);
    assert_eq!(mixed, vec![Some(25.0), Some(75.0)]);
}

#[test]
fn missing_markers_are_not_parsed_as_amounts() {
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
fn security_normalization_covers_code_and_name_cases() {
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
        let result = calculate_valuation(market, purchase);
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
    fn prop_chart_percentages(
        values in proptest::collection::vec(-1e6f64..1e6f64, 0..8usize),
    ) {
        let result = chart_percentages(&values);
        let total: f64 = values.iter().map(|v| v.abs()).sum();
        if total == 0.0 {
            proptest::prop_assert!(result.is_empty());
        } else {
            let in_range = result
                .iter()
                .all(|p| matches!(p, Some(v) if (0.0..=100.0).contains(v)));
            let expected: Vec<Option<f64>> =
                values.iter().map(|v| Some(v.abs() / total * 100.0)).collect();
            proptest::prop_assert_eq!(result, expected);
            proptest::prop_assert!(in_range);
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
