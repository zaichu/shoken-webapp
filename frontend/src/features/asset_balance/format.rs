use crate::ui::format::{
    f64_to_decimal, format_abs_number, format_currency_beyond_decimal, format_fixed_percent,
    to_fixed,
};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::{Decimal, RoundingStrategy};
use shared::format::format_currency as format_currency_decimal;

pub(crate) fn dec_to_f64(value: &Decimal) -> f64 {
    value.to_f64().unwrap_or(0.0)
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
        ("text-negative", "ml-1 text-xs tabular-nums text-negative")
    } else if is_positive_valuation(amount) {
        ("text-positive", "ml-1 text-xs tabular-nums text-positive")
    } else {
        ("text-text-soft", "ml-1 text-xs tabular-nums text-text-soft")
    };
    (text, rate_text, text_class, badge_class)
}

pub(crate) fn is_negative_valuation(amount: Option<f64>) -> bool {
    amount.is_some_and(|value| round_to_yen(value) < 0.0)
}

pub(crate) fn is_positive_valuation(amount: Option<f64>) -> bool {
    amount.is_some_and(|value| round_to_yen(value) > 0.0)
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
