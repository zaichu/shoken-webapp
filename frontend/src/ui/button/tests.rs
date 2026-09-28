use super::*;

const EXPECTED_BASE: &str = "inline-flex items-center justify-center rounded-md font-bold transition-[background-color,border-color,color,box-shadow,transform] focus:outline-none focus:ring-2 focus:ring-accent-bright/50 focus:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-60";

#[test]
fn sized_variants_compose_base_variant_and_size() {
    let cases: [(ButtonVariant, &str); 5] = [
        (
            ButtonVariant::Primary(ButtonSize::Md),
            "border border-ink bg-ink text-text-inverse shadow-edge-lit hover:bg-ink-hover active:bg-ink px-4 py-2 text-sm max-sm:min-h-11",
        ),
        (
            ButtonVariant::Secondary(ButtonSize::Xs),
            "border border-border-strong bg-surface text-text hover:border-border-bold hover:bg-surface-sunken px-2 py-1 text-xs max-sm:min-h-11",
        ),
        (
            ButtonVariant::SecondarySoft(ButtonSize::Xs),
            "border border-border-strong bg-surface text-text-soft hover:border-border-bold hover:bg-surface-sunken px-2 py-1 text-xs max-sm:min-h-11",
        ),
        (
            ButtonVariant::Danger(ButtonSize::Fill),
            "border border-negative text-negative hover:bg-negative hover:text-text-inverse h-11 w-full px-3 text-sm max-sm:min-h-11",
        ),
        (
            ButtonVariant::Header(ButtonSize::Sm),
            "no-print border border-text-inverse/70 text-text-inverse hover:bg-surface hover:text-ink px-3 py-1.5 text-sm max-sm:min-h-11",
        ),
    ];
    for (variant, tail) in cases {
        assert_eq!(
            button_classes(variant, ""),
            format!("{EXPECTED_BASE} {tail}"),
            "{variant:?}"
        );
    }
}

#[test]
fn standalone_variants_do_not_take_size() {
    let cases: [(ButtonVariant, &str); 9] = [
        (ButtonVariant::Ghost, "text-sm text-text-deep hover:underline"),
        (ButtonVariant::Quiet, "chart-toggle-button"),
        (ButtonVariant::Prompt, "review-prompt-button"),
        (ButtonVariant::Login, "login-button"),
        (ButtonVariant::SearchSubmit, "search-submit"),
        (
            ButtonVariant::Retry,
            "inline-flex min-h-11 items-center justify-center rounded-md border border-ink bg-ink px-4 text-sm font-bold text-text-inverse transition-colors hover:bg-ink-hover focus:outline-none focus-visible:ring-2 focus-visible:ring-accent-bright/50 focus-visible:ring-offset-2",
        ),
        (
            ButtonVariant::MenuItem,
            "w-full px-3 py-2 text-left text-sm font-semibold hover:bg-surface-raised",
        ),
        (
            ButtonVariant::MenuItemDanger,
            "w-full px-3 py-2 text-left text-sm font-bold text-negative hover:bg-negative/10",
        ),
        (ButtonVariant::CopyName, "copyable-name"),
    ];
    for (variant, expected) in cases {
        assert_eq!(button_classes(variant, ""), expected, "{variant:?}");
    }
}

#[test]
fn button_classes_appends_extra_class() {
    assert_eq!(
        button_classes(ButtonVariant::Quiet, "no-print mt-2"),
        "chart-toggle-button no-print mt-2"
    );
    assert_eq!(
        button_classes(ButtonVariant::Secondary(ButtonSize::Md), "w-full"),
        format!(
            "{EXPECTED_BASE} border border-border-strong bg-surface text-text hover:border-border-bold hover:bg-surface-sunken px-4 py-2 text-sm max-sm:min-h-11 w-full"
        )
    );
}

#[test]
fn button_size_classes() {
    let cases: [(ButtonSize, &str); 4] = [
        (ButtonSize::Xs, "px-2 py-1 text-xs max-sm:min-h-11"),
        (ButtonSize::Sm, "px-3 py-1.5 text-sm max-sm:min-h-11"),
        (ButtonSize::Md, "px-4 py-2 text-sm max-sm:min-h-11"),
        (ButtonSize::Fill, "h-11 w-full px-3 text-sm max-sm:min-h-11"),
    ];
    for (size, expected) in cases {
        assert_eq!(size.class(), expected, "{size:?}");
    }
}

#[test]
fn icon_button_variant_classes() {
    assert_eq!(
        IconButtonVariant::Boxed.class(),
        "flex h-8 w-8 cursor-pointer items-center justify-center rounded-md border border-border-strong bg-surface text-text-soft max-sm:h-11 max-sm:w-11"
    );
    assert_eq!(IconButtonVariant::Close.class(), "modal-close-button");
}
