use crate::{
    errors::ApiError,
    extractors::auth::AuthenticatedUser,
    state::AppState,
};
use axum::{extract::State, http::StatusCode, response::{IntoResponse, Json}};
use serde::Serialize;
use utoipa::ToSchema;

/// CSV種類ごとの統計情報
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CsvKindStat {
    /// 種類識別子 (assetbalance, fundbalance, dividend, domesticstock, mutualfund)
    pub kind: String,
    /// 表示名
    pub label: String,
    /// 保存済み件数
    pub count: i64,
    /// 最終取込日時 (ISO 8601, UTC)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_imported_at: Option<String>,
}

/// CSV種類統計のレスポンス
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CsvKindStatsResponse {
    pub kinds: Vec<CsvKindStat>,
}

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
    let user_id = auth_user.id();

    // 各テーブルから件数と最新 created_at を並行取得
    let asset_balance = query_kind_stat(&state.pool, user_id, "asset_balances").await?;
    let fund_balance = query_kind_stat(&state.pool, user_id, "fund_balances").await?;
    let dividend = query_kind_stat(&state.pool, user_id, "dividends").await?;
    let domestic_stock = query_kind_stat(&state.pool, user_id, "domestic_stock_transactions").await?;
    let mutual_fund = query_kind_stat(&state.pool, user_id, "mutual_fund_transactions").await?;

    let kinds = vec![
        CsvKindStat {
            kind: "assetbalance".to_string(),
            label: "資産管理(国内株式)".to_string(),
            count: asset_balance.0,
            latest_imported_at: asset_balance.1,
        },
        CsvKindStat {
            kind: "fundbalance".to_string(),
            label: "資産管理(投資信託)".to_string(),
            count: fund_balance.0,
            latest_imported_at: fund_balance.1,
        },
        CsvKindStat {
            kind: "dividend".to_string(),
            label: "取引明細(配当金)".to_string(),
            count: dividend.0,
            latest_imported_at: dividend.1,
        },
        CsvKindStat {
            kind: "domesticstock".to_string(),
            label: "取引明細(国内株式)".to_string(),
            count: domestic_stock.0,
            latest_imported_at: domestic_stock.1,
        },
        CsvKindStat {
            kind: "mutualfund".to_string(),
            label: "取引明細(投資信託)".to_string(),
            count: mutual_fund.0,
            latest_imported_at: mutual_fund.1,
        },
    ];

    Ok((StatusCode::OK, Json(CsvKindStatsResponse { kinds })))
}

async fn query_kind_stat(
    pool: &crate::db::Db,
    user_id: shared::value::UserId,
    table: &str,
) -> Result<(i64, Option<String>), ApiError> {
    let query = format!(
        "SELECT COUNT(*) as count, MAX(created_at) as latest FROM {} WHERE user_id = $1",
        table
    );
    // タプル (count, latest) として取得
    let row: (i64, Option<chrono::DateTime<chrono::Utc>>) = crate::db::query_as(&query, vec![user_id.into()])
        .fetch_one(pool)
        .await?;

    Ok((
        row.0,
        row.1.map(|dt| dt.to_rfc3339()),
    ))
}