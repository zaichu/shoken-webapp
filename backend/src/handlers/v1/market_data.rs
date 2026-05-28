use crate::{
    errors::{ApiError, ErrorResponse},
    extractors::auth::AuthenticatedUser,
    models::jquants::{FinSummaryQuery, FinSummaryResponse},
    state::AppState,
};
use axum::{
    extract::{Query, State},
    Json,
};

// Financial statement handlers
// ---------------------------------------------------------------------------

/// 決算サマリーを取得（v1）
#[utoipa::path(
    get,
    path = "/api/v1/financial-statements",
    operation_id = "v1_get_financial_statements",
    params(
        ("code" = String, Query, description = "銘柄コード"),
        ("from" = Option<String>, Query, description = "開始日（YYYY-MM-DD）"),
        ("to" = Option<String>, Query, description = "終了日（YYYY-MM-DD）"),
    ),
    responses(
        (status = 200, body = FinSummaryResponse),
        (status = 400, body = ErrorResponse),
        (status = 401, body = ErrorResponse),
        (status = 429, body = ErrorResponse),
        (status = 502, body = ErrorResponse),
    ),
    security(("cookieAuth" = []))
)]
pub async fn get_financial_statements(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Query(params): Query<FinSummaryQuery>,
) -> Result<Json<FinSummaryResponse>, ApiError> {
    crate::handlers::jquants::get_fin_summary(State(state), auth_user, Query(params)).await
}

// ---------------------------------------------------------------------------
