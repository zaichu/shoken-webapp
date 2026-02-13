use crate::errors::ApiError;
use crate::models::common::BulkCreateResponse;
use crate::models::dividend::{CreateDividendRequest, Dividend};
use sqlx::PgPool;
use std::time::Instant;
use tracing::info;
use uuid::Uuid;

/// 認証ユーザーの配当金一覧を取得
pub async fn list(pool: &PgPool, user_id: Uuid) -> Result<Vec<Dividend>, ApiError> {
    info!("[dividend.list] リクエスト受信");
    let dividends = sqlx::query_as::<_, Dividend>(
        r#"
        SELECT id, user_id, settlement_date, product, account, security_code, security_name,
               unit_price, shares, dividends_before_tax, taxes, net_amount_received,
               created_at, updated_at
        FROM dividends
        WHERE user_id = $1
        ORDER BY settlement_date DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(dividends)
}

/// 配当金を一括追加（重複はスキップ）
pub async fn bulk_create(
    pool: &PgPool,
    user_id: Uuid,
    items: &[CreateDividendRequest],
) -> Result<BulkCreateResponse, ApiError> {
    let total = items.len();
    info!("[dividend.bulk_create] リクエスト受信: {}件", total);
    let start = Instant::now();

    if items.is_empty() {
        return Ok(BulkCreateResponse {
            inserted: 0,
            skipped: 0,
        });
    }

    // 各フィールドを配列に変換
    let user_ids: Vec<Uuid> = vec![user_id; total];
    let settlement_dates: Vec<chrono::NaiveDate> =
        items.iter().map(|i| i.settlement_date).collect();
    let products: Vec<&str> = items.iter().map(|i| i.product.as_str()).collect();
    let accounts: Vec<&str> = items.iter().map(|i| i.account.as_str()).collect();
    let security_codes: Vec<&str> = items.iter().map(|i| i.security_code.as_str()).collect();
    let security_names: Vec<&str> = items.iter().map(|i| i.security_name.as_str()).collect();
    let unit_prices: Vec<f64> = items.iter().map(|i| i.unit_price).collect();
    let shares: Vec<f64> = items.iter().map(|i| i.shares).collect();
    let dividends_before_taxes: Vec<f64> = items.iter().map(|i| i.dividends_before_tax).collect();
    let taxes: Vec<f64> = items.iter().map(|i| i.taxes).collect();
    let net_amounts: Vec<f64> = items.iter().map(|i| i.net_amount_received).collect();

    // UNNESTを使ったバルクINSERT（1回のクエリで全件挿入）
    let result = sqlx::query(
        r#"
        INSERT INTO dividends (user_id, settlement_date, product, account, security_code,
                               security_name, unit_price, shares, dividends_before_tax,
                               taxes, net_amount_received)
        SELECT * FROM UNNEST(
            $1::uuid[], $2::date[], $3::text[], $4::text[], $5::text[],
            $6::text[], $7::float8[], $8::float8[], $9::float8[],
            $10::float8[], $11::float8[]
        )
        ON CONFLICT (user_id, settlement_date, security_code, shares, dividends_before_tax)
        DO NOTHING
        "#,
    )
    .bind(&user_ids)
    .bind(&settlement_dates)
    .bind(&products)
    .bind(&accounts)
    .bind(&security_codes)
    .bind(&security_names)
    .bind(&unit_prices)
    .bind(&shares)
    .bind(&dividends_before_taxes)
    .bind(&taxes)
    .bind(&net_amounts)
    .execute(pool)
    .await?;

    let inserted = result.rows_affected() as usize;
    let skipped = total - inserted;
    let elapsed = start.elapsed();

    info!(
        "[dividend.bulk_create] 完了: inserted={}, skipped={}, 処理時間={:.2}ms",
        inserted,
        skipped,
        elapsed.as_secs_f64() * 1000.0
    );
    Ok(BulkCreateResponse { inserted, skipped })
}

/// 認証ユーザーの配当金を全削除
pub async fn delete_all(pool: &PgPool, user_id: Uuid) -> Result<u64, ApiError> {
    info!("[dividend.delete_all] リクエスト受信");
    let result = sqlx::query("DELETE FROM dividends WHERE user_id = $1")
        .bind(user_id)
        .execute(pool)
        .await?;

    let deleted = result.rows_affected();
    info!("[dividend.delete_all] 完了: {}件削除", deleted);
    Ok(deleted)
}
