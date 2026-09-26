use crate::errors::{ApiError, CsvError};
use crate::handlers::common::ok_message;
use crate::models::common::MessageResponse;
use crate::models::csv_import::CsvUploadResponse;
use crate::services::csv_domain::CsvDomain;
use axum::{extract::Multipart, http::StatusCode, response::IntoResponse, Json};
use uuid::Uuid;

/// マルチパートフォームから `file` フィールドのバイト列を取得する
pub async fn read_csv_file_bytes(mut multipart: Multipart) -> Result<Vec<u8>, ApiError> {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| CsvError::MultipartRead(e.to_string()))?
    {
        if field.name() == Some("file") {
            // ファイル拡張子チェック（.csv のみ許可・ファイル名なしも拒否）
            let filename = field.file_name().unwrap_or("");
            if !filename.to_lowercase().ends_with(".csv") {
                return Err(CsvError::InvalidFileType.into());
            }
            let bytes = field
                .bytes()
                .await
                .map_err(|e| CsvError::FileRead(e.to_string()))?;
            return Ok(bytes.to_vec());
        }
    }
    Err(CsvError::MissingFile.into())
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
/// ハンドラーから `handle_upload_csv::<DividendDomain>(&state.pool, auth_user.id(), multipart, state.config.user_row_limit).await` のように呼ぶ。
/// HTTP ステータスコードは呼び出し元ハンドラーが決める。
pub async fn handle_upload_csv<D: CsvDomain>(
    pool: &sqlx::PgPool,
    user_id: Uuid,
    multipart: Multipart,
    user_row_limit: i64,
) -> Result<Json<CsvUploadResponse>, ApiError> {
    let bytes = read_csv_file_bytes(multipart).await?;
    let response = D::upload_csv(pool, user_id, &bytes, user_row_limit).await?;
    Ok(Json(response))
}

/// 全削除の定型処理（削除実行 + 完了メッセージ）
///
/// 各ドメインの `delete_all` ハンドラーから
/// `handle_delete_all(service::delete_all(&state.pool, auth_user.id()), "メッセージ").await`
/// のように呼ぶ。戻り値を具体型にすることで、呼び出し元の借用が戻り値に漏れ出さないようにする。
pub async fn handle_delete_all(
    delete: impl std::future::Future<Output = Result<u64, ApiError>>,
    message: &str,
) -> Result<(StatusCode, Json<MessageResponse>), ApiError> {
    delete.await?;
    Ok(ok_message(message))
}

/// CSV インポートの委譲ヘルパー（201 Created + JSON）
///
/// 実処理は `handle_upload_csv` に委譲し、ステータスコード付与まで面倒を見る。
/// 戻り値を具体型にすることで、呼び出し元の借用（`&state.pool`）が戻り値に漏れ出さないようにする。
pub async fn handle_import_csv<D: CsvDomain>(
    pool: &sqlx::PgPool,
    user_id: Uuid,
    multipart: Multipart,
    user_row_limit: i64,
) -> Result<(StatusCode, Json<CsvUploadResponse>), ApiError> {
    let json = handle_upload_csv::<D>(pool, user_id, multipart, user_row_limit).await?;
    Ok((StatusCode::CREATED, json))
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::ErrorResponse;
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

    impl CsvDomain for PreviewDomain {
        fn preview_csv(
            bytes: &[u8],
        ) -> Result<crate::models::csv_import::CsvPreviewResponse, ApiError> {
            Ok(crate::models::csv_import::CsvPreviewResponse {
                total_rows: bytes.len(),
                valid_rows: 1,
                errors: vec![],
                rows: vec![serde_json::json!({ "ok": true })],
            })
        }

        async fn upload_csv(
            _pool: &sqlx::PgPool,
            _user_id: Uuid,
            _bytes: &[u8],
            _user_row_limit: i64,
        ) -> Result<crate::models::csv_import::CsvUploadResponse, ApiError> {
            unreachable!("preview test does not call upload")
        }
    }

    struct UploadDomain;

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
            _user_row_limit: i64,
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
        let content_disposition = filename.map_or_else(
            || format!("Content-Disposition: form-data; name=\"{field_name}\"\r\n"),
            |filename| {
                format!(
                    "Content-Disposition: form-data; name=\"{field_name}\"; filename=\"{filename}\"\r\n"
                )
            },
        );
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
        let json =
            handle_upload_csv::<UploadDomain>(&pool, Uuid::nil(), multipart, 100_000).await?;
        Ok((StatusCode::CREATED, json))
    }

    #[tokio::test]
    async fn test_csv_handler() {
        let content = "symbol,amount\n7203,100\n";
        let response = test_app()
            .oneshot(multipart_request("file", Some("positions.csv"), content))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            to_bytes(response.into_body(), BODY_LIMIT)
                .await
                .unwrap()
                .as_ref(),
            content.as_bytes()
        );

        for (field, filename, msg_fragment) in [
            ("other", Some("positions.csv"), Some("fileフィールド")),
            ("file", Some("positions.txt"), Some(".csv")),
            ("file", Some(""), None),
        ] {
            let response = test_app()
                .oneshot(multipart_request(field, filename, "dummy"))
                .await
                .unwrap();
            let status = response.status();
            let error: ErrorResponse = read_json_response(response).await;

            assert_eq!(
                (
                    status,
                    error.error.code.as_str(),
                    msg_fragment.is_none_or(|fragment| { error.error.message.contains(fragment) }),
                ),
                (StatusCode::BAD_REQUEST, "CSV_ERROR", true)
            );
        }

        let response = preview_app()
            .oneshot(multipart_request("file", Some("preview.csv"), "a,b\n1,2\n"))
            .await
            .unwrap();
        let status = response.status();
        let preview: serde_json::Value = read_json_response(response).await;
        assert_eq!(
            (
                status,
                preview["valid_rows"].as_i64(),
                preview["rows"].as_array().map(Vec::len),
            ),
            (StatusCode::OK, Some(1), Some(1))
        );

        let response = upload_app()
            .oneshot(multipart_request("file", Some("upload.csv"), "a,b\n1,2\n"))
            .await
            .unwrap();
        let status = response.status();
        let upload: serde_json::Value = read_json_response(response).await;
        assert_eq!(
            (
                status,
                upload["inserted"].as_i64(),
                upload["skipped"].as_i64(),
            ),
            (StatusCode::CREATED, Some(1), Some(0))
        );
    }
}
