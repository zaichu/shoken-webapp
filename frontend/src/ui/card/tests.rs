use super::*;

#[test]
fn section_tag_is_only_collapsible_and_summary() {
    let cases: [(CardVariant, bool); 19] = [
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
        (CardVariant::StatSmall, false),
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
