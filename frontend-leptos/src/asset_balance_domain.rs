//! 資産管理の一覧・評価・構成比・KPI の純粋ロジック。
//!
//! React 側の正本との対応:
//! - [`to_finite_amount`], [`calculate_valuation`], [`summarize_valuation`]
//!   は `frontend/src/features/assetBalance/valuation.ts` の同名関数に対応する。
//! - [`calculate_composition_percentage`], [`composition_percentages`]
//!   は `frontend/src/lib/utils/formatters.ts` の `calculatePercentage`
//!   （既定 `decimals = 2`）に対応する。fixture の構成比ケースはこの関数で作られている。
//! - [`should_include_chart_item`], [`chart_percentages`] は
//!   `AssetPortfolioSummary.tsx` のチャート除外条件と
//!   `PortfolioPieChart.tsx` の未丸めパーセンテージに対応する（fixture 対象外）。
//! - [`calculate_portfolio_kpi`] は `AssetPortfolioSummary.tsx` の
//!   合計取得総額・年間配当・配当利回り・銘柄数の計算に対応する。

use rust_decimal::Decimal;
use rust_decimal::RoundingStrategy;
use serde_json::Value;
use std::collections::HashMap;
use std::str::FromStr;

/// `valuation.ts` の `MISSING_MARKERS` と同じ集合（比較は小文字化後）。
const MISSING_MARKERS: &[&str] = &["", "-", "—", "ー", "--", "n/a", "null", "undefined"];

/// 任意の値を有限数に正規化する。欠損は `None` を返す。
/// 数値文字列（カンマ区切り可）は数値として扱う。
/// `valuation.ts` の `toFiniteAmount` に対応する。
pub fn to_finite_amount(value: &Value) -> Option<Decimal> {
    match value {
        Value::Null => None,
        Value::Number(number) => Decimal::from_str(&number.to_string()).ok(),
        Value::String(raw) => {
            let trimmed = raw.replace(',', "").trim().to_string();
            if trimmed.is_empty() {
                return None;
            }
            let lowered = trimmed.to_lowercase();
            if MISSING_MARKERS.contains(&lowered.as_str()) {
                return None;
            }
            if let Ok(exact) = Decimal::from_str(&trimmed) {
                return Some(exact);
            }
            if let Ok(float) = trimmed.parse::<f64>() {
                if float.is_finite() {
                    return Decimal::from_f64_retain(float);
                }
            }
            None
        }
        _ => None,
    }
}

/// 1銘柄の評価損益。`valuation.ts` の `calculateValuation` に対応する。
#[derive(Clone, Debug, PartialEq)]
pub struct ValuationResult {
    pub amount: Option<Decimal>,
    pub rate: Option<Decimal>,
}

pub fn calculate_valuation(market_value: &Value, purchase_amount: &Value) -> ValuationResult {
    let market = to_finite_amount(market_value);
    let purchase = to_finite_amount(purchase_amount);
    match (market, purchase) {
        (Some(market), Some(purchase)) => {
            let amount = market - purchase;
            if purchase.is_zero() {
                ValuationResult {
                    amount: Some(amount),
                    rate: None,
                }
            } else {
                ValuationResult {
                    amount: Some(amount),
                    rate: Some(amount / purchase * Decimal::ONE_HUNDRED),
                }
            }
        }
        _ => ValuationResult {
            amount: None,
            rate: None,
        },
    }
}

/// `summarizeValuation` への入力1件。
#[derive(Clone, Debug)]
pub struct ValuationItem {
    pub market_value: Value,
    pub total_purchase_amount: Value,
}

/// 複数銘柄の合計。`valuation.ts` の `summarizeValuation` に対応する。
/// 率は合計金額から計算し、単純平均しない。
#[derive(Clone, Debug, PartialEq)]
pub struct ValuationSummary {
    pub market_value: Option<Decimal>,
    pub amount: Option<Decimal>,
    pub rate: Option<Decimal>,
    pub incomplete: bool,
}

pub fn summarize_valuation(items: &[ValuationItem]) -> ValuationSummary {
    let mut total_market = Decimal::ZERO;
    let mut total_purchase = Decimal::ZERO;
    for item in items {
        let market = to_finite_amount(&item.market_value);
        let purchase = to_finite_amount(&item.total_purchase_amount);
        match (market, purchase) {
            (Some(market), Some(purchase)) => {
                total_market += market;
                total_purchase += purchase;
            }
            _ => {
                return ValuationSummary {
                    market_value: None,
                    amount: None,
                    rate: None,
                    incomplete: true,
                };
            }
        }
    }
    let amount = total_market - total_purchase;
    if total_purchase.is_zero() {
        ValuationSummary {
            market_value: Some(total_market),
            amount: Some(amount),
            rate: None,
            incomplete: false,
        }
    } else {
        ValuationSummary {
            market_value: Some(total_market),
            amount: Some(amount),
            rate: Some(amount / total_purchase * Decimal::ONE_HUNDRED),
            incomplete: false,
        }
    }
}

/// 構成比（%）。`formatters.ts` の `calculatePercentage(value, total, 2)` に対応する。
/// JS の `toFixed(2)`（半分は切り上げ）に対応するため
/// `MidpointAwayFromZero` で丸める。
pub fn calculate_composition_percentage(value: Decimal, total: Decimal) -> Decimal {
    if total.is_zero() {
        Decimal::ZERO
    } else {
        (value / total * Decimal::ONE_HUNDRED)
            .round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero)
    }
}

/// 構成比の一覧。合計を分母に各要素の割合を求める。
pub fn composition_percentages(values: &[Decimal]) -> Vec<Decimal> {
    let total: Decimal = values.iter().copied().sum();
    values
        .iter()
        .map(|value| calculate_composition_percentage(*value, total))
        .collect()
}

/// チャート表示の除外条件。`AssetPortfolioSummary.tsx` の `chartData` の
/// `filter` に対応する。取得総額が正の銘柄は残し、そうでなければ
/// 評価額を持つ銘柄（欠損・0円以外）だけ残す。
pub fn should_include_chart_item(purchase: Option<Decimal>, market: Option<Decimal>) -> bool {
    if let Some(purchase) = purchase {
        if purchase > Decimal::ZERO {
            return true;
        }
    }
    matches!(market, Some(market) if !market.is_zero())
}

/// チャート用の未丸めパーセンテージ。`PortfolioPieChart.tsx` の
/// `percentage: (item.value / total) * 100` に対応する。
/// 分母が 0 の場合は算出不可として `None`（React の `NaN` に相当）を返す。
/// 評価額を持つ銘柄が1つもない空表示条件では空ベクターを返す。
pub fn chart_percentages(
    values: &[Decimal],
    market_values: &[Option<Decimal>],
) -> Vec<Option<Decimal>> {
    let total: Decimal = values.iter().copied().sum();
    if total.is_zero() {
        let has_valuation = market_values
            .iter()
            .any(|market| matches!(market, Some(value) if !value.is_zero()));
        if !has_valuation {
            return Vec::new();
        }
        return values.iter().map(|_| None).collect();
    }
    values
        .iter()
        .map(|value| Some(*value / total * Decimal::ONE_HUNDRED))
        .collect()
}

/// KPI 計算への入力1件。欠損は呼び出し側で `Decimal::ZERO` に寄せる
///（React の `item.shares || 0`、`item.total_purchase_amount || 0` に対応）。
#[derive(Clone, Debug)]
pub struct KpiHolding {
    pub security_code: String,
    pub shares: Decimal,
    pub total_purchase_amount: Decimal,
}

/// ポートフォリオ KPI。`AssetPortfolioSummary.tsx` の集計に対応する。
#[derive(Clone, Debug, PartialEq)]
pub struct PortfolioKpi {
    pub total_purchase_amount: Decimal,
    pub total_annual_dividends: Option<Decimal>,
    pub dividend_yield: Option<Decimal>,
    pub holdings_count: usize,
}

/// 合計取得総額・年間配当・配当利回り・銘柄数を求める。
/// 年間配当 = Σ(1株配当 × 保有株数)、配当利回り = 年間配当 / 取得総額 × 100。
/// 配当マップが空、または合計が 0 の場合は配当・利回りとも算出不可（`None`）。
pub fn calculate_portfolio_kpi(
    holdings: &[KpiHolding],
    dividends_per_share: &HashMap<String, Decimal>,
) -> PortfolioKpi {
    let total_purchase_amount: Decimal =
        holdings.iter().map(|item| item.total_purchase_amount).sum();
    if dividends_per_share.is_empty() {
        return PortfolioKpi {
            total_purchase_amount,
            total_annual_dividends: None,
            dividend_yield: None,
            holdings_count: holdings.len(),
        };
    }
    let mut total = Decimal::ZERO;
    for item in holdings {
        if let Some(per_share) = dividends_per_share.get(&item.security_code) {
            total += *per_share * item.shares;
        }
    }
    if total.is_zero() {
        return PortfolioKpi {
            total_purchase_amount,
            total_annual_dividends: None,
            dividend_yield: None,
            holdings_count: holdings.len(),
        };
    }
    let dividend_yield = if total_purchase_amount > Decimal::ZERO {
        Some(total / total_purchase_amount * Decimal::ONE_HUNDRED)
    } else {
        None
    };
    PortfolioKpi {
        total_purchase_amount,
        total_annual_dividends: Some(total),
        dividend_yield,
        holdings_count: holdings.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use serde::Deserialize;

    const RATE_TOLERANCE: Decimal = Decimal::from_parts(1, 0, 0, false, 9);

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
        values: Vec<Decimal>,
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
        serde_json::from_str(include_str!(
            "../tests/fixtures/asset_balance/valuation.json"
        ))
        .expect("shared asset balance fixture parses")
    }

    fn expected_decimal(value: &Value) -> Option<Decimal> {
        match value {
            Value::Null => None,
            number => Some(Decimal::from_str(&number.to_string()).expect("fixture number")),
        }
    }

    fn assert_optional_decimal(actual: Option<Decimal>, expected: &Value, case: &str, field: &str) {
        assert_eq!(actual, expected_decimal(expected), "{case} {field} (exact)");
    }

    fn assert_optional_rate(actual: Option<Decimal>, expected: &Value, case: &str, field: &str) {
        match (actual, expected_decimal(expected)) {
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

    fn json_decimal_or_zero(value: &Value) -> Decimal {
        to_finite_amount(value).unwrap_or(Decimal::ZERO)
    }

    #[test]
    fn shared_valuation_cases_match() {
        let fixture = fixture();
        assert_eq!(fixture.valuation_cases.len(), 9);
        for case in &fixture.valuation_cases {
            let result = calculate_valuation(&case.market_value, &case.purchase_amount);
            assert_optional_decimal(result.amount, &case.expected.amount, &case.name, "amount");
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
            assert_optional_decimal(
                summary.market_value,
                &case.expected.market_value,
                &case.name,
                "marketValue",
            );
            assert_optional_decimal(summary.amount, &case.expected.amount, &case.name, "amount");
            assert_optional_rate(summary.rate, &case.expected.rate, &case.name, "rate");
        }
    }

    #[test]
    fn shared_composition_cases_match() {
        let fixture = fixture();
        assert_eq!(fixture.composition_cases.len(), 5);
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
                    shares: json_decimal_or_zero(&item.shares),
                    total_purchase_amount: json_decimal_or_zero(&item.total_purchase_amount),
                })
                .collect();
            let dividends: HashMap<String, Decimal> = case
                .dividends_per_share
                .iter()
                .map(|(code, value)| {
                    (
                        code.clone(),
                        Decimal::from_str(&value.to_string()).expect("dividend per share"),
                    )
                })
                .collect();
            let kpi = calculate_portfolio_kpi(&holdings, &dividends);
            assert_optional_decimal(
                Some(kpi.total_purchase_amount),
                &case.expected.total_purchase_amount,
                &case.name,
                "total_purchase_amount",
            );
            assert_optional_decimal(
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

    #[test]
    fn chart_filter_matches_component_conditions() {
        assert!(should_include_chart_item(Some(dec!(1)), None));
        assert!(should_include_chart_item(
            Some(dec!(250000)),
            Some(dec!(260000))
        ));
        assert!(!should_include_chart_item(Some(dec!(0)), Some(dec!(0))));
        assert!(!should_include_chart_item(Some(dec!(0)), None));
        assert!(!should_include_chart_item(None, None));
        assert!(should_include_chart_item(Some(dec!(0)), Some(dec!(100000))));
        assert!(should_include_chart_item(None, Some(dec!(50000))));
    }

    #[test]
    fn chart_percentages_match_component_conditions() {
        assert!(chart_percentages(&[], &[]).is_empty());
        assert!(chart_percentages(&[dec!(0), dec!(0)], &[Some(dec!(0)), Some(dec!(0))]).is_empty());
        assert_eq!(
            chart_percentages(&[dec!(0)], &[Some(dec!(50000))]),
            vec![None]
        );
        let percentages = chart_percentages(
            &[dec!(250000), dec!(600000)],
            &[Some(dec!(1)), Some(dec!(1))],
        );
        assert_eq!(percentages.len(), 2);
        let total = dec!(850000);
        assert_eq!(
            percentages,
            vec![
                Some(dec!(250000) / total * Decimal::ONE_HUNDRED),
                Some(dec!(600000) / total * Decimal::ONE_HUNDRED),
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
            Some(dec!(1180000))
        );
        assert_eq!(to_finite_amount(&Value::Null), None);
    }
}
