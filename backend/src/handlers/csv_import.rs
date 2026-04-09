use crate::errors::ApiError;
use crate::services::csv_domain::CsvDomain;
use crate::state::AppState;
use axum::{extract::Multipart, http::StatusCode, response::IntoResponse, Json, Router};
use uuid::Uuid;

pub fn csv_import_routes() -> Router<AppState> {
    Router::new()
}

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
    use async_trait::async_trait;
    use axum::{
        body::{to_bytes, Body},
        extract::{Multipart, State},
        http::{Request, StatusCode},
        routing::post,
        Router,
    };
    use serde::de::DeserializeOwned;
    use sqlx::postgres::PgPoolOptions;
    use tower::ServiceExt;
    use uuid::Uuid;

    const BODY_LIMIT: usize = 1024 * 1024;

    fn test_app() -> Router {
        Router::new().route("/csv", post(csv_bytes_endpoint))
    }

    async fn csv_bytes_endpoint(multipart: Multipart) -> Result<Vec<u8>, ApiError> {
        read_csv_file_bytes(multipart).await
    }

    struct PreviewDomain;

    #[async_trait]
    impl CsvDomain for PreviewDomain {
        fn preview_csv(
            bytes: &[u8],
        ) -> Result<crate::models::csv_import::CsvPreviewResponse, ApiError> {
            Ok(crate::models::csv_import::CsvPreviewResponse {
                total_rows: bytes.len(),
                valid_rows: 1,
                errors: vec![],
                rows: vec![serde_json::json!({"ok": true})],
            })
        }

        async fn upload_csv(
            _pool: &sqlx::PgPool,
            _user_id: Uuid,
            _bytes: &[u8],
        ) -> Result<crate::models::csv_import::CsvUploadResponse, ApiError> {
            unreachable!("preview test does not call upload")
        }
    }

    struct UploadDomain;

    #[async_trait]
    impl CsvDomain for UploadDomain {
        fn preview_csv(
            _bytes: &[u8],
        ) -> Result<crate::models::csv_import::CsvPreviewResponse, ApiError> {
            unreachable!("upload test does not call preview")
        }

        async fn upload_csv(
            _pool: &sqlx::PgPool,
            _user_id: Uuid,
            bytes: &[u8],
        ) -> Result<crate::models::csv_import::CsvUploadResponse, ApiError> {
            Ok(crate::models::csv_import::CsvUploadResponse {
                inserted: usize::from(!bytes.is_empty()),
                skipped: 0,
                errors: vec![],
            })
        }
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

    async fn read_json_response<T: DeserializeOwned>(response: axum::response::Response) -> T {
        let body = to_bytes(response.into_body(), BODY_LIMIT).await.unwrap();
        serde_json::from_slice(&body).unwrap()
    }

    fn preview_app() -> Router {
        Router::new().route("/csv", post(preview_endpoint))
    }

    async fn preview_endpoint(multipart: Multipart) -> Result<impl IntoResponse, ApiError> {
        handle_preview_csv::<PreviewDomain>(multipart).await
    }

    fn upload_app() -> Router {
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect_lazy("postgresql://user:password@localhost/test_db")
            .unwrap();

        Router::new()
            .route("/csv", post(upload_endpoint))
            .with_state(pool)
    }

    async fn upload_endpoint(
        State(pool): State<sqlx::PgPool>,
        multipart: Multipart,
    ) -> Result<impl IntoResponse, ApiError> {
        handle_upload_csv::<UploadDomain>(&pool, Uuid::nil(), multipart).await
    }

    #[tokio::test]
    async fn test_read_csv_file_bytes() {
        let content = "symbol,amount\n7203,100\n";
        let response = test_app()
            .oneshot(multipart_request("file", Some("positions.csv"), content))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), BODY_LIMIT).await.unwrap();
        assert_eq!(body.as_ref(), content.as_bytes());
        for (field, filename, msg_fragment) in [
            ("other", Some("positions.csv"), Some("fileフィールド")),
            ("file", Some("positions.txt"), Some(".csv")),
            ("file", Some(""), None),
        ] {
            let response = test_app()
                .oneshot(multipart_request(field, filename, "dummy"))
                .await
                .unwrap();

            assert_eq!(response.status(), StatusCode::BAD_REQUEST);
            let error: ErrorResponse = read_json_response(response).await;
            assert_eq!(error.error.code, "VALIDATION_ERROR");
            if let Some(fragment) = msg_fragment {
                assert!(error.error.message.contains(fragment));
            }
        }
    }

    #[tokio::test]
    async fn test_handle_csv_endpoints() {
        let response = preview_app()
            .oneshot(multipart_request("file", Some("preview.csv"), "a,b\n1,2\n"))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let preview: serde_json::Value = read_json_response(response).await;
        assert_eq!(preview["valid_rows"], 1);
        assert_eq!(preview["rows"].as_array().unwrap().len(), 1);

        let response = upload_app()
            .oneshot(multipart_request("file", Some("upload.csv"), "a,b\n1,2\n"))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
        let upload: serde_json::Value = read_json_response(response).await;
        assert_eq!(upload["inserted"], 1);
        assert_eq!(upload["skipped"], 0);
    }
}
