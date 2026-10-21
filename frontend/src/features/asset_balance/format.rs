use crate::features::asset_balance::model::{format_abs_number, to_fixed};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::{Decimal, RoundingStrategy};
use shared::format::{
    format_currency as format_currency_decimal, format_currency_with_options,
    format_percentage_value as format_percentage_value_decimal,
};

pub(crate) fn dec_to_f64(value: &Decimal) -> f64 {
    value.to_f64().unwrap_or(0.0)
}

fn f64_to_decimal(value: f64) -> Option<Decimal> {
    Decimal::from_str_exact(&value.to_string())
        .ok()
        .map(|value| value.normalize())
}

// Decimal に収まらない合計でも、f64 のまま同じ書式で出す
fn format_currency_beyond_decimal(value: f64) -> String {
    match format_abs_number(to_fixed(value, 0)) {
        Some(body) if value < 0.0 => format!("-¥{body}"),
        Some(body) => format!("¥{body}"),
        None => "—".to_string(),
    }
}

// 表示単位(円/千円/万円)で割って1桁丸める。Decimal に収まらない値用
fn format_amount_beyond_decimal(value: f64) -> String {
    match crate::settings::current_amount_unit() {
        crate::settings::AmountUnit::Yen => format_currency_beyond_decimal(value),
        unit => {
            let scaled = value / unit.divisor_f64();
            match format_abs_number(to_fixed(scaled, 1)) {
                Some(body) if scaled < 0.0 => format!("-{body}{}", unit.label()),
                Some(body) => format!("{body}{}", unit.label()),
                None => "—".to_string(),
            }
        }
    }
}

/// 設定の表示単位(円/千円/万円)を適用する金額。集計(合計欄・サマリー)専用で、
/// 明細の行には format_row_valuation_amount を使う
pub(crate) fn format_amount(value: f64) -> String {
    f64_to_decimal(value).map_or_else(
        || format_amount_beyond_decimal(value),
        crate::settings::format_amount,
    )
}

pub(crate) fn format_currency(value: f64) -> String {
    f64_to_decimal(value).map_or_else(
        || format_currency_beyond_decimal(value),
        format_currency_decimal,
    )
}

/// 1株・1口あたりの金額(取得単価・現在値・基準価額・1株配当)の小数桁の上限
const UNIT_PRICE_DECIMALS_MAX: u32 = 2;

/// 列の中の値の小数桁(最大2桁)。同じ列の中の桁をそろえるための値で、
/// 整数円の値しかない列は0桁になる(不要な .00 を付けない)
pub(crate) fn unit_price_decimals(values: impl Iterator<Item = f64>) -> u32 {
    values
        .filter_map(|value| decimal_places(&format_currency(value)))
        .max()
        .unwrap_or(0)
        .min(UNIT_PRICE_DECIMALS_MAX)
}

fn decimal_places(text: &str) -> Option<u32> {
    let (_, fraction) = text.split_once('.')?;
    Some(fraction.len() as u32)
}

/// 1株・1口あたりの金額(取得単価・現在値・基準価額・1株配当)。
/// 小数点以下は列ごとに決めた桁数(decimals)で固定する
pub(crate) fn format_unit_price(value: f64, decimals: u32) -> String {
    f64_to_decimal(value).map_or_else(
        || format_unit_price_beyond_decimal(value, decimals),
        |value| format_currency_with_options(value, "¥", decimals, decimals),
    )
}

// Decimal に収まらない値でも小数桁をそろえて出す
fn format_unit_price_beyond_decimal(value: f64, decimals: u32) -> String {
    let rounded = to_fixed(value, decimals);
    let Some(body) = format_abs_number(rounded) else {
        return "—".to_string();
    };
    let (integer, fraction) = body
        .split_once('.')
        .map_or((body.as_str(), ""), |(i, f)| (i, f));
    let fraction = format!("{fraction:0<width$}", width = decimals as usize);
    if value < 0.0 {
        format!("-¥{integer}.{fraction}")
    } else {
        format!("¥{integer}.{fraction}")
    }
}

pub(crate) fn format_fixed_percent(value: f64, decimals: u32) -> String {
    if !value.is_finite() {
        return "—".to_string();
    }
    f64_to_decimal(value).map_or_else(
        || {
            format!(
                "{:.prec$}%",
                to_fixed(value, decimals),
                prec = decimals as usize
            )
        },
        |value| format_percentage_value_decimal(value, decimals),
    )
}

pub(crate) fn format_percentage_value(value: f64) -> String {
    format_fixed_percent(value, 2)
}

fn round_to_yen(value: f64) -> f64 {
    f64_to_decimal(value).map_or_else(
        || to_fixed(value, 0),
        |value| {
            value
                .round_dp_with_strategy(0, RoundingStrategy::MidpointAwayFromZero)
                .to_f64()
                .unwrap_or(0.0)
        },
    )
}

/// 損益セル共通の表示部品(金額・率・文字色・バッジ)
pub(crate) fn pnl_cells(
    amount: Option<f64>,
    rate: Option<f64>,
) -> (String, String, &'static str, &'static str) {
    let text = format_signed_row_valuation_amount(amount);
    let rate_text = format_signed_valuation_rate(rate, 1);
    let (text_class, badge_class) = if is_negative_valuation(amount) {
        ("text-negative", "asset-badge asset-badge-loss")
    } else if is_positive_valuation(amount) {
        ("text-positive", "asset-badge asset-badge-gain")
    } else {
        ("text-text-soft", "asset-badge")
    };
    (text, rate_text, text_class, badge_class)
}

pub(crate) fn is_negative_valuation(amount: Option<f64>) -> bool {
    amount.is_some_and(|value| round_to_yen(value) < 0.0)
}

pub(crate) fn is_positive_valuation(amount: Option<f64>) -> bool {
    amount.is_some_and(|value| round_to_yen(value) > 0.0)
}

/// 現在値の基準日("YYYY-MM-DD")を "MM/DD" にする。想定外の形式はそのまま返す
pub(crate) fn format_price_date(as_of: &str) -> String {
    match as_of.split_once('-') {
        Some((year, month_day)) if year.len() == 4 && !month_day.is_empty() => {
            month_day.replace('-', "/")
        }
        _ => as_of.to_string(),
    }
}

/// 基準価額の自動更新の時刻("YYYY-MM-DD HH:MM")を "MM/DD HH:MM" にする。
/// 想定外の形式はそのまま返す
pub(crate) fn format_nav_updated_at(updated_at: &str) -> String {
    let Some((date, time)) = updated_at.split_once(' ') else {
        return updated_at.to_string();
    };
    if date.len() != 10 || time.len() < 5 {
        return updated_at.to_string();
    }
    format!("{} {}", format_price_date(date), &time[..5])
}

/// 日付群の代表(最も多い日付。同数なら新しい方)。日付の無いものは比較に含めない。
/// 見出しはこの日付を出し、表の行は違うときだけ日付を足すので、両者で同じ値を使う
pub(crate) fn price_date_representative<'a>(
    dates: impl Iterator<Item = &'a str>,
) -> Option<&'a str> {
    let mut counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for date in dates.filter(|date| !date.is_empty()) {
        *counts.entry(date).or_default() += 1;
    }
    counts
        .into_iter()
        .max_by(|(date_a, count_a), (date_b, count_b)| {
            count_a.cmp(count_b).then_with(|| date_a.cmp(date_b))
        })
        .map(|(date, _)| date)
}

/// 行の基準日が代表と違うか。日付の無い行は false
pub(crate) fn price_date_is_outlier(date: Option<&str>, representative: Option<&str>) -> bool {
    date.filter(|date| !date.is_empty())
        .is_some_and(|date| Some(date) != representative)
}

/// 各行の基準日が代表と違うかをまとめて返す。`representative` は
/// `price_date_representative` を未絞り込みの行に掛けた値を渡す
/// (見出しは絞り込み前の代表を出すので、行側もそれと同じ基準で比較する)
pub(crate) fn price_date_outliers(
    dates: &[Option<String>],
    representative: Option<&str>,
) -> Vec<bool> {
    dates
        .iter()
        .map(|date| price_date_is_outlier(date.as_deref(), representative))
        .collect()
}

fn format_valuation_with(
    amount: Option<f64>,
    format_yen: fn(Decimal) -> String,
    format_beyond: fn(f64) -> String,
) -> String {
    match amount {
        Some(value) if !value.is_finite() => "—".to_string(),
        Some(value) => f64_to_decimal(value)
            .map(|value| {
                format_yen(
                    value
                        .round_dp_with_strategy(0, RoundingStrategy::MidpointAwayFromZero)
                        .normalize(),
                )
            })
            .unwrap_or_else(|| format_beyond(value)),
        None => "—".to_string(),
    }
}

pub(crate) fn format_valuation_amount(amount: Option<f64>) -> String {
    format_valuation_with(
        amount,
        crate::settings::format_amount,
        format_amount_beyond_decimal,
    )
}

/// 明細の行用。表示単位を適用せず1円単位で出す
pub(crate) fn format_row_valuation_amount(amount: Option<f64>) -> String {
    format_valuation_with(
        amount,
        format_currency_decimal,
        format_currency_beyond_decimal,
    )
}

pub(crate) fn format_valuation_rate(rate: Option<f64>, decimals: u32) -> String {
    match rate {
        Some(value) if value.is_nan() => "—".to_string(),
        Some(value) => format_fixed_percent(value, decimals),
        None => "—".to_string(),
    }
}

fn format_signed_with(amount: Option<f64>, format: fn(Option<f64>) -> String) -> String {
    match amount {
        Some(value) if value.is_finite() && round_to_yen(value) > 0.0 => {
            format!("+{}", format(amount))
        }
        _ => format(amount),
    }
}

/// 損益用の金額。プラスにも「+」を付け、色だけに頼らず符号で正負を示す。
pub(crate) fn format_signed_valuation_amount(amount: Option<f64>) -> String {
    format_signed_with(amount, format_valuation_amount)
}

/// 損益用の金額(明細の行用)。プラスにも「+」を付け、色だけに頼らず符号で正負を示す。
pub(crate) fn format_signed_row_valuation_amount(amount: Option<f64>) -> String {
    format_signed_with(amount, format_row_valuation_amount)
}

/// 損益用の率。プラスにも「+」を付け、色だけに頼らず符号で正負を示す。
pub(crate) fn format_signed_valuation_rate(rate: Option<f64>, decimals: u32) -> String {
    match rate {
        Some(value) if value > 0.0 => format!("+{}", format_valuation_rate(rate, decimals)),
        _ => format_valuation_rate(rate, decimals),
    }
}
