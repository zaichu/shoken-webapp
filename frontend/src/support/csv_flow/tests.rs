use super::*;

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
fn begin_save_and_begin_delete_are_busy_gated() {
    let mut state = CsvTabState::<String>::default();
    assert!(state.begin_save());
    assert!(!state.begin_save());
    state.finish_save(Err("x".to_string()));

    assert!(!state.begin_delete());
    state.open_delete_confirm();
    assert!(state.begin_delete());
    assert!(!state.begin_save());
    assert!(!state.begin_delete());
    state.finish_delete(Ok(()));
    assert!(state.begin_save());
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

#[test]
fn upload_response_deserializes() {
    let response: CsvUploadResponse = serde_json::from_str(
        r#"{"inserted":2,"skipped":1,"errors":[{"row":5,"message":"重複"}]}"#,
    )
    .expect("deserialize");
    assert_eq!(
        (response.inserted, response.skipped, response.errors.len()),
        (2, 1, 1)
    );
    assert_eq!(response.errors[0].row, 5);
}

#[test]
fn csv_error_message_matches_react() {
    for (error, expected) in [
        (ApiError::Network, "ネットワークエラーが発生しました"),
        (ApiError::Timeout, "リクエストがタイムアウトしました"),
        (ApiError::Parse, "応答の解析に失敗しました"),
        (ApiError::http(400), "リクエストが不正です"),
        (ApiError::http(401), "認証が必要です"),
        (ApiError::http(403), "アクセスが拒否されました"),
        (ApiError::http(404), "リソースが見つかりません"),
        (ApiError::http(500), "サーバーエラーが発生しました"),
        (ApiError::http(503), "サーバーエラーが発生しました"),
        (
            ApiError::http(418),
            "エラーが発生しました (ステータス: 418)",
        ),
    ] {
        assert_eq!(csv_error_message(&error), expected);
    }
}

#[test]
fn save_label_matches_react() {
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
fn delete_label_matches_react() {
    let mut state = CsvTabState::<String>::default();
    assert_eq!(state.delete_label(5), "全件削除 (5件)");
    state.deleting = true;
    assert_eq!(state.delete_label(5), "削除中...");
}

#[test]
fn select_file_resets_preview_result_and_error() {
    let mut state = CsvTabState::<String> {
        preview: Some(CsvPreview::default()),
        import_result: Some(CsvUploadResponse {
            inserted: 1,
            skipped: 0,
            errors: vec![],
        }),
        error: Some("失敗".to_string()),
        ..Default::default()
    };
    assert!(state.begin_preview("new.csv".to_string()));
    assert_eq!(state.file_name.as_deref(), Some("new.csv"));
    assert!(state.preview.is_none());
    assert!(state.import_result.is_none());
    assert!(state.error.is_none());
    assert!(state.previewing);
}

#[test]
fn busy_state_blocks_begin_operations() {
    for busy in [
        CsvTabState::<String> {
            previewing: true,
            ..Default::default()
        },
        CsvTabState::<String> {
            saving: true,
            ..Default::default()
        },
        CsvTabState::<String> {
            deleting: true,
            ..Default::default()
        },
    ] {
        let mut state = busy;
        assert!(!state.begin_preview("a.csv".to_string()));
        assert!(!state.begin_save());
    }
}

#[test]
fn save_success_clears_preview_and_sets_result() {
    let mut state = CsvTabState::<String> {
        file_name: Some("a.csv".to_string()),
        preview: Some(CsvPreview::default()),
        saving: true,
        ..Default::default()
    };
    state.finish_save(Ok(CsvUploadResponse {
        inserted: 2,
        skipped: 1,
        errors: vec![],
    }));
    assert!(!state.saving);
    assert!(state.file_name.is_none());
    assert!(state.preview.is_none(), "保存成功でプレビューも消す");
    assert_eq!(
        state.import_result.as_ref().map(|result| result.inserted),
        Some(2)
    );
}

#[test]
fn save_error_keeps_file_and_sets_message() {
    let mut state = CsvTabState::<String> {
        file_name: Some("a.csv".to_string()),
        preview: Some(CsvPreview::default()),
        saving: true,
        ..Default::default()
    };
    state.finish_save(Err("認証が必要です".to_string()));
    assert!(!state.saving);
    assert_eq!(state.file_name.as_deref(), Some("a.csv"));
    assert!(state.preview.is_some());
    assert_eq!(state.error.as_deref(), Some("認証が必要です"));
}

#[test]
fn delete_confirm_state_transitions() {
    let mut state = CsvTabState::<String> {
        import_result: Some(CsvUploadResponse {
            inserted: 1,
            skipped: 0,
            errors: vec![],
        }),
        ..Default::default()
    };
    state.open_delete_confirm();
    assert!(state.show_delete_confirm);
    state.close_delete_confirm();
    assert!(!state.show_delete_confirm);

    state.open_delete_confirm();
    assert!(state.begin_delete());
    assert!(!state.show_delete_confirm);
    assert!(state.deleting);
    assert!(!state.begin_delete(), "deleting 中は再開始しない");

    state.finish_delete(Ok(()));
    assert!(!state.deleting);
    assert!(state.import_result.is_none());
}

#[test]
fn begin_delete_requires_open_confirmation() {
    let mut state = CsvTabState::<String>::default();
    assert!(!state.begin_delete());
    assert!(!state.deleting);
    assert!(!state.show_delete_confirm);
}

#[test]
fn delete_error_keeps_result_and_sets_message() {
    let mut state = CsvTabState::<String> {
        import_result: Some(CsvUploadResponse {
            inserted: 1,
            skipped: 0,
            errors: vec![],
        }),
        deleting: true,
        ..Default::default()
    };
    state.finish_delete(Err("サーバーエラーが発生しました".to_string()));
    assert!(!state.deleting);
    assert!(state.import_result.is_some());
    assert_eq!(state.error.as_deref(), Some("サーバーエラーが発生しました"));
}

#[test]
fn notice_texts_match_react() {
    let result = CsvUploadResponse {
        inserted: 2,
        skipped: 1,
        errors: vec![CsvRowError {
            row: 5,
            message: "重複".to_string(),
        }],
    };
    assert_eq!(result.inserted_text(), "2件反映");
    assert_eq!(result.skipped_text().as_deref(), Some("1件スキップ"));
    assert_eq!(result.error_count_text().as_deref(), Some("1件エラー"));
    assert_eq!(row_error_text(&result.errors[0]), "5行目: 重複");

    let clean = CsvUploadResponse {
        inserted: 2,
        skipped: 0,
        errors: vec![],
    };
    assert_eq!(clean.skipped_text(), None);
    assert_eq!(clean.error_count_text(), None);
}
