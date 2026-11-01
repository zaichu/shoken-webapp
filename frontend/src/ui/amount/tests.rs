use super::{AmountKind, amount_classes, amount_negative_attr};

#[test]
fn delta_positive_uses_green_without_negative_attr() {
    let class = amount_classes(AmountKind::Delta, false, true, "");
    assert!(class.contains("tabular-nums"));
    assert!(class.contains("whitespace-nowrap"));
    assert!(class.contains("text-positive"));
    assert!(!class.contains("text-text-soft"));
    assert_eq!(amount_negative_attr(AmountKind::Delta, false), None);
}

#[test]
fn delta_negative_uses_data_negative_without_green() {
    let class = amount_classes(AmountKind::Delta, true, false, "");
    assert!(class.contains("tabular-nums"));
    assert!(class.contains("whitespace-nowrap"));
    assert!(!class.contains("text-positive"));
    assert_eq!(amount_negative_attr(AmountKind::Delta, true), Some("true"));
}

#[test]
fn delta_zero_has_no_sign_color_and_stays_gray() {
    let class = amount_classes(AmountKind::Delta, false, false, "");
    assert!(class.contains("text-text-soft"));
    assert!(!class.contains("text-positive"));
    assert_eq!(amount_negative_attr(AmountKind::Delta, false), None);
}

#[test]
fn plain_amount_never_colors_green_or_negative() {
    for (negative, positive) in [(false, true), (true, false), (false, false)] {
        let class = amount_classes(AmountKind::Plain, negative, positive, "");
        assert!(!class.contains("text-positive"), "plain must not be green");
        assert!(
            !class.contains("text-text-soft"),
            "plain zero keeps neutral color"
        );
        assert_eq!(amount_negative_attr(AmountKind::Plain, negative), None);
    }
}

#[test]
fn unit_stays_in_same_nowrap_chunk() {
    let class = amount_classes(AmountKind::Plain, false, false, "");
    assert!(class.contains("whitespace-nowrap"));
}
