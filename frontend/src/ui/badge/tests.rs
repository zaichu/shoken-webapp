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
fn account_badge_color_distinguishes_kinds() {
    let general = account_badge_class("一般");
    let specific = account_badge_class("特定");
    let nisa = account_badge_class("NISA");
    let old_nisa = account_badge_class("旧NISA");

    // 単独の口座名は種類ごとに色が変わる
    for (left, right) in [
        (specific, general),
        (nisa, general),
        (old_nisa, general),
        (nisa, old_nisa),
    ] {
        assert_ne!(left, right);
    }

    // NISA 系は同じ種類として同じ色になる
    for label in ["新NISA", "NISA成長", "NISAつみたて"] {
        assert_eq!(account_badge_class(label), nisa, "{label}");
    }

    // 対応表にない口座名は一般と同じ中立色
    for label in ["SBI証券", "nisa", ""] {
        assert_eq!(account_badge_class(label), general, "{label}");
    }
}

#[test]
fn joined_account_is_one_badge_with_the_general_color() {
    // 連結した口座名(「特定・一般」)は分割せず1つの札にする
    assert_eq!(account_display_name("特定・一般"), "特定・一般");
    // 単独の口座ではないので一般と同じ中立色
    assert_eq!(
        account_badge_class("特定・一般"),
        account_badge_class("一般")
    );
    // 単独の「特定」とは区別される
    assert_ne!(
        account_badge_class("特定"),
        account_badge_class("特定・一般")
    );
}
