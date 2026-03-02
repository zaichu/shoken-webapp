use serde::Serialize;
use utoipa::ToSchema;

/// CSV アップロードのリクエストボディ（multipart/form-data の file フィールド）
/// OpenAPI スキーマ定義専用の型。実行時に直接参照されないため dead_code を抑制する。
#[derive(ToSchema)]
#[allow(dead_code)]
pub struct CsvUploadForm {
    /// アップロードするCSVファイル（Shift-JIS または UTF-8）
    #[schema(format = Binary)]
    pub file: String,
}

/// CSV アップロードのレスポンス
#[derive(Debug, Serialize, ToSchema)]
pub struct CsvUploadResponse {
    pub inserted: usize,
    pub skipped: usize,
    pub errors: Vec<CsvRowError>,
}

/// CSV の行エラー情報
#[derive(Debug, Serialize, ToSchema)]
pub struct CsvRowError {
    /// 1始まり（ヘッダー行を除く）
    pub row: usize,
    pub message: String,
}

/// CSV プレビューのレスポンス（DB 書き込みなし）
#[derive(Debug, Serialize, ToSchema)]
pub struct CsvPreviewResponse {
    /// CSVの総行数（ヘッダー除く）
    pub total_rows: usize,
    /// パース成功行数
    pub valid_rows: usize,
    /// 行エラー一覧
    pub errors: Vec<CsvRowError>,
    /// パース成功行のデータ（保存前プレビュー用）
    #[schema(value_type = Vec<Object>)]
    pub rows: Vec<serde_json::Value>,
}
