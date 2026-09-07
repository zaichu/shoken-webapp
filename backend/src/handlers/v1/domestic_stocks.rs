use crate::{
    errors::{ApiError, ErrorResponse},
    extractors::auth::AuthenticatedUser,
    models::{
        common::{MessageResponse, PaginatedSearchResponse, SearchFacets},
        csv_import::{CsvPreviewResponse, CsvUploadForm, CsvUploadResponse},
        domestic_stock::{DomesticStock, DomesticStockSearchQueryParams, DomesticStockSummary},
    },
    services::{csv_domain::DomesticStockDomain, domestic_stock as domestic_stock_service},
    state::AppState,
};
use axum::{
    extract::{Multipart, Query, State},
    response::IntoResponse,
};

// 国内株式取引ハンドラー
// ---------------------------------------------------------------------------

/// 国内株式取引一覧を取得（v1）
#[utoipa::path(
    get,
    path = "/api/v1/domestic-stock-transactions",
    operation_id = "v1_domestic_stock_transaction_list",
    params(
        ("page" = Option<i64>, Query, description = "ページ番号（デフォルト: 1）"),
        ("per_page" = Option<i64>, Query, description = "1ページあたりの件数（デフォルト: 200、最大: 1000）"),
        ("q" = Option<String>, Query, description = "フリーワード検索（口座/銘柄コード/銘柄名の token AND 検索）"),
        ("date_from" = Option<String>, Query, description = "約定日の開始日（YYYY-MM-DD）"),
        ("date_to" = Option<String>, Query, description = "約定日の終了日（YYYY-MM-DD）"),
        ("year" = Option<i32>, Query, description = "約定日の年（YYYY）"),
        ("year_month" = Option<String>, Query, description = "約定日の年月（YYYY-MM）"),
        ("date" = Option<String>, Query, description = "約定日の単日指定（YYYY-MM-DD）"),
        ("account" = Option<String>, Query, description = "口座での絞り込み"),
        ("security_code" = Option<String>, Query, description = "銘柄コードでの絞り込み"),
        ("security_name" = Option<String>, Query, description = "銘柄名での絞り込み"),
        ("include_summary" = Option<bool>, Query, description = "検索条件全体の集計を含めるか"),
        ("include_facets" = Option<bool>, Query, description = "検索候補 facets を含めるか"),
    ),
    responses(
        (status = 200, body = PaginatedSearchResponse<DomesticStock, DomesticStockSummary, SearchFacets>),
        (status = 400, body = ErrorResponse),
        (status = 401, body = ErrorResponse),
    ),
    security(("cookieAuth" = []))
)]
pub async fn list_transactions(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Query(params): Query<DomesticStockSearchQueryParams>,
) -> Result<impl IntoResponse, ApiError> {
    crate::handlers::csv_import::handle_list(domestic_stock_service::search(
        &state.pool,
        auth_user.id(),
        &params,
    ))
    .await
}

/// 国内株式取引を全削除（v1）
#[utoipa::path(
    delete,
    path = "/api/v1/domestic-stock-transactions",
    operation_id = "v1_domestic_stock_transaction_delete_all",
    responses(
        (status = 200, body = MessageResponse),
        (status = 401, body = ErrorResponse),
    ),
    security(("cookieAuth" = []))
)]
pub async fn delete_transactions(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<impl IntoResponse, ApiError> {
    crate::handlers::csv_import::handle_delete_all(
        domestic_stock_service::delete_all(&state.pool, auth_user.id()),
        "全ての国内株式取引データを削除しました",
    )
    .await
}

/// 国内株式取引 CSV をバリデーション（v1）
#[utoipa::path(
    post,
    path = "/api/v1/domestic-stock-import-validations",
    operation_id = "v1_domestic_stock_validate_import",
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
    crate::handlers::csv_import::handle_validate_import::<DomesticStockDomain>(
        _auth_user, multipart,
    )
    .await
}

/// 国内株式取引 CSV をインポート（v1）
#[utoipa::path(
    post,
    path = "/api/v1/domestic-stock-imports",
    operation_id = "v1_domestic_stock_import",
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
    crate::handlers::csv_import::handle_import_csv::<DomesticStockDomain>(
        &state.pool,
        auth_user.id(),
        multipart,
    )
    .await
}

// ---------------------------------------------------------------------------
