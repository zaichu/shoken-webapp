use super::*;

#[test]
fn section_tag_is_only_collapsible_and_summary() {
    let cases: [(CardVariant, bool); 18] = [
        (CardVariant::Panel, false),
        (CardVariant::Login, false),
        (CardVariant::Table, false),
        (CardVariant::Collapsible, true),
        (CardVariant::Feature, false),
        (CardVariant::Rail, false),
        (CardVariant::Shell, false),
        (CardVariant::Summary, true),
        (CardVariant::Soft, false),
        (CardVariant::Item, false),
        (CardVariant::Group, false),
        (CardVariant::Holding, false),
        (CardVariant::Stat, false),
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
        CardVariant::Rail,
        CardVariant::Shell,
        CardVariant::Summary,
        CardVariant::Soft,
        CardVariant::Item,
        CardVariant::Group,
        CardVariant::Holding,
        CardVariant::Stat,
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

#[test]
fn section_header_classes_distinguish_variants() {
    let band = SectionHeaderVariant::Band;
    let card = SectionHeaderVariant::Card;
    for class in [band.frame(), card.frame(), band.heading(), card.heading()] {
        assert!(!class.is_empty());
    }
    assert_ne!(band.frame(), card.frame());
    assert_ne!(band.heading(), card.heading());
}
