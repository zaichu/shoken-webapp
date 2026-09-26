use rust_decimal::{Decimal, RoundingStrategy};

#[must_use]
pub fn format_number(value: Decimal, maximum_fraction_digits: u32) -> String {
    format_number_with_options(value, 0, maximum_fraction_digits, true)
}

#[must_use]
pub fn format_number_with_options(
    value: Decimal,
    minimum_fraction_digits: u32,
    maximum_fraction_digits: u32,
    use_grouping: bool,
) -> String {
    let rounded = value.round_dp_with_strategy(
        maximum_fraction_digits,
        RoundingStrategy::MidpointAwayFromZero,
    );
    let raw = rounded.normalize().to_string();
    let (sign, unsigned) = raw
        .strip_prefix('-')
        .map_or(("", raw.as_str()), |v| ("-", v));
    let (integer, fraction) = unsigned
        .split_once('.')
        .map_or((unsigned, None), |(i, f)| (i, Some(f)));
    let grouped = if use_grouping {
        let reversed: String = integer
            .chars()
            .rev()
            .enumerate()
            .flat_map(|(index, character)| {
                if (index + 1) % 3 == 0 && index + 1 < integer.len() {
                    vec![character, ',']
                } else {
                    vec![character]
                }
            })
            .collect();
        reversed.chars().rev().collect()
    } else {
        integer.to_string()
    };
    let mut fraction = fraction.unwrap_or_default().to_string();
    while fraction.len() < minimum_fraction_digits as usize {
        fraction.push('0');
    }
    if fraction.is_empty() {
        format!("{sign}{grouped}")
    } else {
        format!("{sign}{grouped}.{fraction}")
    }
}

#[must_use]
pub fn format_currency(value: Decimal) -> String {
    format_currency_with_options(value, "¥", 0, 15)
}

#[must_use]
pub fn format_currency_with_options(
    value: Decimal,
    currency: &str,
    minimum_fraction_digits: u32,
    maximum_fraction_digits: u32,
) -> String {
    let formatted = format_number_with_options(
        value.abs(),
        minimum_fraction_digits,
        maximum_fraction_digits,
        true,
    );
    if value.is_sign_negative() {
        format!("{currency} -{formatted}")
    } else {
        format!("{currency} {formatted}")
    }
}

#[must_use]
pub fn format_percentage_value(value: Decimal, decimals: u32) -> String {
    format!(
        "{}%",
        format_number_with_options(value, decimals, decimals, false)
    )
}
