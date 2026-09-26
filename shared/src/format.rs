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
    let fraction = format!(
        "{:0<width$}",
        fraction.unwrap_or_default(),
        width = minimum_fraction_digits as usize
    );
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

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::strategy::Strategy;
    use rust_decimal_macros::dec;

    #[test]
    fn number_formatter_supports_fraction_and_grouping_options() {
        assert_eq!(format_number(dec!(12345), 2), "12,345");
        assert_eq!(format_number(dec!(-12345), 2), "-12,345");
        assert_eq!(format_number(dec!(123.456), 2), "123.46");
        assert_eq!(format_number(dec!(1234.567), 1), "1,234.6");
        assert_eq!(format_number(dec!(0), 0), "0");
        assert_eq!(format_number_with_options(dec!(123), 2, 2, true), "123.00");
        assert_eq!(
            format_number_with_options(dec!(12345), 0, 2, false),
            "12345"
        );
        assert_eq!(
            format_number_with_options(dec!(12345.6), 0, 0, true),
            "12,346"
        );
    }

    #[test]
    fn currency_formatter_places_sign_after_symbol() {
        assert_eq!(format_currency(dec!(12345)), "¥ 12,345");
        assert_eq!(format_currency(dec!(-12345)), "¥ -12,345");
        assert_eq!(format_currency(dec!(0)), "¥ 0");
        assert_eq!(
            format_currency_with_options(dec!(12345), "$", 0, 15),
            "$ 12,345"
        );
        assert_eq!(
            format_currency_with_options(dec!(123.456), "¥", 0, 2),
            "¥ 123.46"
        );
        assert_eq!(
            format_currency_with_options(dec!(-1.5), "¥", 0, 15),
            "¥ -1.5"
        );
    }

    #[test]
    fn percentage_formatter_pads_to_fixed_decimals() {
        assert_eq!(format_percentage_value(dec!(25.5), 2), "25.50%");
        assert_eq!(format_percentage_value(dec!(-33.333), 1), "-33.3%");
        assert_eq!(format_percentage_value(dec!(1234.5), 1), "1234.5%");
        assert_eq!(format_percentage_value(dec!(0), 2), "0.00%");
    }

    fn arb_decimal() -> impl proptest::strategy::Strategy<Value = Decimal> {
        (-9_999_999_999_999i64..9_999_999_999_999i64).prop_map(|mantissa| Decimal::new(mantissa, 3))
    }

    proptest::proptest! {
        #[test]
        fn prop_format_number_with_options_invariants(
            mantissa in -9_999_999_999_999i64..9_999_999_999_999i64,
            scale in 0u32..=4u32,
            min_max in (0u32..=6u32, 0u32..=6u32),
            use_grouping in proptest::bool::ANY,
        ) {
            let value = Decimal::new(mantissa, scale);
            let min = min_max.0.min(min_max.1);
            let max = min_max.0.max(min_max.1);
            let output = format_number_with_options(value, min, max, use_grouping);

            let unsigned = output.strip_prefix('-').unwrap_or(&output);
            let (integer, fraction) = unsigned
                .split_once('.')
                .map_or((unsigned, ""), |(i, f)| (i, f));
            proptest::prop_assert!(fraction.len() <= max as usize);
            proptest::prop_assert!(fraction.len() >= min as usize);

            let digits: String = integer.chars().filter(|c| *c != ',').collect();
            let comma_positions: Vec<usize> = integer
                .chars()
                .enumerate()
                .filter(|(_, c)| *c == ',')
                .map(|(i, _)| i)
                .collect();
            for pos in comma_positions {
                proptest::prop_assert_eq!((integer.len() - pos) % 4, 0, "output={}", output);
            }

            let reparsed: Decimal = format!(
                "{}{}{}{}",
                if output.starts_with('-') { "-" } else { "" },
                digits,
                if fraction.is_empty() { "" } else { "." },
                fraction
            )
            .parse()
            .unwrap();
            proptest::prop_assert_eq!(
                reparsed,
                value.round_dp_with_strategy(max, RoundingStrategy::MidpointAwayFromZero),
                "output={}",
                output
            );
        }

        #[test]
        fn prop_format_currency_sign_convention(value in arb_decimal()) {
            let output = format_currency(value);
            if value.is_sign_negative() {
                proptest::prop_assert!(output.starts_with("¥ -"), "output={output}");
                proptest::prop_assert!(!output.contains("--"));
            } else {
                proptest::prop_assert!(output.starts_with("¥ "), "output={output}");
                proptest::prop_assert!(!output.contains('-'));
            }
        }
    }
}
