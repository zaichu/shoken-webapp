use super::*;

#[test]
fn chip_variant_classes() {
    let cases: [(ChipVariant, &str, &str); 3] = [
        (
            ChipVariant::Filter,
            "rounded border border-ink bg-ink px-3 py-1.5 text-sm font-bold text-text-inverse",
            "rounded border border-border-strong bg-surface px-3 py-1.5 text-sm text-text-soft",
        ),
        (
            ChipVariant::Segment,
            "flex-1 rounded bg-ink px-2 py-1 text-xs font-semibold text-text-inverse",
            "flex-1 rounded bg-surface-raised px-2 py-1 text-xs font-semibold text-text-muted",
        ),
        (ChipVariant::Pill, "filter-chip", "filter-chip"),
    ];
    for (variant, selected, unselected) in cases {
        assert_eq!(variant.selected_class(), selected, "{variant:?} selected");
        assert_eq!(
            variant.unselected_class(),
            unselected,
            "{variant:?} unselected"
        );
    }
}

#[test]
fn chip_segment_disabled_class() {
    assert_eq!(
        ChipVariant::Segment.disabled_class(),
        "flex-1 cursor-not-allowed rounded bg-surface-raised px-2 py-1 text-xs font-semibold text-text-faint"
    );
    for variant in [ChipVariant::Filter, ChipVariant::Pill] {
        assert_eq!(
            variant.disabled_class(),
            variant.unselected_class(),
            "{variant:?} disabled"
        );
    }
}
