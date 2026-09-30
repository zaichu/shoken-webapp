use super::*;

#[test]
fn preview_notice_text_uses_save_action() {
    for (count, action) in [(5, "追加で保存"), (9, "全件置換で保存"), (0, "追加で保存")]
    {
        let notice = preview_notice_text(count, action);
        assert!(notice.contains(&count.to_string()));
        assert!(notice.contains(action));
    }
}

#[test]
fn preview_notice_only_for_selected_settled_file() {
    assert!(should_show_preview_notice(true, true, false));
    for (authenticated, has_file, previewing) in [
        (false, true, false),
        (true, false, false),
        (true, true, true),
        (false, false, false),
    ] {
        assert!(!should_show_preview_notice(
            authenticated,
            has_file,
            previewing
        ));
    }
}
