use crate::asset_balance_domain::{format_abs_number, to_fixed};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;

pub(crate) fn dec_to_f64(value: &Decimal) -> f64 {
    value.to_f64().unwrap_or(0.0)
}

pub(crate) fn format_currency(value: f64) -> String {
    match format_abs_number(to_fixed(value, 15)) {
        None => "-".to_string(),
        Some(body) if value < 0.0 => format!("¥ -{body}"),
        Some(body) => format!("¥ {body}"),
    }
}

pub(crate) fn format_fixed_percent(value: f64, decimals: u32) -> String {
    if value.is_nan() {
        return "-".to_string();
    }
    format!(
        "{:.prec$}%",
        to_fixed(value, decimals),
        prec = decimals as usize
    )
}

pub(crate) fn format_percentage_value(value: f64) -> String {
    format_fixed_percent(value, 2)
}

pub(crate) fn format_valuation_amount(amount: Option<f64>) -> String {
    match amount {
        Some(value) if value.is_nan() => "—".to_string(),
        Some(value) if value >= 0.0 => format!("+{}", format_currency(value)),
        Some(value) => format_currency(value),
        None => "—".to_string(),
    }
}

pub(crate) fn format_valuation_rate(rate: Option<f64>, decimals: u32) -> String {
    match rate {
        Some(value) if value.is_nan() => "—".to_string(),
        Some(value) => {
            let sign = if value >= 0.0 { "+" } else { "" };
            format!("{sign}{}", format_fixed_percent(value, decimals))
        }
        None => "—".to_string(),
    }
}
