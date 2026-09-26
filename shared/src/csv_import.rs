use serde::{Deserialize, Serialize};

/// CSV アップロードのレスポンス
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct CsvUploadResponse {
    pub inserted: usize,
    pub skipped: usize,
    pub errors: Vec<CsvRowError>,
}

/// CSV の行エラー情報
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct CsvRowError {
    /// 1始まり（ヘッダー行を除く）
    pub row: usize,
    pub message: String,
}

/// CSV プレビューのレスポンス（DB 書き込みなし）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct CsvPreviewResponse {
    /// CSVの総行数（ヘッダー除く）
    pub total_rows: usize,
    /// パース成功行数
    pub valid_rows: usize,
    /// 行エラー一覧
    pub errors: Vec<CsvRowError>,
    /// パース成功行のデータ（保存前プレビュー用）
    #[cfg_attr(feature = "utoipa", schema(value_type = Vec<Object>))]
    pub rows: Vec<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csv_upload_response_roundtrips() {
        let response = CsvUploadResponse {
            inserted: 10,
            skipped: 2,
            errors: vec![CsvRowError {
                row: 3,
                message: "数量が数値ではありません".to_string(),
            }],
        };

        let json = serde_json::to_value(&response).expect("serialize");
        assert_eq!(
            json,
            serde_json::json!({
                "inserted": 10,
                "skipped": 2,
                "errors": [{"row": 3, "message": "数量が数値ではありません"}]
            })
        );
        let parsed: CsvUploadResponse = serde_json::from_value(json).expect("deserialize");
        assert_eq!(parsed, response);
    }

    #[test]
    fn csv_preview_response_roundtrips_rows_as_json() {
        let response = CsvPreviewResponse {
            total_rows: 2,
            valid_rows: 1,
            errors: vec![],
            rows: vec![serde_json::json!({"code": "7203", "name": "トヨタ自動車"})],
        };

        let json = serde_json::to_value(&response).expect("serialize");
        let parsed: CsvPreviewResponse = serde_json::from_value(json).expect("deserialize");
        assert_eq!(parsed, response);
    }
}
