use super::*;

#[test]
fn section_for_fragment_accepts_only_known_ids() {
    for id in SECTION_IDS {
        assert_eq!(section_for_fragment(id), id);
        assert_eq!(section_for_fragment(&format!("#{id}")), id);
    }
    assert_eq!(section_for_fragment(""), SECTION_IDS[0]);
    assert_eq!(section_for_fragment("#unknown"), SECTION_IDS[0]);
    assert_eq!(section_for_fragment("account-extra"), SECTION_IDS[0]);
}

#[test]
fn toc_entries_cover_all_section_ids() {
    let mut entries: Vec<&str> = TOC_ENTRIES.iter().map(|(id, _)| *id).collect();
    entries.sort_unstable();
    let mut expected: Vec<&str> = SECTION_IDS.to_vec();
    expected.sort_unstable();
    assert_eq!(entries, expected);
}

#[test]
fn pick_current_selects_last_section_above_the_line() {
    let ids = ["a".to_string(), "b".to_string(), "c".to_string()];
    assert_eq!(pick_current(&ids, &[10.0, 20.0, 30.0], 25.0, "a"), "b");
    assert_eq!(pick_current(&ids, &[10.0, 20.0, 30.0], 5.0, "a"), "a");
    assert_eq!(pick_current(&ids, &[10.0, 20.0, 30.0], 100.0, "a"), "c");
}

#[test]
fn pick_current_ignores_missing_sections() {
    let ids = ["a".to_string(), "b".to_string()];
    assert_eq!(pick_current(&ids, &[10.0, f64::MAX], 100.0, "a"), "a");
}

#[test]
fn pick_current_falls_back_when_no_sections() {
    let ids: [String; 0] = [];
    assert_eq!(pick_current(&ids, &[], 100.0, SECTION_IDS[0]), "account");
}
