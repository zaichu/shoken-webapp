use super::*;

#[test]
fn badge_variant_classes() {
    let cases: [(BadgeVariant, &str); 5] = [
        (BadgeVariant::Accent, "filter-badge"),
        (
            BadgeVariant::AccentFlat,
            "shrink-0 rounded bg-accent-softer px-1.5 py-0.5 text-xs font-bold text-accent-text",
        ),
        (
            BadgeVariant::Info,
            "inline-flex items-center rounded-md border border-info-border bg-info-soft px-3 py-1 text-sm font-bold text-info",
        ),
        (
            BadgeVariant::Positive,
            "inline-flex items-center rounded bg-positive-softer px-1.5 py-0.5 text-xs font-medium text-positive",
        ),
        (BadgeVariant::File, "file-chip"),
    ];
    for (variant, expected) in cases {
        assert_eq!(variant.class(), expected, "{variant:?}");
    }
}
