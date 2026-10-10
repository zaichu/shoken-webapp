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

    pub fn meta(&self) -> CsvTabMeta {
        CsvTabMeta {
            file_name: self.file_name.clone(),
            preview: self.preview.as_ref().map(|preview| CsvPreviewMeta {
                valid_rows: preview.valid_rows,
                errors: preview.errors.clone(),
                has_rows: !preview.rows.is_empty(),
            }),
            import_result: self.import_result.clone(),
            error: self.error.clone(),
            previewing: self.previewing,
            saving: self.saving,
            deleting: self.deleting,
            show_delete_confirm: self.show_delete_confirm,
        }
    }
}

// プレビュー行を含まない CSV 状態の投影。
// 共通 UI(CsvSection)の Memo が読むのはこれだけにして、評価ごとの全行複製を避ける
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CsvTabMeta {
    pub file_name: Option<String>,
    pub preview: Option<CsvPreviewMeta>,
    pub import_result: Option<CsvUploadResponse>,
    pub error: Option<String>,
    pub previewing: bool,
    pub saving: bool,
    pub deleting: bool,
    pub show_delete_confirm: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CsvPreviewMeta {
    pub valid_rows: usize,
    pub errors: Vec<CsvRowError>,
    pub has_rows: bool,
}

impl CsvTabMeta {
    pub fn busy(&self) -> bool {
        self.previewing || self.saving || self.deleting
    }

    pub fn has_preview_rows(&self) -> bool {
        self.preview
            .as_ref()
            .is_some_and(|preview| preview.has_rows)
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

fn chunk_form_data(chunk: &web_sys::Blob, filename: &str) -> Result<web_sys::FormData, ApiError> {
    let form = web_sys::FormData::new().map_err(|_| ApiError::Network)?;
    form.append_with_blob_and_filename("file", chunk, filename)
        .map_err(|_| ApiError::Network)?;
    Ok(form)
}

/// 1 リクエストあたりのデータ行上限。Workers 無料枠の CPU 10ms/呼び出しに収めるため
/// 余裕を持った値(wrangler dev 実測で parse+transform は約 4µs/行)
const CSV_CHUNK_ROWS: usize = 500;

/// CSV の分割送信指定。`header_lines` は backend の
/// `CsvParserConfig::skip_header_rows` + ヘッダ行の合計に一致させる
#[derive(Clone, Copy)]
pub struct CsvChunking {
    /// 各チャンクの先頭に付ける前置行数(前置きスキップ行 + ヘッダ行)
    pub header_lines: usize,
    /// false のとき import は分割しない。
    /// 置換型ドメインは分割すると後続チャンクが前チャンクの行を消すため
    pub import_chunked: bool,
}

/// CSV バイト列をレコード境界(引用符内の改行を除く)で切る。
/// '"'/'\n' は Shift_JIS(CP932)・UTF-8 のマルチバイト内に出現しないため
/// バイト走査だけで安全に分割できる
fn record_ranges(bytes: &[u8]) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut start = 0;
    let mut in_quotes = false;
    let mut field_start = true;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'"' if !in_quotes && field_start => {
                in_quotes = true;
                field_start = false;
            }
            b'"' if in_quotes => {
                if bytes.get(i + 1) == Some(&b'"') {
                    i += 1;
                } else {
                    in_quotes = false;
                }
            }
            b',' if !in_quotes => field_start = true,
            b'\n' if !in_quotes => {
                ranges.push((start, i + 1));
                start = i + 1;
                field_start = true;
            }
            _ if !in_quotes => field_start = false,
            _ => {}
        }
        i += 1;
    }
    if start < bytes.len() {
        ranges.push((start, bytes.len()));
    }
    ranges
}

/// データ行を最大 max_rows ずつの連続レンジに切る。
/// backend(domestic_stock)は完全同一行を occurrence_index 付きで保持するため、
/// バイト列が同じレコード群がチャンク境界を跨ぐと単発送信と結果が変わる。
/// 境界を跨ぐ同バイト群はまとめて前のチャンクに引き込む
fn chunk_record_groups<'a>(
    bytes: &[u8],
    data: &'a [(usize, usize)],
    max_rows: usize,
) -> Vec<&'a [(usize, usize)]> {
    use std::collections::HashMap;
    // 各レコードのバイト列が最後に現れる位置。チャンク末尾はこの最大値まで伸ばす
    let mut last_index: HashMap<&[u8], usize> = HashMap::new();
    for (i, (start, end)) in data.iter().enumerate() {
        last_index.insert(&bytes[*start..*end], i);
    }
    let mut groups = Vec::new();
    let mut start = 0;
    while start < data.len() {
        let mut end = (start + max_rows).min(data.len());
        // チャンク内のレコードと同じバイト列が後方に残る限り末尾を伸ばす
        while let Some(last) = data[start..end]
            .iter()
            .map(|(s, e)| last_index[&bytes[*s..*e]])
            .max()
            .filter(|last| *last >= end)
        {
            end = last + 1;
        }
        groups.push(&data[start..end]);
        start = end;
    }
    groups
}

/// ファイルを「前置行 + 最大 max_rows レコード」のチャンク Blob 列に切る。
/// 小さいファイルはそのまま 1 要素で返す
async fn split_file_chunks(
    file: &web_sys::File,
    header_lines: usize,
    max_rows: usize,
) -> Result<Vec<web_sys::Blob>, ApiError> {
    let buf = wasm_bindgen_futures::JsFuture::from(file.array_buffer())
        .await
        .map_err(|_| ApiError::Network)?;
    let bytes = js_sys::Uint8Array::new(&buf).to_vec();
    let records = record_ranges(&bytes);
    let header_records = records.len().min(header_lines);
    let data = &records[header_records..];
    if data.len() <= max_rows {
        // 前置行を含めてそのまま送れる
        return file
            .slice_with_f64_and_f64(0.0, file.size())
            .map(|b| vec![b])
            .map_err(|_| ApiError::Network);
    }
    let header_end = records
        .get(header_records.saturating_sub(1))
        .map(|(_, end)| *end)
        .unwrap_or(0) as f64;
    let header = file
        .slice_with_f64_and_f64(0.0, header_end)
        .map_err(|_| ApiError::Network)?;
    chunk_record_groups(&bytes, data, max_rows)
        .iter()
        .map(|group| {
            let data_slice = file
                .slice_with_f64_and_f64(group[0].0 as f64, group.last().unwrap().1 as f64)
                .map_err(|_| ApiError::Network)?;
            let parts = js_sys::Array::new();
            parts.push(&header);
            parts.push(&data_slice);
            web_sys::Blob::new_with_u8_array_sequence(&parts).map_err(|_| ApiError::Network)
        })
        .collect()
}

pub async fn preview_csv(
    path: &str,
    file: &web_sys::File,
    chunking: CsvChunking,
) -> Result<CsvPreviewResponse, ApiError> {
    let chunks = split_file_chunks(file, chunking.header_lines, CSV_CHUNK_ROWS).await?;
    let client = ApiClient::default_client();
    // backend の拡張子ガードと E2E モックの filename ルックアップが実名を使うため維持する
    let filename = file.name();
    let mut merged = CsvPreviewResponse {
        total_rows: 0,
        valid_rows: 0,
        errors: Vec::new(),
        rows: Vec::new(),
    };
    for chunk in chunks {
        let mut response: CsvPreviewResponse = client
            .post_multipart(path, &chunk_form_data(&chunk, &filename)?)
            .await?;
        // backend の行番号は「ヘッダ・空行・除外行を除いた処理対象レコードの連番」なので、
        // チャンク内番号に直前までの処理対象数(total_rows)を足すと単発送信と一致する
        for error in &mut response.errors {
            error.row += merged.total_rows;
        }
        merged.total_rows += response.total_rows;
        merged.valid_rows += response.valid_rows;
        merged.errors.append(&mut response.errors);
        merged.rows.append(&mut response.rows);
    }
    Ok(merged)
}

/// 分割 import は「途中で失敗すると手前のチャンクだけコミットされる」。
/// Append 型ドメインは ON CONFLICT の重複スキップで冪等なので、
/// 失敗時は同じファイルをそのまま再送すれば残りが追いつく
pub async fn upload_csv(
    path: &str,
    file: &web_sys::File,
    chunking: CsvChunking,
) -> Result<CsvUploadResponse, ApiError> {
    if !chunking.import_chunked {
        return ApiClient::default_client()
            .post_multipart(path, &csv_form_data(file)?)
            .await;
    }
    let chunks = split_file_chunks(file, chunking.header_lines, CSV_CHUNK_ROWS).await?;
    let client = ApiClient::default_client();
    let filename = file.name();
    let mut merged = CsvUploadResponse {
        inserted: 0,
        skipped: 0,
        errors: Vec::new(),
    };
    let mut offset = 0usize;
    for chunk in chunks {
        let mut response: CsvUploadResponse = client
            .post_multipart(path, &chunk_form_data(&chunk, &filename)?)
            .await?;
        for error in &mut response.errors {
            error.row += offset;
        }
        // 行番号は処理対象レコードの連番(= inserted+skipped+errors)
        offset += response.inserted + response.skipped + response.errors.len();
        merged.inserted += response.inserted;
        merged.skipped += response.skipped;
        merged.errors.append(&mut response.errors);
    }
    Ok(merged)
}

pub async fn delete_all(path: &str) -> Result<(), ApiError> {
    ApiClient::default_client().delete_empty(path).await
}

#[cfg(test)]
mod tests;
