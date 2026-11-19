use super::{AmountColor, AmountKind, amount_color, amount_negative_attr};
use crate::ui::labels;

#[test]
fn negative_display_is_negative_for_both_kinds() {
    for text in ["-¥1", "-¥1,000", "-1.0千円", "-1.0万円", "-1.0%", "(-1.0%)"] {
        for kind in [AmountKind::Plain, AmountKind::Delta] {
            for (negative, positive) in [(false, false), (true, false), (false, true)] {
                assert_eq!(
                    amount_color(kind, negative, positive, text),
                    AmountColor::Negative,
                    "{text:?} {kind:?}"
                );
            }
        }
    }
}

#[test]
fn positive_display_is_positive_only_for_delta_or_explicit_hint() {
    for text in [
        "¥1,000",
        "1.0千円",
        "1.0万円",
        "1.0%",
        "(+1.0%)",
        "+¥1,000",
        "+1.0%",
    ] {
        assert_eq!(
            amount_color(AmountKind::Delta, false, false, text),
            AmountColor::Positive,
            "{text}"
        );
        assert_eq!(
            amount_color(AmountKind::Plain, false, false, text),
            AmountColor::Neutral,
            "{text}"
        );
        assert_eq!(
            amount_color(AmountKind::Plain, false, true, text),
            AmountColor::Positive,
            "{text}"
        );
    }
}

#[test]
fn signed_values_use_positive_negative_or_neutral_color() {
    assert_eq!(
        amount_color(AmountKind::Delta, false, true, "¥1"),
        AmountColor::Positive
    );
    assert_eq!(
        amount_color(AmountKind::Delta, true, false, "-¥1"),
        AmountColor::Negative
    );
    assert_eq!(
        amount_color(AmountKind::Plain, false, false, "¥0"),
        AmountColor::Neutral
    );
    assert_eq!(
        amount_color(AmountKind::Delta, false, false, "+0.0%"),
        AmountColor::Neutral
    );
    assert_eq!(
        amount_color(AmountKind::Delta, false, false, "-0.0%"),
        AmountColor::Neutral
    );
    assert_eq!(
        amount_color(AmountKind::Delta, false, false, "+1.0%"),
        AmountColor::Positive
    );
    assert_eq!(
        amount_color(AmountKind::Plain, false, false, "-1.0%"),
        AmountColor::Negative
    );
    assert_eq!(amount_negative_attr(AmountColor::Positive), None);
    assert_eq!(amount_negative_attr(AmountColor::Negative), Some("true"));
    assert_eq!(amount_negative_attr(AmountColor::Neutral), None);
}

#[test]
fn unsigned_zero_and_missing_values_stay_neutral() {
    for text in ["¥0", "-¥0", "0.0万円", "+0.0%", "-0.0%", "(-0.0%)", "—"] {
        for kind in [AmountKind::Plain, AmountKind::Delta] {
            for (negative, positive) in [(false, false), (true, false), (false, true)] {
                assert_eq!(
                    amount_color(kind, negative, positive, text),
                    AmountColor::Neutral,
                    "{text}"
                );
            }
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

#[test]
fn tax_suffix_inside_amount_text_loses_sign_color() {
    // 税区分の接尾辞を金額の文字列に混ぜると正負の色が出ない。
    // 接尾辞は Amount の外の別の要素に出す(呼び出し側の決まり)
    let suffixed = format!("+¥2,867({})", labels::AFTER_TAX);
    assert_eq!(
        amount_color(AmountKind::Delta, false, true, &suffixed),
        AmountColor::Neutral,
        "{suffixed}"
    );
    assert_eq!(
        amount_color(AmountKind::Delta, false, true, "+¥2,867"),
        AmountColor::Positive
    );
    let suffixed = format!("-¥2,867({})", labels::AFTER_TAX);
    assert_eq!(
        amount_color(AmountKind::Delta, true, false, &suffixed),
        AmountColor::Neutral,
        "{suffixed}"
    );
    assert_eq!(
        amount_color(AmountKind::Delta, true, false, "-¥2,867"),
        AmountColor::Negative
    );
}
