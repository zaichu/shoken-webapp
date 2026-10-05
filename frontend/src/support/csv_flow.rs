use crate::api::dto::{CsvPreviewResponse, CsvRowError, CsvUploadResponse};
use crate::api::{ApiClient, ApiError};

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

    pub fn has_preview_rows(&self) -> bool {
        self.preview
            .as_ref()
            .is_some_and(|preview| !preview.rows.is_empty())
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
            self.preview.as_ref().map_or_else(
                || action.to_string(),
                |preview| format!("{}件 {}", preview.valid_rows, action),
            )
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

// 共有クレートの型に固有メソッドを生やせないため、表示用文字列の組み立ては拡張トレイトで提供する
pub trait CsvUploadResponseExt {
    fn inserted_text(&self) -> String;
    fn skipped_text(&self) -> Option<String>;
    fn error_count_text(&self) -> Option<String>;
}

impl CsvUploadResponseExt for CsvUploadResponse {
    fn inserted_text(&self) -> String {
        format!("{}件反映", self.inserted)
    }

    fn skipped_text(&self) -> Option<String> {
        (self.skipped > 0).then(|| format!("{}件スキップ", self.skipped))
    }

    fn error_count_text(&self) -> Option<String> {
        (!self.errors.is_empty()).then(|| format!("{}件エラー", self.errors.len()))
    }
}

pub fn row_error_text(error: &CsvRowError) -> String {
    format!("{}行目: {}", error.row, error.message)
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
mod tests;
