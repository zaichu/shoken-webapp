use super::*;

#[test]
fn button_classes_preserves_variant_and_extra_input() {
    for variant in [
        ButtonVariant::Primary(ButtonSize::Md),
        ButtonVariant::Secondary(ButtonSize::Xs),
        ButtonVariant::DangerSolid(ButtonSize::Fill),
        ButtonVariant::Quiet,
        ButtonVariant::CopyName,
    ] {
        let base = button_classes(variant, "");
        assert_eq!(base, variant.classes());
        assert_eq!(
            button_classes(variant, "custom-a custom-b"),
            format!("{base} custom-a custom-b")
        );
    }
}
