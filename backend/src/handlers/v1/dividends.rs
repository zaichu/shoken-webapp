use crate::{
    errors::{ApiError, ErrorResponse},
    extractors::{auth::AuthenticatedUser, validated_json::ValidatedJson},
    handlers::common::ok_message,
    models::{
        common::{MessageResponse, PaginatedSearchResponse, SearchFacets},
        csv_import::{CsvPreviewResponse, CsvUploadForm, CsvUploadResponse},
        dividend::{Dividend, DividendSearchQueryParams, DividendSummary},
        dividend_cache::{DividendPerShareBatchRequest, DividendPerShareBatchResponse},
    },
    services::{csv_domain::DividendDomain, dividend as dividend_service},
    state::AppState,
};
use axum::{
    extract::{Multipart, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};

// 配当金ハンドラー
// ---------------------------------------------------------------------------

/// 配当金一覧を取得（v1）
#[utoipa::path(
    get,
    path = "/api/v1/dividends",
    operation_id = "v1_dividend_list",
    params(
        ("page" = Option<i64>, Query, description = "ページ番号（デフォルト: 1）"),
        ("per_page" = Option<i64>, Query, description = "1ページあたりの件数（デフォルト: 200、最大: 1000）"),
        ("q" = Option<String>, Query, description = "フリーワード検索（商品/口座/銘柄コード/銘柄名の token AND 検索）"),
        ("date_from" = Option<String>, Query, description = "決済日の開始日（YYYY-MM-DD）"),
        ("date_to" = Option<String>, Query, description = "決済日の終了日（YYYY-MM-DD）"),
        ("year" = Option<i32>, Query, description = "決済日の年（YYYY）"),
        ("year_month" = Option<String>, Query, description = "決済日の年月（YYYY-MM）"),
        ("date" = Option<String>, Query, description = "決済日の単日指定（YYYY-MM-DD）"),
        ("product" = Option<String>, Query, description = "商品での絞り込み"),
        ("account" = Option<String>, Query, description = "口座での絞り込み"),
        ("security_code" = Option<String>, Query, description = "銘柄コードでの絞り込み"),
        ("security_name" = Option<String>, Query, description = "銘柄名での絞り込み"),
        ("include_summary" = Option<bool>, Query, description = "検索条件全体の集計を含めるか"),
        ("include_facets" = Option<bool>, Query, description = "検索候補 facets を含めるか"),
    ),
    responses(
        (status = 200, body = PaginatedSearchResponse<Dividend, DividendSummary, SearchFacets>),
        (status = 400, body = ErrorResponse),
        (status = 401, body = ErrorResponse),
    ),
    security(("cookieAuth" = []))
)]
pub async fn list(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Query(params): Query<DividendSearchQueryParams>,
) -> Result<impl IntoResponse, ApiError> {
    let result = dividend_service::search(&state.pool, auth_user.id(), &params).await?;
    Ok((StatusCode::OK, Json(result)))
}

/// 配当金を全削除（v1）
#[utoipa::path(
    delete,
    path = "/api/v1/dividends",
    operation_id = "v1_dividend_delete_all",
    responses(
        (status = 200, body = MessageResponse),
        (status = 401, body = ErrorResponse),
    ),
    security(("cookieAuth" = []))
)]
pub async fn delete_all(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<impl IntoResponse, ApiError> {
    dividend_service::delete_all(&state.pool, auth_user.id()).await?;
    Ok(ok_message("全ての配当金データを削除しました"))
}

/// 配当金 CSV をバリデーション（DB 書き込みなし）（v1）
#[utoipa::path(
    post,
    path = "/api/v1/dividend-import-validations",
    operation_id = "v1_dividend_validate_import",
    request_body(content = CsvUploadForm, content_type = "multipart/form-data"),
    responses(
        (status = 200, body = CsvPreviewResponse),
        (status = 400, body = ErrorResponse),
        (status = 401, body = ErrorResponse),
    ),
    security(("cookieAuth" = []))
)]
pub async fn validate_import(
    _auth_user: AuthenticatedUser,
    multipart: Multipart,
) -> Result<impl IntoResponse, ApiError> {
    crate::handlers::csv_import::handle_preview_csv::<DividendDomain>(multipart).await
}

/// 配当金 CSV をインポート（v1）
#[utoipa::path(
    post,
    path = "/api/v1/dividend-imports",
    operation_id = "v1_dividend_import",
    request_body(content = CsvUploadForm, content_type = "multipart/form-data"),
    responses(
        (status = 201, body = CsvUploadResponse),
        (status = 400, body = ErrorResponse),
        (status = 401, body = ErrorResponse),
    ),
    security(("cookieAuth" = []))
)]
pub async fn import(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    multipart: Multipart,
) -> Result<impl IntoResponse, ApiError> {
    let json = crate::handlers::csv_import::handle_upload_csv::<DividendDomain>(
        &state.pool,
        auth_user.id(),
        multipart,
    )
    .await?;
    Ok((StatusCode::CREATED, json))
}

/// 配当利回りを一括取得（v1）
#[utoipa::path(
    post,
    path = "/api/v1/dividend-per-share-estimates",
    operation_id = "v1_dividend_per_share_estimate",
    request_body = DividendPerShareBatchRequest,
    responses(
        (status = 200, body = DividendPerShareBatchResponse),
        (status = 400, body = ErrorResponse),
        (status = 401, body = ErrorResponse),
    ),
    security(("cookieAuth" = []))
)]
pub async fn estimate_per_share(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    ValidatedJson(data): ValidatedJson<DividendPerShareBatchRequest>,
) -> Result<impl IntoResponse, ApiError> {
    crate::handlers::dividend_per_share::batch(State(state), auth_user, ValidatedJson(data)).await
}

// ---------------------------------------------------------------------------
