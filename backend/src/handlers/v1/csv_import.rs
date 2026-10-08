use crate::db::Db;
use crate::errors::{ApiError, CsvError};
use crate::handlers::common::ok_message;
use crate::models::common::MessageResponse;
use crate::models::csv_import::CsvUploadResponse;
use crate::services::csv::import::CsvImport;
use crate::services::domain::Domain;
use crate::services::domain::bulk::{self, RowLimit};
use axum::{Json, extract::Multipart, http::StatusCode, response::IntoResponse};
use shared::value::UserId;

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

pub async fn handle_preview_csv<D: CsvImport>(
    multipart: Multipart,
) -> Result<impl IntoResponse, ApiError> {
    let bytes = read_csv_file_bytes(multipart).await?;
    let response = D::preview_csv(&bytes)?;
    Ok((StatusCode::OK, Json(response)))
}

pub async fn handle_delete_all<D: Domain>(
    pool: &Db,
    user_id: UserId,
    message: &str,
) -> Result<(StatusCode, Json<MessageResponse>), ApiError> {
    bulk::delete_all::<D>(pool, user_id).await?;
    Ok(ok_message(message))
}

/// 戻り値を具体型にすることで、呼び出し元の借用（`&state.pool`）が戻り値に漏れ出さないようにする。
pub async fn handle_import_csv<D: CsvImport>(
    pool: &Db,
    user_id: UserId,
    multipart: Multipart,
    user_row_limit: RowLimit,
) -> Result<(StatusCode, Json<CsvUploadResponse>), ApiError> {
    let bytes = read_csv_file_bytes(multipart).await?;
    let response = D::upload_csv(pool, user_id, &bytes, user_row_limit).await?;
    Ok((StatusCode::CREATED, Json(response)))
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::ErrorResponse;
    use crate::models::common::BulkCreateResponse;
    use crate::models::csv_import::CsvRowError;
    use crate::services::csv::import::validate_csv_rows;
    use crate::services::csv::pipeline::{CsvParserConfig, CsvTable};
    use crate::services::csv::util::CsvCells;
    use axum::{
        Router,
        body::{Body, to_bytes},
        extract::{Multipart, State},
        http::{Request, StatusCode},
        routing::post,
    };
    use serde::de::DeserializeOwned;
    use tower::ServiceExt;

    const BODY_LIMIT: usize = 1024 * 1024;

    fn test_app() -> Router {
        Router::new().route("/csv", post(csv_bytes_endpoint))
    }

    async fn csv_bytes_endpoint(multipart: Multipart) -> Result<Vec<u8>, ApiError> {
        read_csv_file_bytes(multipart).await
    }

    struct TestCsvImport;

    impl CsvImport for TestCsvImport {
        type Row = String;
        const CSV_CONFIG: CsvParserConfig = CsvParserConfig {
            skip_header_rows: 0,
            exclude_row_fn: None,
        };

        fn transform_rows(table: &CsvTable) -> (Vec<Self::Row>, Vec<CsvRowError>) {
            validate_csv_rows(table, |row, row_num| {
                let value = row.cell("a");
                if value.is_empty() {
                    Err(CsvRowError {
                        row: row_num.get(),
                        message: "a is empty".to_string(),
                    })
                } else {
                    Ok(value.to_string())
                }
            })
        }

        async fn bulk_create(
            _pool: &Db,
            user_id: UserId,
            items: &[Self::Row],
            limit: RowLimit,
        ) -> Result<BulkCreateResponse, ApiError> {
            assert_eq!(user_id, UserId::default());
            assert_eq!(limit, RowLimit::new(100_000));
            if items.iter().any(|value| value == "fail") {
                return Err(ApiError::Internal("test upload failure"));
            }
            Ok(BulkCreateResponse {
                inserted: items.len(),
                skipped: 0,
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
        handle_preview_csv::<TestCsvImport>(multipart).await
    }

    fn upload_app() -> Router {
        let pool = crate::db::connect_pool_lazy("postgresql://user:password@localhost/test_db", 1)
            .unwrap();

        Router::new()
            .route("/csv", post(upload_endpoint))
            .with_state(pool)
    }

    async fn upload_endpoint(
        State(pool): State<Db>,
        multipart: Multipart,
    ) -> Result<impl IntoResponse, ApiError> {
        handle_import_csv::<TestCsvImport>(
            &pool,
            UserId::default(),
            multipart,
            RowLimit::new(100_000),
        )
        .await
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

    #[tokio::test]
    async fn test_csv_default_pipeline_preserves_valid_rows_and_errors() {
        let content = "a,b\n1,2\n,3\n4,5\n";
        let response = preview_app()
            .oneshot(multipart_request("file", Some("preview.csv"), content))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let preview: serde_json::Value = read_json_response(response).await;
        assert_eq!(preview["total_rows"], 3);
        assert_eq!(preview["valid_rows"], 2);
        assert_eq!(preview["rows"], serde_json::json!(["1", "4"]));
        assert_eq!(preview["errors"].as_array().unwrap().len(), 1);
        assert_eq!(preview["errors"][0]["row"], 2);

        let response = upload_app()
            .oneshot(multipart_request("file", Some("upload.csv"), content))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
        let upload: serde_json::Value = read_json_response(response).await;
        assert_eq!(upload["inserted"], 2);
        assert_eq!(upload["skipped"], 0);
        assert_eq!(upload["errors"], preview["errors"]);
    }

    #[tokio::test]
    async fn test_csv_default_pipeline_propagates_parse_and_bulk_errors() {
        for (content, status, code) in [
            ("", StatusCode::BAD_REQUEST, "CSV_ERROR"),
            (
                "a,b\nfail,1\n",
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
            ),
        ] {
            let response = upload_app()
                .oneshot(multipart_request("file", Some("upload.csv"), content))
                .await
                .unwrap();
            assert_eq!(response.status(), status);
            let error: ErrorResponse = read_json_response(response).await;
            assert_eq!(error.error.code, code);
        }
    }
}
