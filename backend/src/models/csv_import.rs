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
