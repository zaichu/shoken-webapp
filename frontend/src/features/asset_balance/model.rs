//! 資産管理の一覧・評価・構成比・KPI の純粋ロジック。

use rust_decimal::prelude::ToPrimitive;
use rust_decimal::{Decimal, RoundingStrategy};
use std::collections::HashMap;

fn f64_to_decimal_exact(value: f64) -> Option<Decimal> {
    Decimal::from_str_exact(&value.to_string()).ok()
}

fn valuation_parts(
    market_dec: Option<Decimal>,
    purchase_dec: Option<Decimal>,
    market: f64,
    purchase: f64,
) -> (f64, Option<f64>) {
    let amount_dec = market_dec
        .zip(purchase_dec)
        .and_then(|(market, purchase)| market.checked_sub(purchase));
    let amount = amount_dec
        .and_then(|value| value.to_f64())
        .unwrap_or(market - purchase);
    let rate = if purchase == 0.0 {
        None
    } else {
        amount_dec
            .zip(purchase_dec)
            .and_then(|(amount, purchase)| amount.checked_div(purchase))
            .and_then(|rate| rate.checked_mul(Decimal::ONE_HUNDRED))
            .and_then(|rate| rate.to_f64())
            .or_else(|| Some(amount / purchase * 100.0))
    };
    (amount, rate)
}

/// JS の `Number(value.toFixed(decimals))` と同じ結果を返す。
/// f64 の2進の値で最も近い `10^-decimals` の倍数を選び、同距離なら大きい方を選ぶ。
/// 負値は符号を分けて絶対値で丸めてから符号を戻す。
/// 非有限値と巨大値はそのまま返し、panic しない。
pub fn to_fixed(value: f64, decimals: u32) -> f64 {
    if !value.is_finite() {
        return value;
    }
    let negative = value.is_sign_negative();
    let magnitude = value.abs();
    if magnitude == 0.0 {
        return 0.0;
    }
    let bits = magnitude.to_bits();
    let raw_exponent = ((bits >> 52) & 0x7ff) as i32;
    let (mantissa, exponent) = if raw_exponent == 0 {
        (bits & 0xfffffffffffff, -1074)
    } else {
        (
            bits & 0xfffffffffffff | 0x10000000000000,
            raw_exponent - 1075,
        )
    };
    if exponent >= 0 {
        return value;
    }
    let shift = -exponent;
    if shift >= 128 {
        return if negative { -0.0 } else { 0.0 };
    }
    let numerator = (mantissa as u128) * 10u128.pow(decimals);
    let quantum = 1u128 << (shift as u32);
    let rounded = numerator / quantum + u128::from(numerator % quantum * 2 >= quantum);
    if rounded == 0 {
        return if negative { -0.0 } else { 0.0 };
    }
    let result = rounded as f64 / 10f64.powi(decimals as i32);
    if negative {
        -result
    } else {
        result
    }
}

/// `Intl.NumberFormat`(maximumFractionDigits) 相当の丸め。
/// `toFixed` と異なり、f64 の最短10進表記を10進の値として丸めるため、
/// 捨てる部分が半分以上なら切り上げる（`1.005` → `1.01`、`2.675` → `2.68`）。
/// `Decimal` に載らない巨大値・多桁値は `to_fixed` にフォールバックする。
pub fn intl_fixed(value: f64, decimals: u32) -> f64 {
    if !value.is_finite() {
        return value;
    }
    let negative = value.is_sign_negative();
    let fallback = || to_fixed(value, decimals);
    let Ok(decimal) = Decimal::from_str_exact(&value.abs().to_string()) else {
        return fallback();
    };
    let Some(result) = decimal
        .round_dp_with_strategy(decimals, RoundingStrategy::MidpointAwayFromZero)
        .to_f64()
    else {
        return fallback();
    };
    if negative {
        -result
    } else {
        result
    }
}

/// `formatters.ts` の `safeAdd` に対応する。加算ごとに小数10桁で丸める。
pub fn safe_add(a: f64, b: f64) -> f64 {
    to_fixed(a + b, 10)
}

fn group_thousands(digits: &str) -> String {
    let mut grouped = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, byte) in digits.bytes().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            grouped.push(',');
        }
        grouped.push(byte as char);
    }
    grouped
}

// 符号は呼び出し側が丸め前の値で `value < 0` と判定する（`-0.0` は負としない）。
// この関数は絶対値の桁区切りだけを返す。
pub(crate) fn format_abs_number(value: f64) -> Option<String> {
    if !value.is_finite() {
        return None;
    }
    let text = value.abs().to_string();
    let (integer, fraction) = match text.split_once('.') {
        Some((integer, fraction)) => (integer, Some(fraction)),
        None => (text.as_str(), None),
    };
    let grouped = group_thousands(integer);
    Some(match fraction {
        Some(fraction) => format!("{grouped}.{fraction}"),
        None => grouped,
    })
}

pub fn format_number_value(value: f64) -> String {
    match format_abs_number(intl_fixed(value, 2)) {
        None => "—".to_string(),
        Some(body) if value < 0.0 => format!("-{body}"),
        Some(body) => body,
    }
}

/// 1銘柄の評価損益。`valuation.ts` の `calculateValuation` に対応する。
#[derive(Clone, Debug, PartialEq)]
pub struct ValuationResult {
    pub amount: Option<f64>,
    pub rate: Option<f64>,
}

pub fn calculate_valuation_from_decimal(market: Decimal, purchase: Decimal) -> ValuationResult {
    let (amount, rate) = valuation_parts(
        Some(market),
        Some(purchase),
        market.to_f64().unwrap_or(0.0),
        purchase.to_f64().unwrap_or(0.0),
    );
    ValuationResult {
        amount: Some(amount),
        rate,
    }
}

/// `summarizeValuation` への入力1件。
#[derive(Clone, Debug)]
pub struct ValuationItem {
    pub market_value: Option<f64>,
    pub total_purchase_amount: Option<f64>,
}

/// 複数銘柄の合計。`valuation.ts` の `summarizeValuation` に対応する。
/// 率は合計金額から計算し、単純平均しない。
#[derive(Clone, Debug, PartialEq)]
pub struct ValuationSummary {
    pub market_value: Option<f64>,
    pub amount: Option<f64>,
    pub rate: Option<f64>,
    pub incomplete: bool,
}

pub fn summarize_valuation(items: &[ValuationItem]) -> ValuationSummary {
    let mut market_dec = Decimal::ZERO;
    let mut purchase_dec = Decimal::ZERO;
    let mut market_f64 = 0.0;
    let mut purchase_f64 = 0.0;
    let mut exact = true;
    for item in items {
        let (Some(market), Some(purchase)) = (item.market_value, item.total_purchase_amount) else {
            return ValuationSummary {
                market_value: None,
                amount: None,
                rate: None,
                incomplete: true,
            };
        };
        market_f64 += market;
        purchase_f64 += purchase;
        if exact {
            exact = match (f64_to_decimal_exact(market), f64_to_decimal_exact(purchase)) {
                (Some(market), Some(purchase)) => match (
                    market_dec.checked_add(market),
                    purchase_dec.checked_add(purchase),
                ) {
                    (Some(market), Some(purchase)) => {
                        market_dec = market;
                        purchase_dec = purchase;
                        true
                    }
                    _ => false,
                },
                _ => false,
            };
        }
    }
    let (market_dec, purchase_dec) = if exact {
        (Some(market_dec), Some(purchase_dec))
    } else {
        (None, None)
    };
    let market_total = market_dec
        .and_then(|value| value.to_f64())
        .unwrap_or(market_f64);
    let purchase_total = purchase_dec
        .and_then(|value| value.to_f64())
        .unwrap_or(purchase_f64);
    let (amount, rate) = valuation_parts(market_dec, purchase_dec, market_total, purchase_total);
    ValuationSummary {
        market_value: Some(market_total),
        amount: Some(amount),
        rate,
        incomplete: false,
    }
}

/// API summary による上書き入力。
#[derive(Clone, Debug)]
pub struct SummaryOverride {
    pub total_purchase_amount: Decimal,
    pub total_market_value: Decimal,
}

impl SummaryOverride {
    pub fn sum(items: impl IntoIterator<Item = (Decimal, Decimal)>) -> Option<Self> {
        items.into_iter().try_fold(
            Self {
                total_purchase_amount: Decimal::ZERO,
                total_market_value: Decimal::ZERO,
            },
            |total, (market, purchase)| {
                Some(Self {
                    total_purchase_amount: total.total_purchase_amount.checked_add(purchase)?,
                    total_market_value: total.total_market_value.checked_add(market)?,
                })
            },
        )
    }
}

/// 評価損益の集計。
/// summary がある場合は検索条件全体の集計を優先し、なければ表示中データから集計する。
/// summary の欠損、または明細側の欠損がある場合は不完全として金額・率を表示しない。
pub fn summarize_valuation_with_summary(
    items: &[ValuationItem],
    summary: Option<&SummaryOverride>,
) -> ValuationSummary {
    let detail = summarize_valuation(items);
    let Some(summary) = summary else {
        return detail;
    };
    if detail.incomplete {
        return ValuationSummary {
            market_value: None,
            amount: None,
            rate: None,
            incomplete: true,
        };
    }
    let purchase = summary.total_purchase_amount;
    let market = summary.total_market_value;
    let (amount, rate) = valuation_parts(
        Some(market),
        Some(purchase),
        market.to_f64().unwrap_or(0.0),
        purchase.to_f64().unwrap_or(0.0),
    );
    ValuationSummary {
        market_value: market.to_f64(),
        amount: Some(amount),
        rate,
        incomplete: false,
    }
}

/// チャート表示の除外条件。取得総額が正の銘柄は残し、そうでなければ
/// 評価額を持つ銘柄（欠損・0円以外）だけ残す。
pub fn should_include_chart_item(purchase: Option<f64>, market: Option<f64>) -> bool {
    if let Some(purchase) = purchase {
        if purchase > 0.0 {
            return true;
        }
    }
    matches!(market, Some(market) if market != 0.0)
}

/// チャート用の未丸めパーセンテージ（`item.value / total * 100`）。
/// 分母が 0 の場合は算出不可として `None` を返す。
/// 評価額を持つ銘柄が1つもない空表示条件では空ベクターを返す。
pub fn chart_percentages(values: &[f64], market_values: &[Option<f64>]) -> Vec<Option<f64>> {
    let total: f64 = values.iter().sum();
    if total == 0.0 {
        let has_valuation = market_values
            .iter()
            .any(|market| matches!(market, Some(value) if *value != 0.0));
        if !has_valuation {
            return Vec::new();
        }
        return values.iter().map(|_| None).collect();
    }
    values
        .iter()
        .map(|value| Some(*value / total * 100.0))
        .collect()
}

/// KPI 計算への入力1件。欠損は呼び出し側で `0.0` に寄せる。
#[derive(Clone, Debug)]
pub struct KpiHolding {
    pub security_code: String,
    pub shares: f64,
    pub total_purchase_amount: f64,
}

/// ポートフォリオ KPI。
#[derive(Clone, Debug, PartialEq)]
pub struct PortfolioKpi {
    pub total_purchase_amount: f64,
    pub total_annual_dividends: Option<f64>,
    pub dividend_yield: Option<f64>,
    pub holdings_count: usize,
}

/// 合計取得総額。summary がある場合は検索条件全体の集計を優先し、
/// なければ表示中データから `safeAdd` で積み上げる。
pub fn total_purchase_amount(holdings: &[KpiHolding], summary_total: Option<f64>) -> f64 {
    match summary_total {
        Some(total) => total,
        None => holdings
            .iter()
            .fold(0.0, |sum, item| safe_add(sum, item.total_purchase_amount)),
    }
}

/// 合計取得総額・年間配当・配当利回り・銘柄数を求める。
/// 年間配当 = Σ(1株配当 × 保有株数)、配当利回り = 年間配当 / 取得総額 × 100。
/// 配当マップが空、または合計が 0 の場合は配当・利回りとも算出不可（`None`）。
pub fn calculate_portfolio_kpi(
    holdings: &[KpiHolding],
    dividends_per_share: &HashMap<String, f64>,
    summary_total: Option<f64>,
) -> PortfolioKpi {
    let total_purchase_amount = total_purchase_amount(holdings, summary_total);
    if dividends_per_share.is_empty() {
        return PortfolioKpi {
            total_purchase_amount,
            total_annual_dividends: None,
            dividend_yield: None,
            holdings_count: holdings.len(),
        };
    }
    let mut total = 0.0;
    for item in holdings {
        if let Some(per_share) = dividends_per_share.get(&item.security_code) {
            total += per_share * item.shares;
        }
    }
    if total == 0.0 {
        return PortfolioKpi {
            total_purchase_amount,
            total_annual_dividends: None,
            dividend_yield: None,
            holdings_count: holdings.len(),
        };
    }
    let dividend_yield = if total_purchase_amount > 0.0 {
        Some(total / total_purchase_amount * 100.0)
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

pub use shared::normalize::{normalize_display_name, normalize_security_code};

#[cfg(test)]
mod tests;
