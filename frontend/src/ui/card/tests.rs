use super::*;

#[test]
fn section_tag_is_only_collapsible() {
    let cases: [(CardVariant, bool); 6] = [
        (CardVariant::Panel, false),
        (CardVariant::Collapsible, true),
        (CardVariant::Item, false),
        (CardVariant::Group, false),
        (CardVariant::Sunken, false),
        (CardVariant::GroupLabel, false),
    ];
    for (variant, expected) in cases {
        assert_eq!(variant.section_tag(), expected, "{variant:?}");
    }
}

#[test]
fn card_variant_classes_are_distinguishable() {
    let variants = [
        CardVariant::Panel,
        CardVariant::Collapsible,
        CardVariant::Item,
        CardVariant::Group,
        CardVariant::Sunken,
        CardVariant::GroupLabel,
    ];
    let classes: Vec<String> = variants.iter().map(|variant| variant.class()).collect();
    for (i, class) in classes.iter().enumerate() {
        assert!(!class.is_empty(), "{:?}", variants[i]);
        assert!(!classes[..i].contains(class), "{:?}", variants[i]);
    }
}
