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

pub(crate) fn format_currency(value: f64) -> String {
    f64_to_decimal(value).map_or_else(|| "-".to_string(), format_currency_decimal)
}

pub(crate) fn format_fixed_percent(value: f64, decimals: u32) -> String {
    f64_to_decimal(value)
        .map(|value| format_percentage_value_decimal(value, decimals))
        .unwrap_or_else(|| "-".to_string())
}

pub(crate) fn format_percentage_value(value: f64) -> String {
    format_fixed_percent(value, 2)
}

pub(crate) fn format_valuation_amount(amount: Option<f64>) -> String {
    match amount {
        Some(value) if value.is_nan() => "—".to_string(),
        Some(value) => f64_to_decimal(value)
            .map(|value| {
                format_currency_decimal(
                    value
                        .round_dp_with_strategy(0, RoundingStrategy::MidpointAwayFromZero)
                        .normalize(),
                )
            })
            .unwrap_or_else(|| "—".to_string()),
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
