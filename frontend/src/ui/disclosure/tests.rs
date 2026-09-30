use super::*;

#[test]
fn disclosure_style_classes() {
    let cases: [(DisclosureStyle, &str); 6] = [
        (DisclosureStyle::Toolbar, "mobile-toolbar-trigger"),
        (DisclosureStyle::Rail, "rail-toggle"),
        (DisclosureStyle::GroupCard, "group-card-trigger"),
        (
            DisclosureStyle::HeaderFlat,
            "flex w-full items-start justify-between gap-3 border-b border-ink/10 pb-2.5 text-left",
        ),
        (
            DisclosureStyle::AssetCard,
            "block min-h-11 w-full px-3.5 py-4 text-left",
        ),
        (
            DisclosureStyle::SearchCard,
            "flex w-full min-w-0 items-center gap-2.5 text-left select-none cursor-pointer max-sm:min-h-11",
        ),
    ];
    for (style, expected) in cases {
        assert_eq!(style.class(), expected, "{style:?}");
    }
}
