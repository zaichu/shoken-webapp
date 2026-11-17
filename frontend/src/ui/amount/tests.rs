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
