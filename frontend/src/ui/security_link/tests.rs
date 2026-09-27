use super::*;

#[test]
fn searchable_code_matches_react_regex() {
    assert!(is_searchable_code("7203"));
    assert!(is_searchable_code("BRK.B"));
    assert!(!is_searchable_code(""));
    assert!(!is_searchable_code("7203: トヨタ"));
    assert!(!is_searchable_code("７２０３"));
}

#[test]
fn copy_text_matches_react_format() {
    assert_eq!(
        instrument_copy_text("トヨタ自動車", Some("7203")),
        "トヨタ自動車(7203)"
    );
    assert_eq!(instrument_copy_text("トヨタ自動車", None), "トヨタ自動車");
    assert_eq!(instrument_copy_text("  ", Some("7203")), "—(7203)");
    assert_eq!(
        instrument_copy_text("名", Some("7203: トヨタ自動車")),
        "名(7203)"
    );
    assert_eq!(instrument_copy_text("名", Some("brk.b")), "名(BRK.B)");
    assert_eq!(instrument_copy_text("名", Some("  ")), "名");
}

#[test]
fn font_weight_detection_matches_react_regex() {
    assert!(has_font_weight_class("font-semibold"));
    assert!(has_font_weight_class("text-xs sm:font-medium"));
    assert!(has_font_weight_class("hover:font-bold"));
    assert!(!has_font_weight_class("text-xs"));
    assert!(!has_font_weight_class("font-boldx"));
    assert!(!has_font_weight_class("font-bold_x"));
    assert!(!has_font_weight_class("font-bold1"));
    assert!(!has_font_weight_class(""));
}
