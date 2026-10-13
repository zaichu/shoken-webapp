use super::*;

#[test]
fn badge_variant_classes_are_distinguishable() {
    let variants = [
        BadgeVariant::Accent,
        BadgeVariant::AccentFlat,
        BadgeVariant::Positive,
    ];
    let classes: Vec<&str> = variants.iter().map(|variant| variant.class()).collect();
    for (i, class) in classes.iter().enumerate() {
        assert!(!class.is_empty(), "{:?}", variants[i]);
        assert!(!classes[..i].contains(class), "{:?}", variants[i]);
    }
}

#[test]
fn nisa_account_is_detected_from_label() {
    assert!(is_nisa_account("新NISA"));
    assert!(is_nisa_account("旧NISA"));
    assert!(!is_nisa_account("特定"));
    assert!(!is_nisa_account("一般"));
    assert!(!is_nisa_account("nisa"));
}
