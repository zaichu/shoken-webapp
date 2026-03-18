use crate::errors::ApiError;
use crate::models::asset_balance::{AssetBalance, CreateAssetBalanceRequest};
use crate::models::common::BulkCreateResponse;
use crate::models::csv_import::{CsvPreviewResponse, CsvUploadResponse};
use crate::services::asset_balance_csv::parse_asset_balance_csv;
use crate::services::csv_import::finish_csv_upload;
use crate::services::shared::BulkTimer;
use rust_decimal::Decimal;
use sqlx::PgPool;
use tracing::info;
use uuid::Uuid;

/// 認証ユーザーの保有銘柄一覧を取得
pub async fn list(pool: &PgPool, user_id: Uuid) -> Result<Vec<AssetBalance>, ApiError> {
    info!("[asset_balance.list] リクエスト受信");
    let balances = sqlx::query_as::<_, AssetBalance>(
        r#"
        SELECT id, user_id, security_code, security_name, shares, executing_shares,
               average_purchase_price, total_purchase_amount, current_price,
               daily_change, market_value, profit_loss_rate, created_at, updated_at
        FROM asset_balances
        WHERE user_id = $1
        ORDER BY security_code
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(balances)
}

/// 保有銘柄を一括登録（既存データを全削除してから挿入）
pub async fn bulk_create(
    pool: &PgPool,
    user_id: Uuid,
    items: &[CreateAssetBalanceRequest],
) -> Result<BulkCreateResponse, ApiError> {
    let total = items.len();
    let timer = BulkTimer::new("asset_balance", total);

    // 各フィールドを配列に変換
    let user_ids: Vec<Uuid> = vec![user_id; total];
    let security_codes: Vec<&str> = items.iter().map(|i| i.security_code.as_str()).collect();
    let security_names: Vec<&str> = items.iter().map(|i| i.security_name.as_str()).collect();
    let shares: Vec<Decimal> = items.iter().map(|i| i.shares).collect();
    let executing_shares: Vec<Decimal> = items.iter().map(|i| i.executing_shares).collect();
    let average_purchase_prices: Vec<Decimal> =
        items.iter().map(|i| i.average_purchase_price).collect();
    let total_purchase_amounts: Vec<Decimal> =
        items.iter().map(|i| i.total_purchase_amount).collect();
    let current_prices: Vec<Decimal> = items.iter().map(|i| i.current_price).collect();
    let daily_changes: Vec<Decimal> = items.iter().map(|i| i.daily_change).collect();
    let market_values: Vec<Decimal> = items.iter().map(|i| i.market_value).collect();
    let profit_loss_rates: Vec<Decimal> = items.iter().map(|i| i.profit_loss_rate).collect();

    // トランザクション内で全削除 → 全件挿入（スナップショット置き換え）
    let mut tx = pool.begin().await?;

    // ユーザー単位のadvisory lockで並行bulk_createを直列化（READ COMMITTEDでのA∪B混入を防止）
    sqlx::query("SELECT pg_advisory_xact_lock(hashtext($1::text))")
        .bind(user_id.to_string())
        .execute(&mut *tx)
        .await?;

    sqlx::query("DELETE FROM asset_balances WHERE user_id = $1")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    if !items.is_empty() {
        sqlx::query(
            r#"
            INSERT INTO asset_balances (user_id, security_code, security_name, shares, executing_shares,
                                        average_purchase_price, total_purchase_amount, current_price,
                                        daily_change, market_value, profit_loss_rate)
            SELECT * FROM UNNEST(
                $1::uuid[], $2::text[], $3::text[], $4::numeric[], $5::numeric[],
                $6::numeric[], $7::numeric[], $8::numeric[], $9::numeric[], $10::numeric[], $11::numeric[]
            )
            "#,
        )
        .bind(&user_ids)
        .bind(&security_codes)
        .bind(&security_names)
        .bind(&shares)
        .bind(&executing_shares)
        .bind(&average_purchase_prices)
        .bind(&total_purchase_amounts)
        .bind(&current_prices)
        .bind(&daily_changes)
        .bind(&market_values)
        .bind(&profit_loss_rates)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(timer.finish(total))
}

/// CSV bytes をパースしてプレビュー情報を返す（DB 書き込みなし）
/// 現在の取込対象形式では、先頭6行はメタデータのためスキップ
pub fn preview_csv(bytes: &[u8]) -> Result<CsvPreviewResponse, ApiError> {
    let (items, errors) = parse_asset_balance_csv(bytes)?;
    let rows = items
        .iter()
        .map(|item| serde_json::to_value(item).unwrap_or(serde_json::Value::Null))
        .collect();
    Ok(CsvPreviewResponse {
        total_rows: items.len() + errors.len(),
        valid_rows: items.len(),
        errors,
        rows,
    })
}

/// CSV bytes をパースして保有銘柄を一括登録
pub async fn upload_csv(
    pool: &PgPool,
    user_id: Uuid,
    bytes: &[u8],
) -> Result<CsvUploadResponse, ApiError> {
    let (items, errors) = parse_asset_balance_csv(bytes)?;
    let result = bulk_create(pool, user_id, &items).await?;
    Ok(finish_csv_upload(result, errors))
}

/// 認証ユーザーの保有銘柄を全削除
pub async fn delete_all(pool: &PgPool, user_id: Uuid) -> Result<u64, ApiError> {
    crate::services::shared::delete_all_for_user(pool, user_id, "asset_balances", "asset_balance")
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preview_csv_valid() {
        let csv = [
            "■現在の評価額合計［円］,,\"9,474,000\"",
            "■評価損益合計,前日比［円］,\"288,000\"",
            ",前月比［円］,\"-120,000\"",
            ",評価損益［円］,\"2,005,900\"",
            "■特定口座",
            "",
            "銘柄コード,銘柄名,保有数量［株］,執行中［株］,(内訳　通常数量[株]),(内訳　積立数量[株]),平均取得価額［円］,取得総額［円］,現在値［円］,現在値（前日比）［円］,時価評価額［円］,評価損益［％］",
            "\"1605\",\"ＩＮＰＥＸ\",\"200\",\"0\",\"200\",\"0\",\"2,355.00\",\"471,000\",\"3,685.0\",\"65.0\",\"737,000\",\"56.47\"",
            "\"7974\",\"任天堂\",\"1,000\",\"0\",\"1,000\",\"0\",\"5,997.60\",\"5,997,600\",\"8,737.0\",\"223.0\",\"8,737,000\",\"45.67\"",
            ",,,,,,特定口座合計,\"11,245,249\",,,\"14,517,240\",\"29.09\"",
        ]
        .join("\n");

        let preview = preview_csv(csv.as_bytes()).unwrap();

        assert_eq!(preview.total_rows, 2);
        assert_eq!(preview.valid_rows, 2);
        assert!(
            preview.errors.is_empty(),
            "unexpected errors: {:?}",
            preview.errors
        );
        assert_eq!(preview.rows.len(), 2);
    }
}
