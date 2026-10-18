use super::*;
use crate::api::dto::CsvRowError;

#[test]
fn preview_notice_text_says_what_can_be_saved() {
    for (count, action) in [(5, "追加で保存"), (9, "全件置換で保存")] {
        let notice = preview_notice_text(count, 0, action);
        assert!(notice.starts_with(&format!("{count}件を")));
        assert!(notice.contains(action));
        assert!(notice.contains("できます"), "保存前の案内と分かる言い方");
    }
    // 行エラーがあるときは件数を添える
    let notice = preview_notice_text(3, 2, "追加で保存");
    assert!(notice.contains("3件を追加で保存できます"));
    assert!(notice.contains("2件エラー"));
}

#[test]
fn preview_notice_text_without_valid_rows_tells_nothing_importable() {
    let notice = preview_notice_text(0, 36, "追加で保存");
    assert!(notice.contains("取り込める行がありません"));
    assert!(notice.contains("36件エラー"));
    // エラー0件で行が無いときも同じ見出し
    let empty = preview_notice_text(0, 0, "全件置換で保存");
    assert!(empty.contains("取り込める行がありません"));
}

#[test]
fn shared_message_only_when_every_row_failed_the_same_way() {
    let error = |row, message: &str| CsvRowError {
        row,
        message: message.to_string(),
    };

    let same: Vec<_> = (1..=6)
        .map(|row| error(row, "受取通貨が不正です"))
        .collect();
    assert_eq!(shared_message(&same).as_deref(), Some("受取通貨が不正です"));

    // 件数が少ない同一エラーは集約しない(そのまま並ぶ方が読みやすい)
    let few: Vec<_> = (1..=3)
        .map(|row| error(row, "受取通貨が不正です"))
        .collect();
    assert_eq!(shared_message(&few), None);

    // 1件でも違う理由が混ざれば集約しない
    let mut mixed = same.clone();
    mixed.push(error(7, "数量が数値ではありません"));
    assert_eq!(shared_message(&mixed), None);

    assert_eq!(shared_message(&[]), None);
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
