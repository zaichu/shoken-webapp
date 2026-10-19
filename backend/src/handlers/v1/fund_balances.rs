use crate::{
    errors::{ApiError, ErrorResponse},
    extractors::auth::AuthenticatedUser,
    models::{
        common::{MessageResponse, PaginatedSearchResponse},
        csv_import::{CsvPreviewResponse, CsvUploadForm, CsvUploadResponse},
        fund_balance::{FundBalance, FundBalanceSearchQueryParams, FundBalanceSummary},
    },
    services::fund_balance::FundBalanceDomain,
    state::AppState,
};
use axum::{
    extract::{Multipart, Query, State},
    response::IntoResponse,
};

/// 投資信託残高一覧を取得（v1）
#[utoipa::path(
    get,
    path = "/api/v1/fund-balances",
    operation_id = "v1_fund_balance_list",
    params(
        ("page" = Option<i64>, Query, description = "ページ番号（デフォルト: 1）"),
        ("per_page" = Option<i64>, Query, description = "1ページあたりの件数（デフォルト: 200、最大: 1000）"),
        ("q" = Option<String>, Query, description = "フリーワード検索（ファンド名/口座区分の token AND 検索）"),
        ("fund_name" = Option<String>, Query, description = "ファンド名での絞り込み"),
        ("account" = Option<String>, Query, description = "口座区分での絞り込み"),
        ("include_summary" = Option<bool>, Query, description = "検索条件全体の集計を含めるか"),
        ("include_facets" = Option<bool>, Query, description = "検索候補 facets を含めるか"),
    ),
    responses(
        (status = 200, description = "投資信託残高の一覧を返す", body = PaginatedSearchResponse<FundBalance, FundBalanceSummary>),
        (status = 400, description = "検索パラメータが不正", body = ErrorResponse),
        (status = 401, description = "認証が必要", body = ErrorResponse),
        (status = 429, description = "レート制限を超過", body = ErrorResponse),
    ),
    security(("cookieAuth" = []))
)]
pub async fn list(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Query(params): Query<FundBalanceSearchQueryParams>,
) -> Result<impl IntoResponse, ApiError> {
    crate::handlers::common::handle_search::<FundBalanceDomain>(&state.pool, auth_user.id(), params)
        .await
}

/// 投資信託残高を全削除（v1）
#[utoipa::path(
    delete,
    path = "/api/v1/fund-balances",
    operation_id = "v1_fund_balance_delete_all",
    responses(
        (status = 200, description = "全ての投資信託残高データを削除しました", body = MessageResponse),
        (status = 401, description = "認証が必要", body = ErrorResponse),
        (status = 429, description = "レート制限を超過", body = ErrorResponse),
    ),
    security(("cookieAuth" = []))
)]
pub async fn delete_all(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<impl IntoResponse, ApiError> {
    crate::handlers::v1::csv_import::handle_delete_all::<FundBalanceDomain>(
        &state.pool,
        auth_user.id(),
        "全ての投資信託残高データを削除しました",
    )
    .await
}

/// 投資信託残高 CSV をバリデーション（v1）
#[utoipa::path(
    post,
    path = "/api/v1/fund-balance-import-validations",
    operation_id = "v1_fund_balance_validate_import",
    request_body(content = CsvUploadForm, content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "CSV のプレビューを返す", body = CsvPreviewResponse),
        (status = 400, description = "CSV の形式が不正", body = ErrorResponse),
        (status = 401, description = "認証が必要", body = ErrorResponse),
        (status = 429, description = "レート制限を超過", body = ErrorResponse),
    ),
    security(("cookieAuth" = []))
)]
pub async fn validate_import(
    _auth_user: AuthenticatedUser,
    multipart: Multipart,
) -> Result<impl IntoResponse, ApiError> {
    crate::handlers::v1::csv_import::handle_preview_csv::<FundBalanceDomain>(multipart).await
}

/// 投資信託残高 CSV をインポート（v1）
#[utoipa::path(
    post,
    path = "/api/v1/fund-balance-imports",
    operation_id = "v1_fund_balance_import",
    request_body(content = CsvUploadForm, content_type = "multipart/form-data"),
    responses(
        (status = 201, description = "CSV のインポートが完了しました", body = CsvUploadResponse),
        (status = 400, description = "CSV の形式が不正", body = ErrorResponse),
        (status = 401, description = "認証が必要", body = ErrorResponse),
    ),
    security(("cookieAuth" = []))
)]
pub async fn import(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    multipart: Multipart,
) -> Result<impl IntoResponse, ApiError> {
    crate::handlers::v1::csv_import::handle_import_csv::<FundBalanceDomain>(
        &state.pool,
        auth_user.id(),
        multipart,
        state.config.user_row_limit,
    )
    .await
}
