use super::*;

#[test]
fn section_tag_is_only_collapsible() {
    let cases: [(CardVariant, bool); 15] = [
        (CardVariant::Panel, false),
        (CardVariant::Login, false),
        (CardVariant::Table, false),
        (CardVariant::Collapsible, true),
        (CardVariant::Feature, false),
        (CardVariant::Shell, false),
        (CardVariant::Soft, false),
        (CardVariant::Item, false),
        (CardVariant::Group, false),
        (CardVariant::Holding, false),
        (CardVariant::Sunken, false),
        (CardVariant::Strip, false),
        (CardVariant::DashedCompact, false),
        (CardVariant::Tile, false),
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
        CardVariant::Login,
        CardVariant::Table,
        CardVariant::Collapsible,
        CardVariant::Feature,
        CardVariant::Shell,
        CardVariant::Soft,
        CardVariant::Item,
        CardVariant::Group,
        CardVariant::Holding,
        CardVariant::Sunken,
        CardVariant::Strip,
        CardVariant::DashedCompact,
        CardVariant::Tile,
        CardVariant::GroupLabel,
    ];
    let classes: Vec<String> = variants.iter().map(|variant| variant.class()).collect();
    for (i, class) in classes.iter().enumerate() {
        assert!(!class.is_empty(), "{:?}", variants[i]);
        assert!(!classes[..i].contains(class), "{:?}", variants[i]);
    }
}
