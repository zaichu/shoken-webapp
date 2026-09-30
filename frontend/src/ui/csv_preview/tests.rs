use super::*;

#[test]
fn preview_notice_text_uses_save_action() {
    assert_eq!(
        preview_notice_text(5, "追加で保存"),
        "5件 追加で保存されます"
    );
    assert_eq!(
        preview_notice_text(9, "全件置換で保存"),
        "9件 全件置換で保存されます"
    );
    assert_eq!(
        preview_notice_text(0, "追加で保存"),
        "0件 追加で保存されます"
    );
}
