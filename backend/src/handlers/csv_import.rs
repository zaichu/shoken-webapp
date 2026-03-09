use crate::errors::ApiError;
use crate::services::csv_domain::CsvDomain;
use axum::{extract::Multipart, http::StatusCode, response::IntoResponse, Json};
use uuid::Uuid;

/// マルチパートフォームから `file` フィールドのバイト列を取得する
pub async fn read_csv_file_bytes(mut multipart: Multipart) -> Result<Vec<u8>, ApiError> {
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        ApiError::ValidationError(format!("マルチパートの読み込みに失敗しました: {}", e))
    })? {
        if field.name() == Some("file") {
            // ファイル拡張子チェック（.csv のみ許可・ファイル名なしも拒否）
            let filename = field.file_name().unwrap_or("");
            if !filename.to_lowercase().ends_with(".csv") {
                return Err(ApiError::ValidationError(
                    "CSVファイル（.csv）のみアップロードできます".to_string(),
                ));
            }
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

/// ドメイン共通のプレビュー処理
///
/// ハンドラーから `handle_preview_csv::<DividendDomain>(multipart).await` のように呼ぶ。
pub async fn handle_preview_csv<D: CsvDomain>(
    multipart: Multipart,
) -> Result<impl IntoResponse, ApiError> {
    let bytes = read_csv_file_bytes(multipart).await?;
    let response = D::preview_csv(&bytes)?;
    Ok((StatusCode::OK, Json(response)))
}

/// ドメイン共通のアップロード処理
///
/// ハンドラーから `handle_upload_csv::<DividendDomain>(&state.pool, auth_user.id(), multipart).await` のように呼ぶ。
pub async fn handle_upload_csv<D: CsvDomain>(
    pool: &sqlx::PgPool,
    user_id: Uuid,
    multipart: Multipart,
) -> Result<impl IntoResponse, ApiError> {
    let bytes = read_csv_file_bytes(multipart).await?;
    let response = D::upload_csv(pool, user_id, &bytes).await?;
    Ok((StatusCode::CREATED, Json(response)))
}
