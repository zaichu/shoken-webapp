use crate::{errors::ApiError, extractors::auth::AuthenticatedUser, state::AppState};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
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
    let stats = collect_csv_kind_stats(&state.pool, auth_user.id()).await?;
    Ok((StatusCode::OK, Json(stats)))
}

pub async fn collect_csv_kind_stats(
    pool: &crate::db::Db,
    user_id: shared::value::UserId,
) -> Result<CsvKindStatsResponse, ApiError> {
    let asset_balance = query_kind_stat(pool, user_id, "asset_balances").await?;
    let fund_balance = query_kind_stat(pool, user_id, "fund_balances").await?;
    let dividend = query_kind_stat(pool, user_id, "dividends").await?;
    let domestic_stock = query_kind_stat(pool, user_id, "domestic_stocks").await?;
    let mutual_fund = query_kind_stat(pool, user_id, "mutualfunds").await?;

    let mut kinds = vec![
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

    let history: Vec<(String, chrono::DateTime<chrono::Utc>)> = crate::db::query_as(
        "SELECT kind, last_imported_at FROM csv_import_history WHERE user_id = $1",
        vec![user_id.into()],
    )
    .fetch_all(pool)
    .await?;
    for (kind, imported_at) in history {
        if let Some(stat) = kinds.iter_mut().find(|stat| stat.kind == kind) {
            stat.latest_imported_at = Some(imported_at.to_rfc3339());
        }
    }
    // 全削除後も履歴行は残るため、件数0の種類は未取得として扱う
    for stat in &mut kinds {
        if stat.count == 0 {
            stat.latest_imported_at = None;
        }
    }

    Ok(CsvKindStatsResponse { kinds })
}

async fn query_kind_stat(
    pool: &crate::db::Db,
    user_id: shared::value::UserId,
    table: &str,
) -> Result<(i64, Option<String>), ApiError> {
    let query = match table {
        "asset_balances" => {
            "SELECT COUNT(*), MAX(created_at) FROM asset_balances WHERE user_id = $1"
        }
        "fund_balances" => "SELECT COUNT(*), MAX(created_at) FROM fund_balances WHERE user_id = $1",
        "dividends" => "SELECT COUNT(*), MAX(created_at) FROM dividends WHERE user_id = $1",
        "domestic_stocks" => {
            "SELECT COUNT(*), MAX(created_at) FROM domestic_stocks WHERE user_id = $1"
        }
        "mutualfunds" => "SELECT COUNT(*), MAX(created_at) FROM mutualfunds WHERE user_id = $1",
        _ => return Err(ApiError::Internal("不明なCSV種類です")),
    };
    let row: (i64, Option<chrono::DateTime<chrono::Utc>>) =
        crate::db::query_as(query, vec![user_id.into()])
            .fetch_one(pool)
            .await?;

    Ok((row.0, row.1.map(|dt| dt.to_rfc3339())))
}
