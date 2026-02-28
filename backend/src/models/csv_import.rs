use serde::Serialize;
use utoipa::ToSchema;

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
