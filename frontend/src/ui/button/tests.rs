use super::*;

#[test]
fn button_classes_preserves_variant_and_extra_input() {
    for variant in [
        ButtonVariant::Primary(ButtonSize::Md),
        ButtonVariant::Secondary(ButtonSize::Xs),
        ButtonVariant::DangerSolid(ButtonSize::Sm),
        ButtonVariant::CopyName,
        ButtonVariant::PageTab,
        ButtonVariant::Segmented,
        ButtonVariant::ConditionSegment,
        ButtonVariant::YearOption,
        ButtonVariant::MenuItem,
        ButtonVariant::MenuItemDanger,
        ButtonVariant::RowButton,
    ] {
        let base = button_classes(variant, "");
        assert_eq!(base, variant.classes());
        assert_eq!(
            button_classes(variant, "custom-a custom-b"),
            format!("{base} custom-a custom-b")
        );
    }
}

#[test]
fn segmented_variants_share_single_class() {
    assert_eq!(
        ButtonVariant::Segmented.classes(),
        ButtonVariant::ConditionSegment.classes()
    );
}

#[test]
fn action_buttons_are_three_kinds_with_enough_height() {
    for variant in [
        ButtonVariant::Primary(ButtonSize::Md),
        ButtonVariant::Secondary(ButtonSize::Md),
        ButtonVariant::DangerSolid(ButtonSize::Md),
    ] {
        assert!(
            variant.classes().contains("min-h-11"),
            "action buttons stay 44px or taller: {variant:?}"
        );
    }
    assert!(
        ButtonVariant::Primary(ButtonSize::Md)
            .classes()
            .contains("bg-brand")
    );
    assert!(
        ButtonVariant::Secondary(ButtonSize::Md)
            .classes()
            .contains("bg-surface")
    );
    let danger = ButtonVariant::DangerSolid(ButtonSize::Md).classes();
    assert!(danger.contains("bg-surface"));
    assert!(danger.contains("text-negative-vivid"));
}

#[test]
fn retry_is_a_blue_text_button() {
    let retry = ButtonVariant::Retry.classes();
    assert!(retry.contains("text-accent-text"));
    assert!(!retry.contains("bg-ink"));
}
