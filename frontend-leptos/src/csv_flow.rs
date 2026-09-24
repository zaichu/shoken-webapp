use crate::api::{ApiClient, ApiError};
use crate::dto::{CsvPreviewResponse, CsvRowError, CsvUploadResponse};

#[derive(Clone, Debug, PartialEq)]
pub struct CsvPreview<R> {
    pub total_rows: usize,
    pub valid_rows: usize,
    pub errors: Vec<CsvRowError>,
    pub rows: Vec<R>,
}

impl<R> Default for CsvPreview<R> {
    fn default() -> Self {
        Self {
            total_rows: 0,
            valid_rows: 0,
            errors: Vec::new(),
            rows: Vec::new(),
        }
    }
}

impl<R> CsvPreview<R> {
    // 画面は有効件数とエラー件数を別々のスタイルで描画するため、連結済みのこの文言は未使用
    #[allow(dead_code)]
    pub fn summary_text(&self, action: &str) -> String {
        let base = format!("{}件 {}", self.valid_rows, action);
        if self.errors.is_empty() {
            base
        } else {
            format!("{base} / {}件エラー", self.errors.len())
        }
    }

    pub fn from_response(
        response: CsvPreviewResponse,
        parse: impl Fn(serde_json::Value) -> R,
    ) -> Self {
        Self {
            total_rows: response.total_rows,
            valid_rows: response.valid_rows,
            errors: response.errors,
            rows: response.rows.into_iter().map(parse).collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CsvTabState<R> {
    pub file_name: Option<String>,
    pub preview: Option<CsvPreview<R>>,
    pub import_result: Option<CsvUploadResponse>,
    pub error: Option<String>,
    pub previewing: bool,
    pub saving: bool,
    pub deleting: bool,
    pub show_delete_confirm: bool,
}

impl<R> Default for CsvTabState<R> {
    fn default() -> Self {
        Self {
            file_name: None,
            preview: None,
            import_result: None,
            error: None,
            previewing: false,
            saving: false,
            deleting: false,
            show_delete_confirm: false,
        }
    }
}

impl<R> CsvTabState<R> {
    pub fn busy(&self) -> bool {
        self.previewing || self.saving || self.deleting
    }

    pub fn begin_preview(&mut self, file_name: String) -> bool {
        if self.busy() {
            return false;
        }
        self.file_name = Some(file_name);
        self.preview = None;
        self.import_result = None;
        self.error = None;
        self.previewing = true;
        true
    }

    // プレビュー失敗は画面に出さない(ファイル選択は残し preview のみ未設定)
    pub fn finish_preview(&mut self, preview: Option<CsvPreview<R>>) {
        self.previewing = false;
        if let Some(preview) = preview {
            self.preview = Some(preview);
        }
    }

    // 資産管理側はプレビュー失敗を画面に出す
    #[allow(dead_code)]
    pub fn fail_preview(&mut self, message: String) {
        self.previewing = false;
        self.error = Some(message);
    }

    pub fn begin_save(&mut self) -> bool {
        if self.busy() {
            return false;
        }
        self.error = None;
        self.saving = true;
        true
    }

    pub fn finish_save(&mut self, result: Result<CsvUploadResponse, String>) {
        self.saving = false;
        match result {
            Ok(result) => {
                self.file_name = None;
                // 消さないと全件削除後もプレビュー行が一覧に残る
                self.preview = None;
                self.import_result = Some(result);
            }
            Err(message) => self.error = Some(message),
        }
    }

    pub fn open_delete_confirm(&mut self) {
        self.show_delete_confirm = true;
    }

    pub fn close_delete_confirm(&mut self) {
        self.show_delete_confirm = false;
    }

    // 誤起動しないよう確認表示中だけ開始する
    pub fn begin_delete(&mut self) -> bool {
        if self.deleting || !self.show_delete_confirm {
            return false;
        }
        self.show_delete_confirm = false;
        self.error = None;
        self.deleting = true;
        true
    }

    pub fn finish_delete(&mut self, result: Result<(), String>) {
        self.deleting = false;
        match result {
            Ok(()) => self.import_result = None,
            Err(message) => self.error = Some(message),
        }
    }

    pub fn save_label(&self, action: &str) -> String {
        if self.saving {
            "保存中...".to_string()
        } else if self.previewing {
            "解析中...".to_string()
        } else {
            self.preview
                .as_ref()
                .map(|preview| format!("{}件 {}", preview.valid_rows, action))
                .unwrap_or_else(|| action.to_string())
        }
    }

    pub fn delete_label(&self, db_count: usize) -> String {
        if self.deleting {
            "削除中...".to_string()
        } else {
            format!("全件削除 ({db_count}件)")
        }
    }
}

impl CsvUploadResponse {
    pub fn inserted_text(&self) -> String {
        format!("{}件反映", self.inserted)
    }

    pub fn skipped_text(&self) -> Option<String> {
        (self.skipped > 0).then(|| format!("{}件スキップ", self.skipped))
    }

    pub fn error_count_text(&self) -> Option<String> {
        (!self.errors.is_empty()).then(|| format!("{}件エラー", self.errors.len()))
    }
}

pub fn row_error_text(error: &CsvRowError) -> String {
    format!("{}行目: {}", error.row, error.message)
}

// user_message() とは別の文言体系(HTTP ステータス別の既定文)を使う
pub fn csv_error_message(error: &ApiError) -> String {
    match error {
        ApiError::Network => "ネットワークエラーが発生しました".to_string(),
        ApiError::Timeout => "リクエストがタイムアウトしました".to_string(),
        ApiError::Parse => "応答の解析に失敗しました".to_string(),
        ApiError::Http { status } => match status {
            400 => "リクエストが不正です".to_string(),
            401 => "認証が必要です".to_string(),
            403 => "アクセスが拒否されました".to_string(),
            404 => "リソースが見つかりません".to_string(),
            500..=599 => "サーバーエラーが発生しました".to_string(),
            _ => format!("エラーが発生しました (ステータス: {status})"),
        },
    }
}

fn csv_form_data(file: &web_sys::File) -> Result<web_sys::FormData, ApiError> {
    let form = web_sys::FormData::new().map_err(|_| ApiError::Network)?;
    form.append_with_blob("file", file)
        .map_err(|_| ApiError::Network)?;
    Ok(form)
}

pub async fn preview_csv(path: &str, file: &web_sys::File) -> Result<CsvPreviewResponse, ApiError> {
    ApiClient::default_client()
        .post_multipart(path, &csv_form_data(file)?)
        .await
}

pub async fn upload_csv(path: &str, file: &web_sys::File) -> Result<CsvUploadResponse, ApiError> {
    ApiClient::default_client()
        .post_multipart(path, &csv_form_data(file)?)
        .await
}

pub async fn delete_all(path: &str) -> Result<(), ApiError> {
    ApiClient::default_client().delete_empty(path).await
}

#[cfg(test)]
mod tests {
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
            (ApiError::Http { status: 400 }, "リクエストが不正です"),
            (ApiError::Http { status: 401 }, "認証が必要です"),
            (ApiError::Http { status: 403 }, "アクセスが拒否されました"),
            (ApiError::Http { status: 404 }, "リソースが見つかりません"),
            (
                ApiError::Http { status: 500 },
                "サーバーエラーが発生しました",
            ),
            (
                ApiError::Http { status: 503 },
                "サーバーエラーが発生しました",
            ),
            (
                ApiError::Http { status: 418 },
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
        assert!(
            state.preview.is_none(),
            "React は保存成功の SET_RAW_FILE でプレビューも消す"
        );
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
        let preview = CsvPreview::<String> {
            valid_rows: 3,
            errors: vec![],
            ..Default::default()
        };
        assert_eq!(
            preview.summary_text("追加で保存されます"),
            "3件 追加で保存されます"
        );
        let with_errors = CsvPreview::<String> {
            valid_rows: 3,
            errors: vec![CsvRowError {
                row: 2,
                message: "形式エラー".to_string(),
            }],
            ..Default::default()
        };
        assert_eq!(
            with_errors.summary_text("追加で保存されます"),
            "3件 追加で保存されます / 1件エラー"
        );

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
}
