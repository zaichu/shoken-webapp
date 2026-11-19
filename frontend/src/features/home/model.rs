//! ホームダッシュボードの純粋な集計ロジック。表示用のフォーマットは view 側。

use crate::api::dto::{
    AssetBalance, AssetBalanceSummary, Dividend, FundBalance, FundBalanceSummary,
};
use crate::features::asset_balance;
use crate::features::asset_balance::{KpiHolding, PortfolioKpi, ValuationInput};
use crate::features::dividend_per_share::DividendMaps;
use crate::ui::badge;
use crate::ui::format;
use crate::ui::labels;
use crate::urls;
use rust_decimal::Decimal;
use shared::tax::TAX_RATE;

/// 株式・ファンドの集計を合算した保有全体の集計。
/// ファンド側に priced_* の区別がないため、価格あり集計にはファンドは全件含める
pub(crate) fn merged_asset_summary(
    stock: Option<&AssetBalanceSummary>,
    fund: Option<&FundBalanceSummary>,
) -> Option<AssetBalanceSummary> {
    if stock.is_none() && fund.is_none() {
        return None;
    }
    let pick = |f: &dyn Fn(&AssetBalanceSummary) -> Decimal| stock.map_or_default(f);
    let fund_market = fund.map_or_default(|s| s.total_market_value);
    let fund_purchase = fund.map_or_default(|s| s.total_purchase_amount);
    let fund_daily = fund.map_or_default(|s| s.total_daily_change);
    Some(AssetBalanceSummary {
        total_market_value: pick(&|s| s.total_market_value) + fund_market,
        total_purchase_amount: pick(&|s| s.total_purchase_amount) + fund_purchase,
        total_daily_change: pick(&|s| s.total_daily_change) + fund_daily,
        priced_market_value: pick(&|s| s.priced_market_value) + fund_market,
        priced_purchase_amount: pick(&|s| s.priced_purchase_amount) + fund_purchase,
        has_missing_daily_change: stock.is_some_and(|s| s.has_missing_daily_change),
    })
}

/// 集計が欠けたときの行からの合成。価格なし行は評価額に算入しない(API の集計と同じ扱い)
pub(crate) fn stock_summary_from_rows(rows: &[AssetBalance]) -> AssetBalanceSummary {
    let mut summary = AssetBalanceSummary::default();
    for row in rows {
        let price = row.current_price;
        let shares = row.shares;
        summary.total_purchase_amount += row.total_purchase_amount;
        if row.prev_close.is_some() || row.price_as_of.is_none() {
            summary.total_daily_change += row.daily_change * shares;
        } else {
            summary.has_missing_daily_change = true;
        }
        if price > Decimal::ZERO {
            summary.total_market_value += shares * price;
            summary.priced_market_value += shares * price;
            summary.priced_purchase_amount += row.total_purchase_amount;
        }
    }
    summary
}

pub(crate) fn fund_summary_from_rows(rows: &[FundBalance]) -> FundBalanceSummary {
    let mut summary = FundBalanceSummary::default();
    for row in rows {
        summary.total_market_value += row.market_value;
        summary.total_purchase_amount += row.total_purchase_amount;
    }
    summary
}

/// ホーム帯の前日比。株式0件の正常応答(COALESCEのゼロ集計)も欠測として扱い、
/// 投信側は基準価額の取得件数が0のとき欠測にする。
/// 株式・投信のどちらも算出できないときは全体も欠測にする(0と断定しない)。
/// 株式側は資産管理の国内株式タブと同じく API 集計の値をそのまま使う
pub(crate) fn merged_daily_change(
    stock: Option<&AssetBalanceSummary>,
    has_stocks: bool,
    fund: Option<&FundBalanceSummary>,
) -> Option<f64> {
    let stock_daily = has_stocks
        .then(|| stock.map(|summary| asset_balance::dec_to_f64(&summary.total_daily_change)))
        .flatten();
    let fund_daily = fund.and_then(|summary| {
        (summary.latest_nav_count > 0)
            .then(|| asset_balance::dec_to_f64(&summary.total_daily_change))
    });
    match (stock_daily, fund_daily) {
        (Some(stock), Some(fund)) => Some(stock + fund),
        (stock, fund) => stock.or(fund),
    }
}

/// ホーム帯の時点。株と投信を合わせ、最も古い日を出す(嘘の新しさを見せない)。
/// 価格なし株は時点と結びつかないため除く
pub(crate) fn oldest_home_as_of(stocks: &[AssetBalance], funds: &[FundBalance]) -> Option<String> {
    oldest_home_price_iso(stocks, funds)
        .as_deref()
        .map(labels::valuation_point_label)
        .filter(|label| !label.is_empty())
}

/// ホーム帯の最古の価格日(YYYY-MM-DD)。比較用に素の日付を返す
pub(crate) fn oldest_home_price_iso(
    stocks: &[AssetBalance],
    funds: &[FundBalance],
) -> Option<String> {
    let mut dates: Vec<String> = stocks
        .iter()
        .filter(|row| row.current_price > Decimal::ZERO)
        .filter_map(|row| row.price_as_of.clone())
        .map(|date| date.to_string())
        .collect();
    dates.extend(funds.iter().map(|row| row.price_as_of.to_string()));
    dates.sort();
    dates.into_iter().next()
}

/// ホーム帯の更新待ち。評価額の計算に使う価格が無い株か基準価額が古い投信があれば true
pub(crate) fn home_has_pending(
    stocks: &[AssetBalance],
    funds: &[FundBalance],
    stock_summary: Option<&AssetBalanceSummary>,
) -> bool {
    let stock_unpriced = match stock_summary {
        Some(summary) if !stocks.is_empty() => {
            asset_balance::summary_valuation(summary).has_unpriced
        }
        _ => stocks.iter().any(|row| row.current_price <= Decimal::ZERO),
    };
    let fund_stale = funds.iter().any(|row| !row.nav_is_latest);
    asset_balance::valuation_has_pending(stock_unpriced, fund_stale)
}

/// 年間予想配当(税引前)と取得額に対する率。資産管理の KPI と同じく
/// 株式の取得総額を分母にする(ファンドは予想配当の対象外)
pub(crate) fn annual_dividend_forecast(
    stock_rows: &[AssetBalance],
    maps: &DividendMaps,
) -> PortfolioKpi {
    let holdings: Vec<KpiHolding> = stock_rows
        .iter()
        .map(|row| KpiHolding {
            security_code: row.security_code.to_string(),
            shares: asset_balance::dec_to_f64(&row.shares),
            total_purchase_amount: asset_balance::dec_to_f64(&row.total_purchase_amount),
        })
        .collect();
    asset_balance::calculate_portfolio_kpi(&holdings, &maps.per_share)
}

/// 予想配当の推定値がまだ計算中(pending)の銘柄が残っているか
pub(crate) fn has_pending_estimate(maps: &DividendMaps) -> bool {
    maps.status.values().any(|status| status == "pending")
}

fn month_of(settlement_date: &str) -> Option<u32> {
    format::date_parts(settlement_date).map(|(_, month, _)| month)
}

/// 税引後入金額の月別合計(1月..12月)
pub(crate) fn monthly_net_totals(rows: &[Dividend]) -> [Decimal; 12] {
    let mut totals = [Decimal::ZERO; 12];
    for row in rows {
        if let Some(month) = month_of(&row.settlement_date) {
            totals[(month - 1) as usize] += row.net_amount_received;
        }
    }
    totals
}

/// 実効税率 = Σ税額 ÷ Σ税引前。配当に NISA(税0)と課税口座が混ざるため、
/// 予想の税引後換算には実績の加重平均を使う。税引前合計が 0 なら算出不可
pub(crate) fn effective_tax_rate(primary: &[Dividend], fallback: &[Dividend]) -> Option<Decimal> {
    let before: Decimal = primary
        .iter()
        .chain(fallback.iter())
        .map(|row| row.dividends_before_tax)
        .sum();
    if before.is_zero() {
        return None;
    }
    let tax: Decimal = primary
        .iter()
        .chain(fallback.iter())
        .map(|row| row.taxes)
        .sum();
    Some(tax / before)
}

/// 今年の配当金カードの表示モデル。
pub(crate) struct DividendCardModel {
    /// 今年の税引後受取額
    pub(crate) received: Decimal,
    /// 年間予想配当(税引前)を実効税率で税引後に換算した予想
    pub(crate) forecast_net: Option<f64>,
    /// 予想に対する受取の割合(0..1 超過あり)
    pub(crate) progress: Option<f64>,
    /// 月別の(今年, 前年)入金額
    pub(crate) monthly: Vec<MonthlyDividends>,
}

pub(crate) struct MonthlyDividends {
    pub(crate) month: u32,
    pub(crate) current: Decimal,
    pub(crate) previous: Decimal,
}

pub(crate) fn build_dividend_card(
    current_rows: &[Dividend],
    previous_rows: Option<&[Dividend]>,
    forecast_gross: Option<f64>,
) -> DividendCardModel {
    let previous_rows = previous_rows.unwrap_or(&[]);
    let received: Decimal = current_rows.iter().map(|row| row.net_amount_received).sum();
    let rate = effective_tax_rate(current_rows, previous_rows).unwrap_or(TAX_RATE);
    let forecast_net = forecast_gross.map(|gross| gross * (1.0 - asset_balance::dec_to_f64(&rate)));
    let received_f64 = asset_balance::dec_to_f64(&received);
    let progress = forecast_net
        .filter(|forecast| *forecast > 0.0)
        .map(|forecast| received_f64 / forecast);
    let previous_totals = monthly_net_totals(previous_rows);
    let current_totals = monthly_net_totals(current_rows);
    let monthly = (1..=12u32)
        .map(|month| MonthlyDividends {
            month,
            current: current_totals[(month - 1) as usize],
            previous: previous_totals[(month - 1) as usize],
        })
        .collect();
    DividendCardModel {
        received,
        forecast_net,
        progress,
        monthly,
    }
}

/// 進捗率の表示。100% を超えてもバーは満タン(100%)で止め、
/// 超えたことは `over` と文言で見せる。計算は変えない(受取 ÷ 年間予想(税引後))
pub(crate) struct ProgressDisplay {
    /// 「1611%」などの表示文。予想なしは「—」
    pub(crate) text: String,
    /// バーの幅(%)。0..=100 に丸める
    pub(crate) bar_pct: f64,
}

pub(crate) fn progress_display(progress: Option<f64>) -> ProgressDisplay {
    progress.map_or_else(
        || ProgressDisplay {
            text: labels::MISSING.to_string(),
            bar_pct: 0.0,
        },
        |value| {
            let pct = value * 100.0;
            ProgressDisplay {
                text: format::format_integer_percent(pct),
                bar_pct: pct.clamp(0.0, 100.0),
            }
        },
    )
}

/// 内訳チャートの1区分
pub(crate) struct CompositionSlice {
    pub(crate) label: String,
    pub(crate) amount: f64,
}

pub(crate) struct ProductCompositionSlice {
    pub(crate) label: String,
    pub(crate) amount: f64,
    pub(crate) item_count: usize,
}

/// 商品別の資産内訳(時価ベース)。0 以下の区分は落とし、大きい順に並べる
pub(crate) fn product_slices(
    stocks: &[AssetBalance],
    funds: &[FundBalance],
    stock_market: f64,
    fund_market: f64,
) -> Vec<ProductCompositionSlice> {
    let mut slices = vec![
        ProductCompositionSlice {
            label: labels::DOMESTIC_STOCK.to_string(),
            amount: stock_market,
            item_count: stocks.len(),
        },
        ProductCompositionSlice {
            label: labels::MUTUAL_FUND.to_string(),
            amount: fund_market,
            item_count: funds.len(),
        },
    ];
    slices.retain(|slice| slice.amount > 0.0);
    slices.sort_by(|a, b| b.amount.total_cmp(&a.amount));
    slices
}

/// 口座別の資産内訳(時価ベース)。国内株式の口座は商品別と口座別を混ぜない仕様のため
/// 投資信託だけで構成する。ラベルは取引明細と同じ表示名対応表を通す
pub(crate) fn account_slices(funds: &[FundBalance]) -> Vec<CompositionSlice> {
    let mut by_account: Vec<(String, f64)> = Vec::new();
    for fund in funds {
        let account = badge::account_display_name(fund.account.as_ref()).to_string();
        let amount = asset_balance::dec_to_f64(&fund.market_value);
        match by_account.iter_mut().find(|(label, _)| *label == account) {
            Some((_, total)) => *total += amount,
            None => by_account.push((account, amount)),
        }
    }
    let mut slices: Vec<CompositionSlice> = by_account
        .into_iter()
        .map(|(label, amount)| CompositionSlice { label, amount })
        .collect();
    slices.retain(|slice| slice.amount > 0.0);
    slices.sort_by(|a, b| b.amount.total_cmp(&a.amount));
    slices
}

/// 含み損益ランキングの1行(株式・ファンド共通)
#[derive(Clone)]
pub(crate) struct HoldingPnl {
    pub(crate) code: String,
    pub(crate) name: String,
    /// 詳細ページへのパス(一覧の行と同じ保存済みレコード ID)
    pub(crate) href: String,
    pub(crate) profit_loss: f64,
    pub(crate) rate: Option<f64>,
}

/// 価格のある保有を含み損益の行に変換する。価格なし株式は損益が出せないため除外
pub(crate) fn holding_pnls(stocks: &[AssetBalance], funds: &[FundBalance]) -> Vec<HoldingPnl> {
    let stock_rows = stocks.iter().filter_map(|row| {
        let valuation = asset_balance::holding_valuation(&ValuationInput {
            current_price: asset_balance::dec_to_f64(&row.current_price),
            shares: asset_balance::dec_to_f64(&row.shares),
            total_purchase_amount: asset_balance::dec_to_f64(&row.total_purchase_amount),
        });
        let profit_loss = valuation.profit_loss.amount?;
        let name = if row.security_name.is_empty() {
            row.security_code.to_string()
        } else {
            asset_balance::normalize_display_name(&row.security_name)
        };
        Some(HoldingPnl {
            code: row.security_code.to_string(),
            name,
            href: asset_balance::holding_stock_href(row.security_code.as_ref()),
            profit_loss,
            rate: valuation.profit_loss.rate,
        })
    });
    let fund_rows = funds.iter().map(|row| HoldingPnl {
        code: String::new(),
        name: asset_balance::normalize_display_name(&row.fund_name),
        href: urls::fund_holding_href(&row.id),
        profit_loss: asset_balance::dec_to_f64(&row.profit_loss),
        rate: row.profit_loss_rate.as_ref().map(asset_balance::dec_to_f64),
    });
    stock_rows.chain(fund_rows).collect()
}

/// 含み益の大きい順に `gain_limit` 件、含み損の大きい順に `loss_limit` 件を選ぶ。
/// 同額は名前で安定させる
pub(crate) fn pnl_extremes(
    holdings: &[HoldingPnl],
    gain_limit: usize,
    loss_limit: usize,
) -> (Vec<HoldingPnl>, Vec<HoldingPnl>) {
    let by_amount_desc = |a: &HoldingPnl, b: &HoldingPnl| {
        b.profit_loss
            .total_cmp(&a.profit_loss)
            .then_with(|| a.name.cmp(&b.name))
    };
    let mut gains: Vec<&HoldingPnl> = holdings.iter().filter(|h| h.profit_loss > 0.0).collect();
    gains.sort_by(|a, b| by_amount_desc(a, b));
    let mut losses: Vec<&HoldingPnl> = holdings.iter().filter(|h| h.profit_loss < 0.0).collect();
    // 損は絶対額が大きい(= 値が小さい)順
    losses.sort_by(|a, b| {
        a.profit_loss
            .total_cmp(&b.profit_loss)
            .then_with(|| a.name.cmp(&b.name))
    });
    (
        gains.into_iter().take(gain_limit).cloned().collect(),
        losses.into_iter().take(loss_limit).cloned().collect(),
    )
}

#[cfg(test)]
mod tests;
