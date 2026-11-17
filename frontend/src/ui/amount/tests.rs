use super::{AmountColor, AmountKind, amount_color, amount_negative_attr};

#[test]
fn signed_values_use_positive_negative_or_neutral_color() {
    for kind in [AmountKind::Plain, AmountKind::Delta] {
        assert_eq!(amount_color(kind, false, true, "¥1"), AmountColor::Positive);
        assert_eq!(
            amount_color(kind, true, false, "-¥1"),
            AmountColor::Negative
        );
        assert_eq!(amount_color(kind, false, false, "¥0"), AmountColor::Neutral);
        assert_eq!(
            amount_color(kind, false, false, "+0.0%"),
            AmountColor::Neutral
        );
        assert_eq!(
            amount_color(kind, false, false, "-0.0%"),
            AmountColor::Neutral
        );
        assert_eq!(
            amount_color(kind, false, false, "+1.0%"),
            AmountColor::Positive
        );
        assert_eq!(
            amount_color(kind, false, false, "-1.0%"),
            AmountColor::Negative
        );
    }
    assert_eq!(amount_negative_attr(AmountColor::Positive), None);
    assert_eq!(amount_negative_attr(AmountColor::Negative), Some("true"));
    assert_eq!(amount_negative_attr(AmountColor::Neutral), None);
}

#[test]
fn unsigned_zero_remains_neutral() {
    assert_eq!(
        amount_color(AmountKind::Plain, false, false, "¥0"),
        AmountColor::Neutral
    );
}

#[test]
fn currency_and_rates_follow_the_displayed_value() {
    for text in ["¥1,000", "1.0千円", "1.0万円", "1.0%", "(+1.0%)", "+¥1,000"] {
        for kind in [AmountKind::Plain, AmountKind::Delta] {
            assert_eq!(
                amount_color(kind, false, false, text),
                AmountColor::Positive,
                "{text}"
            );
            assert_eq!(
                amount_color(kind, true, false, text),
                AmountColor::Positive,
                "{text}"
            );
        }
    }
    for text in ["-¥1,000", "-1.0千円", "-1.0万円", "-1.0%", "(-1.0%)"] {
        assert_eq!(
            amount_color(AmountKind::Plain, false, true, text),
            AmountColor::Negative,
            "{text}"
        );
    }
}

#[test]
fn rounded_zero_and_missing_values_ignore_sign_hints() {
    for text in ["¥0", "-¥0", "0.0万円", "+0.0%", "-0.0%", "(-0.0%)", "—"] {
        for (negative, positive) in [(false, false), (true, false), (false, true)] {
            assert_eq!(
                amount_color(AmountKind::Delta, negative, positive, text),
                AmountColor::Neutral,
                "{text}"
            );
        }
    }
}

#[test]
fn quantities_dates_and_labels_remain_neutral() {
    for text in ["100株", "10,000口", "2026/11/18", "特定", "100"] {
        assert_eq!(
            amount_color(AmountKind::Plain, false, false, text),
            AmountColor::Neutral,
            "{text}"
        );
    }
}
