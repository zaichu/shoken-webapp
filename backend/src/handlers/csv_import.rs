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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::ErrorResponse;
    use axum::{
        body::{to_bytes, Body},
        extract::Multipart,
        http::{Request, StatusCode},
        routing::post,
        Router,
    };
    use tower::ServiceExt;

    const BODY_LIMIT: usize = 1024 * 1024;

    fn test_app() -> Router {
        Router::new().route("/csv", post(csv_bytes_endpoint))
    }

    async fn csv_bytes_endpoint(multipart: Multipart) -> Result<Vec<u8>, ApiError> {
        read_csv_file_bytes(multipart).await
    }

    fn multipart_request(field_name: &str, filename: Option<&str>, content: &str) -> Request<Body> {
        let boundary = "boundary123";
        let content_disposition = match filename {
            Some(filename) => format!(
                "Content-Disposition: form-data; name=\"{field_name}\"; filename=\"{filename}\"\r\n"
            ),
            None => format!("Content-Disposition: form-data; name=\"{field_name}\"\r\n"),
        };
        let body = format!(
            "--{boundary}\r\n{content_disposition}Content-Type: text/csv\r\n\r\n{content}\r\n--{boundary}--\r\n"
        );

        Request::builder()
            .method("POST")
            .uri("/csv")
            .header(
                "content-type",
                format!("multipart/form-data; boundary={boundary}"),
            )
            .body(Body::from(body))
            .unwrap()
    }

    async fn read_error_response(response: axum::response::Response) -> ErrorResponse {
        let body = to_bytes(response.into_body(), BODY_LIMIT).await.unwrap();
        serde_json::from_slice(&body).unwrap()
    }

    #[tokio::test]
    async fn test_read_csv_file_bytes_valid() {
        let content = "symbol,amount\n7203,100\n";
        let response = test_app()
            .oneshot(multipart_request("file", Some("positions.csv"), content))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), BODY_LIMIT).await.unwrap();
        assert_eq!(body.as_ref(), content.as_bytes());
    }

    #[tokio::test]
    async fn test_read_csv_file_bytes_no_file_field() {
        let response = test_app()
            .oneshot(multipart_request("other", Some("positions.csv"), "dummy"))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let error = read_error_response(response).await;
        assert_eq!(error.error.code, "VALIDATION_ERROR");
        assert!(error.error.message.contains("fileフィールド"));
    }

    #[tokio::test]
    async fn test_read_csv_file_bytes_wrong_extension() {
        let response = test_app()
            .oneshot(multipart_request("file", Some("positions.txt"), "dummy"))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let error = read_error_response(response).await;
        assert_eq!(error.error.code, "VALIDATION_ERROR");
        assert!(error.error.message.contains(".csv"));
    }

    #[tokio::test]
    async fn test_read_csv_file_bytes_no_filename() {
        let response = test_app()
            .oneshot(multipart_request("file", Some(""), "dummy"))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let error = read_error_response(response).await;
        assert_eq!(error.error.code, "VALIDATION_ERROR");
    }
}
