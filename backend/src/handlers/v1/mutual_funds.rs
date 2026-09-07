use crate::{
    errors::{ApiError, ErrorResponse},
    extractors::auth::AuthenticatedUser,
    models::{
        common::{MessageResponse, PaginatedSearchResponse, SearchFacets},
        csv_import::{CsvPreviewResponse, CsvUploadForm, CsvUploadResponse},
        mutualfund::{Mutualfund, MutualfundSearchQueryParams, MutualfundSummary},
    },
    services::{csv_domain::MutualfundDomain, mutualfund as mutualfund_service},
    state::AppState,
};
use axum::{
    extract::{Multipart, Query, State},
    response::IntoResponse,
};

// 投資信託取引ハンドラー
// ---------------------------------------------------------------------------

/// 投資信託一覧を取得（v1）
#[utoipa::path(
    get,
    path = "/api/v1/mutual-fund-transactions",
    operation_id = "v1_mutual_fund_transaction_list",
    params(
        ("page" = Option<i64>, Query, description = "ページ番号（デフォルト: 1）"),
        ("per_page" = Option<i64>, Query, description = "1ページあたりの件数（デフォルト: 200、最大: 1000）"),
        ("q" = Option<String>, Query, description = "フリーワード検索（口座/ファンド名/分配金の token AND 検索）"),
        ("date_from" = Option<String>, Query, description = "約定日の開始日（YYYY-MM-DD）"),
        ("date_to" = Option<String>, Query, description = "約定日の終了日（YYYY-MM-DD）"),
        ("year" = Option<i32>, Query, description = "約定日の年（YYYY）"),
        ("year_month" = Option<String>, Query, description = "約定日の年月（YYYY-MM）"),
        ("date" = Option<String>, Query, description = "約定日の単日指定（YYYY-MM-DD）"),
        ("account" = Option<String>, Query, description = "口座での絞り込み"),
        ("fund_name" = Option<String>, Query, description = "ファンド名での絞り込み"),
        ("dividends" = Option<String>, Query, description = "分配金での絞り込み"),
        ("include_summary" = Option<bool>, Query, description = "検索条件全体の集計を含めるか"),
        ("include_facets" = Option<bool>, Query, description = "検索候補 facets を含めるか"),
    ),
    responses(
        (status = 200, body = PaginatedSearchResponse<Mutualfund, MutualfundSummary, SearchFacets>),
        (status = 400, body = ErrorResponse),
        (status = 401, body = ErrorResponse),
    ),
    security(("cookieAuth" = []))
)]
pub async fn list_transactions(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Query(params): Query<MutualfundSearchQueryParams>,
) -> Result<impl IntoResponse, ApiError> {
    crate::handlers::csv_import::handle_list(mutualfund_service::search(
        &state.pool,
        auth_user.id(),
        &params,
    ))
    .await
}

/// 投資信託を全削除（v1）
#[utoipa::path(
    delete,
    path = "/api/v1/mutual-fund-transactions",
    operation_id = "v1_mutual_fund_transaction_delete_all",
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
        mutualfund_service::delete_all(&state.pool, auth_user.id()),
        "全ての投資信託データを削除しました",
    )
    .await
}

/// 投資信託 CSV をバリデーション（v1）
#[utoipa::path(
    post,
    path = "/api/v1/mutual-fund-import-validations",
    operation_id = "v1_mutual_fund_validate_import",
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
    crate::handlers::csv_import::handle_validate_import::<MutualfundDomain>(_auth_user, multipart)
        .await
}

/// 投資信託 CSV をインポート（v1）
#[utoipa::path(
    post,
    path = "/api/v1/mutual-fund-imports",
    operation_id = "v1_mutual_fund_import",
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
    crate::handlers::csv_import::handle_import_csv::<MutualfundDomain>(
        &state.pool,
        auth_user.id(),
        multipart,
    )
    .await
}

// ---------------------------------------------------------------------------
