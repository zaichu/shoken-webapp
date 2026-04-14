use crate::errors::{ApiError, ErrorResponse};
use crate::extractors::auth::AuthenticatedUser;
use crate::models::jquants::{FinSummaryQuery, FinSummaryResponse};
use crate::services::jquants::{JQuantsService, FIN_SUMMARY_URL};
use crate::state::AppState;
use axum::{
    extract::{Query, State},
    response::Json,
    routing::get,
    Router,
};

pub fn jquants_routes() -> Router<AppState> {
    Router::new().route("/jquants/fins/summary", get(get_fin_summary))
}

/// 決算サマリーを取得（J-Quants API V2）
#[utoipa::path(
    get,
    path = "/jquants/fins/summary",
    operation_id = "jquants_fin_summary",
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
pub async fn get_fin_summary(
    State(state): State<AppState>,
    _auth_user: AuthenticatedUser,
    Query(params): Query<FinSummaryQuery>,
) -> Result<Json<FinSummaryResponse>, ApiError> {
    tracing::info!("決算サマリー取得パラメータ: {:?}", params);

    let api_key =
        state.secrets.jquants_api_key.as_ref().ok_or_else(|| {
            ApiError::ApiError("JQUANTS_API_KEY が設定されていません".to_string())
        })?;

    let response =
        JQuantsService::get_fin_summary(&state.client, params, api_key, FIN_SUMMARY_URL).await?;
    Ok(Json(response))
}
