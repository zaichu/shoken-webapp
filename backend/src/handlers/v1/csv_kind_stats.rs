use crate::{errors::ApiError, extractors::auth::AuthenticatedUser, state::AppState};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};

pub use shared::csv_kind_stats::{CsvKindStat, CsvKindStatsResponse};

/// GET /api/v1/csv-kinds/stats - CSV管理画面用の種類ごと統計を取得
#[utoipa::path(
    get,
    path = "/api/v1/csv-kinds/stats",
    tag = "CSV管理",
    security(
        ("cookieAuth" = [])
    ),
    responses(
        (status = 200, description = "CSV種類ごとの件数と最終取込日時", body = CsvKindStatsResponse),
        (status = 401, description = "ログインが必要です"),
    )
)]
pub async fn csv_kind_stats(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<impl IntoResponse, ApiError> {
    let stats =
        crate::services::csv::stats::collect_csv_kind_stats(&state.pool, auth_user.id()).await?;
    Ok((StatusCode::OK, Json(stats)))
}
