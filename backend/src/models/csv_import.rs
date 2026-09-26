use utoipa::ToSchema;

pub use shared::csv_import::{CsvPreviewResponse, CsvRowError, CsvUploadResponse};

/// CSV アップロードのリクエストボディ（multipart/form-data の file フィールド）
/// OpenAPI スキーマ定義専用の型。実行時に直接参照されないため dead_code を抑制する。
#[derive(ToSchema)]
#[allow(dead_code)]
pub struct CsvUploadForm {
    /// アップロードするCSVファイル（Shift-JIS または UTF-8）
    #[schema(format = Binary)]
    pub file: String,
}
