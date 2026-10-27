//! 資産管理の一覧・構成比・KPI と、ホームの評価損益表示の純粋ロジック。

use crate::api::dto::AssetBalanceSummary;
use crate::ui::format::intl_fixed;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use std::collections::HashMap;

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

/// `formatters.ts` の `safeAdd` に対応する。加算ごとに小数10桁で丸める。
pub fn safe_add(a: f64, b: f64) -> f64 {
    intl_fixed(a + b, 10)
}

/// 1銘柄の評価損益。`valuation.ts` の `calculateValuation` に対応する。
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ValuationResult {
    pub amount: Option<f64>,
    pub rate: Option<f64>,
}

/// 評価額と取得総額から含み損益(額・率)を求める。
/// どちらかが無い場合は算出不能(額・率とも None)。取得総額 0 なら率だけ None。
pub fn calculate_valuation(market: Option<f64>, purchase: Option<f64>) -> ValuationResult {
    let (Some(market), Some(purchase)) = (market, purchase) else {
        return ValuationResult {
            amount: None,
            rate: None,
        };
    };
    let (amount, rate) = valuation_parts(
        Decimal::from_str_exact(&market.to_string()).ok(),
        Decimal::from_str_exact(&purchase.to_string()).ok(),
        market,
        purchase,
    );
    ValuationResult {
        amount: Some(amount),
        rate,
    }
}

/// 評価額・含み損益の計算入力1件。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ValuationInput {
    /// 現在値。0 以下は価格なし(算出不能)として扱い、NaN も弾く
    pub current_price: f64,
    pub shares: f64,
    pub total_purchase_amount: f64,
}

// 現在値は「無い」と「0」を区別できないため、0 以下はまとめて価格なしとみなす
fn priced_market_value(input: &ValuationInput) -> Option<f64> {
    (input.current_price > 0.0).then_some(input.current_price * input.shares)
}

/// 1銘柄の評価額と含み損益。
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HoldingValuation {
    /// 評価額(現在値×数量)。価格なしは None
    pub market_value: Option<f64>,
    /// 含み損益(評価額−取得総額)と率(損益÷取得総額)
    pub profit_loss: ValuationResult,
}

pub fn holding_valuation(input: &ValuationInput) -> HoldingValuation {
    let market_value = priced_market_value(input);
    HoldingValuation {
        market_value,
        profit_loss: calculate_valuation(market_value, Some(input.total_purchase_amount)),
    }
}

/// 保有全体の評価額と含み損益。
/// 価格なし銘柄は分子・分母の両方から外し、取得額分の損と誤表示しない。
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PortfolioValuation {
    /// 評価額合計(価格あり銘柄のみ)。価格のある銘柄が1件も無ければ None
    pub market_value: Option<f64>,
    /// 含み損益合計と率。率の分母は価格あり銘柄の取得総額
    pub profit_loss: ValuationResult,
    /// 価格が無く集計から外した銘柄がある
    pub has_unpriced: bool,
}

pub fn portfolio_valuation(holdings: &[ValuationInput]) -> PortfolioValuation {
    let mut market = 0.0;
    let mut priced_purchase = 0.0;
    let mut has_priced = false;
    let mut has_unpriced = false;
    for input in holdings {
        match priced_market_value(input) {
            Some(value) => {
                market = safe_add(market, value);
                priced_purchase = safe_add(priced_purchase, input.total_purchase_amount);
                has_priced = true;
            }
            None => has_unpriced = true,
        }
    }
    let market_value = has_priced.then_some(market);
    PortfolioValuation {
        market_value,
        profit_loss: calculate_valuation(market_value, Some(priced_purchase)),
        has_unpriced,
    }
}

/// 一覧の取得上限で行が欠けるときのフォールバック。API 集計値から評価額・含み損益を出す。
/// `priced_*` は価格あり行だけの集計で個別カードと同じ対象行になる。
/// `priced_purchase_amount < total_purchase_amount` なら価格なし行がいるとみなす。
pub fn summary_valuation(summary: &AssetBalanceSummary) -> PortfolioValuation {
    let market = summary.priced_market_value.to_f64().unwrap_or(0.0);
    let market_value = (market > 0.0).then_some(market);
    let priced_purchase = summary.priced_purchase_amount.to_f64().unwrap_or(0.0);
    let total_purchase = summary.total_purchase_amount.to_f64().unwrap_or(0.0);
    PortfolioValuation {
        market_value,
        profit_loss: calculate_valuation(market_value, Some(priced_purchase)),
        has_unpriced: priced_purchase < total_purchase,
    }
}

/// チャート用の未丸めパーセンテージ（`|item.value| / Σ|values| * 100`）。
/// 分母が 0 の場合は空ベクターを返す。
/// 取得額にマイナスが混ざっても帯グラフの幅が 0〜100% に収まるよう絶対値で出す。
pub fn chart_percentages(values: &[f64]) -> Vec<Option<f64>> {
    let total: f64 = values.iter().map(|value| value.abs()).sum();
    if total == 0.0 {
        return Vec::new();
    }
    values
        .iter()
        .map(|value| Some(value.abs() / total * 100.0))
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

/// 合計取得総額。表示中データから `safeAdd` で積み上げる。
pub fn total_purchase_amount(holdings: &[KpiHolding]) -> f64 {
    holdings
        .iter()
        .fold(0.0, |sum, item| safe_add(sum, item.total_purchase_amount))
}

/// 合計取得総額・年間配当・配当利回り・銘柄数を求める。
/// 年間配当 = Σ(1株配当 × 保有株数)、配当利回り = 年間配当 / 取得総額 × 100。
/// 配当マップが空、または合計が 0 の場合は配当・利回りとも算出不可（`None`）。
pub fn calculate_portfolio_kpi(
    holdings: &[KpiHolding],
    dividends_per_share: &HashMap<String, f64>,
) -> PortfolioKpi {
    let total_purchase_amount = total_purchase_amount(holdings);
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
