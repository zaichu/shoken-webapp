use crate::errors::ApiError;
use axum::extract::Multipart;

/// マルチパートフォームから `file` フィールドのバイト列を取得する
pub async fn read_csv_file_bytes(mut multipart: Multipart) -> Result<Vec<u8>, ApiError> {
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        ApiError::ValidationError(format!("マルチパートの読み込みに失敗しました: {}", e))
    })? {
        if field.name() == Some("file") {
            let bytes = field.bytes().await.map_err(|e| {
                ApiError::ValidationError(format!("ファイルの読み込みに失敗しました: {}", e))
            })?;
            return Ok(bytes.to_vec());
        }
    }
    Err(ApiError::ValidationError(
        "fileフィールドが見つかりません".to_string(),
    ))
}
