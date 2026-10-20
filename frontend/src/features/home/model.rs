//! ホームダッシュボードの純粋な集計ロジック。表示用のフォーマットは view 側。

use crate::api::dto::{
    AssetBalance, AssetBalanceSummary, Dividend, FundBalance, FundBalanceSummary,
};
use crate::features::asset_balance::{
    KpiHolding, PortfolioKpi, ValuationInput, calculate_portfolio_kpi, dec_to_f64,
    format_valuation_amount, holding_valuation, normalize_display_name,
};
use crate::features::dividend_per_share::DividendMaps;
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
    let pick = |f: &dyn Fn(&AssetBalanceSummary) -> Decimal| stock.map(f).unwrap_or_default();
    let fund_market = fund.map(|s| s.total_market_value).unwrap_or_default();
    let fund_purchase = fund.map(|s| s.total_purchase_amount).unwrap_or_default();
    let fund_daily = fund.map(|s| s.total_daily_change).unwrap_or_default();
    Some(AssetBalanceSummary {
        total_market_value: pick(&|s| s.total_market_value) + fund_market,
        total_purchase_amount: pick(&|s| s.total_purchase_amount) + fund_purchase,
        total_daily_change: pick(&|s| s.total_daily_change) + fund_daily,
        priced_market_value: pick(&|s| s.priced_market_value) + fund_market,
        priced_purchase_amount: pick(&|s| s.priced_purchase_amount) + fund_purchase,
    })
}

/// 集計が欠けたときの行からの合成。価格なし行は評価額に算入しない(API の集計と同じ扱い)
pub(crate) fn stock_summary_from_rows(rows: &[AssetBalance]) -> AssetBalanceSummary {
    let mut summary = AssetBalanceSummary::default();
    for row in rows {
        let price = row.current_price;
        let shares = row.shares;
        summary.total_purchase_amount += row.total_purchase_amount;
        summary.total_daily_change += row.daily_change;
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
            shares: dec_to_f64(&row.shares),
            total_purchase_amount: dec_to_f64(&row.total_purchase_amount),
        })
        .collect();
    calculate_portfolio_kpi(&holdings, &maps.per_share)
}

/// 予想配当の推定値がまだ計算中(pending)の銘柄が残っているか
pub(crate) fn has_pending_estimate(maps: &DividendMaps) -> bool {
    maps.status.values().any(|status| status == "pending")
}

fn date_parts(date: &str) -> Option<(i32, u32, u32)> {
    if !date.is_ascii()
        || date.len() < 10
        || date.as_bytes()[4] != b'-'
        || date.as_bytes()[7] != b'-'
    {
        return None;
    }
    let year: i32 = date[..4].parse().ok()?;
    let month: u32 = date[5..7].parse().ok()?;
    let day: u32 = date[8..10].parse().ok()?;
    if !(1..=12).contains(&month) || day == 0 {
        return None;
    }
    Some((year, month, day))
}

/// updated_at(UTC) を JST の取込日として出す。+9h で日付が繰り上がる場合だけ桁を進める
pub(crate) fn jst_date(iso: &str) -> Option<(i32, u32, u32)> {
    let (mut year, mut month, mut day) = date_parts(iso)?;
    let hour: u32 = if iso.len() >= 13 && (iso.as_bytes()[10] == b'T' || iso.as_bytes()[10] == b' ')
    {
        iso[11..13].parse().ok()?
    } else {
        0
    };
    if hour + 9 >= 24 {
        day += 1;
        if day > days_in_month(year, month) {
            day = 1;
            month += 1;
            if month > 12 {
                month = 1;
                year += 1;
            }
        }
    }
    Some((year, month, day))
}

fn days_in_month(year: i32, month: u32) -> u32 {
    let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        _ => 28,
    }
}

/// 最新の取込日(JST)。"YYYY/M/D" で返す。有効な日時が無ければ None
pub(crate) fn latest_import_label<'a>(timestamps: impl Iterator<Item = &'a str>) -> Option<String> {
    timestamps
        .filter_map(jst_date)
        .max()
        .map(|(y, m, d)| format!("{y}/{m}/{d}"))
}

fn month_of(settlement_date: &str) -> Option<u32> {
    date_parts(settlement_date).map(|(_, month, _)| month)
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

pub(crate) fn sum_month_range(totals: &[Decimal; 12], from: u32, to: u32) -> Decimal {
    let from = from.clamp(1, 12);
    let to = to.clamp(1, 12);
    if from > to {
        return Decimal::ZERO;
    }
    totals[(from - 1) as usize..to as usize]
        .iter()
        .copied()
        .sum()
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

/// 今年の配当金カードの表示モデル。`current_month` は 1..=12。
pub(crate) struct DividendCardModel {
    /// 今年の税引後受取額
    pub(crate) received: Decimal,
    /// 年間予想配当(税引前)を実効税率で税引後に換算した予想
    pub(crate) forecast_net: Option<f64>,
    /// 予想に対する受取の割合(0..1 超過あり)
    pub(crate) progress: Option<f64>,
    /// 残り = 予想 - 受取
    pub(crate) remaining: Option<f64>,
    /// 前年の残り期間(今月より後の月)の入金。前年の取得に失敗したときは None
    pub(crate) previous_remaining: Option<Decimal>,
    /// 前年同期(1月〜今月)の入金との差 = 受取 - 前年同期。前年の取得に失敗したときは None
    pub(crate) yoy_diff: Option<Decimal>,
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
    current_month: u32,
) -> DividendCardModel {
    // 前年の取得に失敗(None)したときは前年系列を空に倒し、前年比較の数値は出さない。
    // 失敗由来の 0 を実績として表示しないため
    let previous_known = previous_rows.is_some();
    let previous_rows = previous_rows.unwrap_or(&[]);
    let received: Decimal = current_rows.iter().map(|row| row.net_amount_received).sum();
    let rate = effective_tax_rate(current_rows, previous_rows).unwrap_or(TAX_RATE);
    let forecast_net = forecast_gross.map(|gross| gross * (1.0 - dec_to_f64(&rate)));
    let received_f64 = dec_to_f64(&received);
    let progress = forecast_net
        .filter(|forecast| *forecast > 0.0)
        .map(|forecast| received_f64 / forecast);
    let remaining = forecast_net.map(|forecast| forecast - received_f64);
    let previous_totals = monthly_net_totals(previous_rows);
    let current_month = current_month.clamp(1, 12);
    let previous_ytd = sum_month_range(&previous_totals, 1, current_month);
    let previous_remaining = if current_month >= 12 {
        Decimal::ZERO
    } else {
        sum_month_range(&previous_totals, current_month + 1, 12)
    };
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
        remaining,
        previous_remaining: previous_known.then_some(previous_remaining),
        yoy_diff: previous_known.then_some(received - previous_ytd),
        monthly,
    }
}

/// 「残り」欄の表示文。受取が予想を超えたとき負の残りではなく
/// 「予想を N 上回り」に書き分ける
pub(crate) fn remaining_label(remaining: Option<f64>) -> String {
    match remaining {
        Some(value) if value < 0.0 => {
            format!("予想を {} 上回り", format_valuation_amount(Some(-value)))
        }
        Some(value) => format!("残り {}", format_valuation_amount(Some(value))),
        None => "残り —".to_string(),
    }
}

/// 月別グラフの棒の高さ(%)。0 より大きい値は潰れないよう最小 3% にする
pub(crate) fn bar_height(value: Decimal, max: Decimal) -> f64 {
    if value.is_zero() || max.is_zero() {
        return 0.0;
    }
    (dec_to_f64(&value) / dec_to_f64(&max) * 100.0).max(3.0)
}

/// 内訳チャートの1区分
pub(crate) struct CompositionSlice {
    pub(crate) label: String,
    pub(crate) amount: f64,
}

/// 商品別の資産内訳(時価ベース)。0 以下の区分は落とし、大きい順に並べる
pub(crate) fn product_slices(stock_market: f64, fund_market: f64) -> Vec<CompositionSlice> {
    let mut slices = vec![
        CompositionSlice {
            label: "国内株式".to_string(),
            amount: stock_market,
        },
        CompositionSlice {
            label: "投資信託".to_string(),
            amount: fund_market,
        },
    ];
    slices.retain(|slice| slice.amount > 0.0);
    slices.sort_by(|a, b| b.amount.total_cmp(&a.amount));
    slices
}

/// 口座別の資産内訳(時価ベース)。保有株式の行は口座を持たないため、
/// 口座がわかる投資信託だけで構成する
pub(crate) fn account_slices(funds: &[FundBalance]) -> Vec<CompositionSlice> {
    let mut by_account: Vec<(String, f64)> = Vec::new();
    for fund in funds {
        let account = fund.account.to_string();
        let amount = dec_to_f64(&fund.market_value);
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
        let valuation = holding_valuation(&ValuationInput {
            current_price: dec_to_f64(&row.current_price),
            shares: dec_to_f64(&row.shares),
            total_purchase_amount: dec_to_f64(&row.total_purchase_amount),
        });
        let profit_loss = valuation.profit_loss.amount?;
        let name = if row.security_name.is_empty() {
            row.security_code.to_string()
        } else {
            normalize_display_name(&row.security_name)
        };
        Some(HoldingPnl {
            code: row.security_code.to_string(),
            name,
            href: format!("/assetbalance/holdings/{}", row.id),
            profit_loss,
            rate: valuation.profit_loss.rate,
        })
    });
    let fund_rows = funds.iter().map(|row| HoldingPnl {
        code: String::new(),
        name: normalize_display_name(&row.fund_name),
        href: format!("/assetbalance/funds/{}", row.id),
        profit_loss: dec_to_f64(&row.profit_loss),
        rate: row.profit_loss_rate.as_ref().map(dec_to_f64),
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
