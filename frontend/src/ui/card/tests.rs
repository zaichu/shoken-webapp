use super::*;

#[test]
fn card_variant_classes() {
    let cases: [(CardVariant, &str); 22] = [
        (CardVariant::Panel, "panel-card"),
        (CardVariant::Login, "login-card"),
        (CardVariant::Table, "table-card"),
        (CardVariant::Collapsible, "collapsible-card"),
        (CardVariant::Feature, "feature-card"),
        (CardVariant::Rail, "rail-panel"),
        (
            CardVariant::Shell,
            "overflow-hidden rounded-xl border border-ink/10 bg-surface/95 shadow-elevation-2",
        ),
        (
            CardVariant::Summary,
            "rounded-xl border border-ink/10 bg-surface/95 px-5 py-5 shadow-summary-card",
        ),
        (
            CardVariant::Soft,
            "overflow-hidden rounded-xl border border-ink/10 bg-surface/90 shadow-card",
        ),
        (
            CardVariant::Item,
            "rounded-lg border border-border-strong bg-surface",
        ),
        (
            CardVariant::Group,
            "overflow-hidden rounded-lg border border-border-subtle",
        ),
        (
            CardVariant::Holding,
            "rounded-lg border border-ink/10 bg-surface shadow-sm",
        ),
        (
            CardVariant::Stat,
            "rounded-lg border border-ink/10 bg-surface px-4 py-4 shadow-sm",
        ),
        (
            CardVariant::StatSmall,
            "rounded-lg border border-border-subtle bg-surface px-3.5 py-3",
        ),
        (
            CardVariant::Sunken,
            "rounded-lg bg-surface-sunken px-4 py-3",
        ),
        (
            CardVariant::Strip,
            "overflow-hidden rounded-md bg-surface-sunken",
        ),
        (
            CardVariant::Hint,
            "rounded-xl border border-ink/10 bg-surface-sunken/80 p-4",
        ),
        (
            CardVariant::Dashed,
            "rounded-xl border border-dashed border-border-strong bg-surface-raised/70 px-4 py-4",
        ),
        (
            CardVariant::DashedCompact,
            "rounded-lg border border-dashed border-border-strong bg-surface-sunken p-3",
        ),
        (
            CardVariant::Step,
            "rounded-lg border border-ink/10 bg-surface/75 px-3 py-3",
        ),
        (
            CardVariant::Tile,
            "rounded-xl border border-ink/10 bg-surface px-4 py-4 shadow-sm",
        ),
        (
            CardVariant::GroupLabel,
            "flex min-h-11 items-center rounded-lg bg-surface-raised px-3 py-2",
        ),
    ];
    for (variant, expected) in cases {
        assert_eq!(variant.class(), expected, "{variant:?}");
    }
}

#[test]
fn section_tag_is_only_collapsible_and_summary() {
    let cases: [(CardVariant, bool); 22] = [
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
        (CardVariant::Hint, false),
        (CardVariant::Dashed, false),
        (CardVariant::DashedCompact, false),
        (CardVariant::Step, false),
        (CardVariant::Tile, false),
        (CardVariant::GroupLabel, false),
    ];
    for (variant, expected) in cases {
        assert_eq!(variant.section_tag(), expected, "{variant:?}");
    }
}

#[test]
fn section_header_classes() {
    let cases: [(SectionHeaderVariant, &str, &str); 3] = [
        (
            SectionHeaderVariant::Band,
            "summary-section-header",
            "text-sm font-black text-text-strong",
        ),
        (
            SectionHeaderVariant::Card,
            "flex flex-col gap-3 border-b border-ink/10 pb-4 sm:flex-row sm:items-end sm:justify-between",
            "text-sm font-black text-ink",
        ),
        (
            SectionHeaderVariant::Divider,
            "flex items-start justify-between gap-3 border-b border-ink/10 pb-2.5",
            "text-sm font-black text-ink",
        ),
    ];
    for (variant, frame, heading) in cases {
        assert_eq!(variant.frame(), frame, "{variant:?}.frame");
        assert_eq!(variant.heading(), heading, "{variant:?}.heading");
    }
}
