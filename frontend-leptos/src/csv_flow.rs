#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_label_uses_append_action() {
        let mut state = CsvTabState::<String>::default();
        assert_eq!(state.save_label("追加で保存"), "追加で保存");
        state.preview = Some(CsvPreview {
            valid_rows: 3,
            ..Default::default()
        });
        assert_eq!(state.save_label("追加で保存"), "3件 追加で保存");
        state.previewing = true;
        assert_eq!(state.save_label("追加で保存"), "解析中...");
        state.saving = true;
        assert_eq!(state.save_label("追加で保存"), "保存中...");
    }

    #[test]
    fn save_label_uses_replace_action() {
        let mut state = CsvTabState::<String>::default();
        assert_eq!(state.save_label("全件置換で保存"), "全件置換で保存");
        state.preview = Some(CsvPreview {
            valid_rows: 3,
            ..Default::default()
        });
        assert_eq!(state.save_label("全件置換で保存"), "3件 全件置換で保存");
        state.previewing = true;
        assert_eq!(state.save_label("全件置換で保存"), "解析中...");
        state.saving = true;
        assert_eq!(state.save_label("全件置換で保存"), "保存中...");
    }

    #[test]
    fn fail_preview_clears_busy_and_keeps_file() {
        let mut state = CsvTabState::<String> {
            file_name: Some("asset.csv".to_string()),
            previewing: true,
            ..Default::default()
        };
        state.fail_preview("サーバーエラーが発生しました".to_string());
        assert!(!state.previewing);
        assert_eq!(state.file_name.as_deref(), Some("asset.csv"));
        assert_eq!(state.error.as_deref(), Some("サーバーエラーが発生しました"));
        assert!(state.preview.is_none());
    }
}
