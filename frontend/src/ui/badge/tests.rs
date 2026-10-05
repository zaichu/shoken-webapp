use super::*;

#[test]
fn badge_variant_classes_are_distinguishable() {
    let variants = [
        BadgeVariant::Accent,
        BadgeVariant::AccentFlat,
        BadgeVariant::Info,
        BadgeVariant::Positive,
    ];
    let classes: Vec<&str> = variants.iter().map(|variant| variant.class()).collect();
    for (i, class) in classes.iter().enumerate() {
        assert!(!class.is_empty(), "{:?}", variants[i]);
        assert!(!classes[..i].contains(class), "{:?}", variants[i]);
    }
}
