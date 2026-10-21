use super::*;

#[test]
fn badge_variant_classes_are_distinguishable() {
    let variants = [BadgeVariant::Positive];
    let classes: Vec<&str> = variants.iter().map(|variant| variant.class()).collect();
    for (i, class) in classes.iter().enumerate() {
        assert!(!class.is_empty(), "{:?}", variants[i]);
        assert!(!classes[..i].contains(class), "{:?}", variants[i]);
    }
}

#[test]
fn account_display_name_shortens_known_labels() {
    for (input, expected) in [
        ("特定口座", "特定"),
        ("特定", "特定"),
        ("一般口座", "一般"),
        ("NISA口座", "NISA"),
        ("新NISA", "NISA"),
        ("NISA成長投資枠", "NISA成長"),
        ("NISAつみたて投資枠", "NISAつみたて"),
        ("旧NISA", "旧NISA"),
    ] {
        assert_eq!(account_display_name(input), expected);
    }
}

#[test]
fn account_display_name_keeps_unknown_labels() {
    for input in [
        "特定・一般",
        "SBI証券",
        "SBI証券 東京本店第一営業部 特定口座(新NISA成長投資枠・つみたて投資枠・iDeCo・ジュニアNISA兼用) 管理番号1234567890",
        "",
    ] {
        assert_eq!(account_display_name(input), input);
    }
}

#[test]
fn account_labels_split_joined_csv_accounts() {
    assert_eq!(account_labels("特定"), vec!["特定"]);
    assert_eq!(account_labels("特定・一般"), vec!["特定", "一般"]);
    assert_eq!(
        account_labels(" NISA成長投資枠 ・ 旧NISA "),
        vec!["NISA成長投資枠", "旧NISA"]
    );
    assert_eq!(account_labels("・"), Vec::<&str>::new());
}

#[test]
fn account_badge_class_picks_kind_by_label() {
    assert_eq!(
        account_badge_class("特定"),
        "account-badge account-badge-specific"
    );
    assert_eq!(
        account_badge_class("旧NISA"),
        "account-badge account-badge-old-nisa"
    );
    for label in ["NISA", "新NISA", "NISA成長", "NISAつみたて"] {
        assert_eq!(
            account_badge_class(label),
            "account-badge account-badge-nisa",
            "{label}"
        );
    }
    for label in ["一般", "SBI証券", "nisa", ""] {
        assert_eq!(account_badge_class(label), "account-badge", "{label}");
    }
}
