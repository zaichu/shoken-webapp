use crate::asset_balance_domain::{format_abs_number, to_fixed};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::{Decimal, RoundingStrategy};
use shared::format::{
    format_currency as format_currency_decimal,
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

fn format_currency_f64(value: f64, digits: u32) -> String {
    match format_abs_number(to_fixed(value, digits)) {
        None => "-".to_string(),
        Some(body) if value < 0.0 => format!("¥ -{body}"),
        Some(body) => format!("¥ {body}"),
    }
}

pub(crate) fn format_currency(value: f64) -> String {
    f64_to_decimal(value).map_or_else(|| format_currency_f64(value, 15), format_currency_decimal)
}

pub(crate) fn format_fixed_percent(value: f64, decimals: u32) -> String {
    if value.is_nan() {
        return "-".to_string();
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

pub(crate) fn is_negative_valuation(amount: Option<f64>) -> bool {
    amount.is_some_and(|value| round_to_yen(value) < 0.0)
}

pub(crate) fn format_valuation_amount(amount: Option<f64>) -> String {
    match amount {
        Some(value) if !value.is_finite() => "—".to_string(),
        Some(value) => f64_to_decimal(value)
            .map(|value| {
                format_currency_decimal(
                    value
                        .round_dp_with_strategy(0, RoundingStrategy::MidpointAwayFromZero)
                        .normalize(),
                )
            })
            .unwrap_or_else(|| format_currency_f64(to_fixed(value, 0), 0)),
        None => "—".to_string(),
    }
}

pub(crate) fn format_valuation_rate(rate: Option<f64>, decimals: u32) -> String {
    match rate {
        Some(value) if value.is_nan() => "—".to_string(),
        Some(value) => format_fixed_percent(value, decimals),
        None => "—".to_string(),
    }
}
