use utoipa::ToSchema;

pub use shared::csv_import::{CsvPreviewResponse, CsvRowError, CsvUploadResponse};

/// CSV アップロードのリクエストボディ（multipart/form-data の file フィールド）
#[derive(ToSchema)]
pub struct CsvUploadForm {
    /// アップロードするCSVファイル（Shift-JIS または UTF-8）
    #[schema(format = Binary)]
    pub file: String,
}
