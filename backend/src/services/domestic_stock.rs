use crate::errors::ApiError;
use crate::models::common::BulkCreateResponse;
use crate::models::domestic_stock::{CreateDomesticStockRequest, DomesticStock};
use sqlx::PgPool;
use std::time::Instant;
use tracing::info;
use uuid::Uuid;

/// 認証ユーザーの国内株式取引一覧を取得
pub async fn list(pool: &PgPool, user_id: Uuid) -> Result<Vec<DomesticStock>, ApiError> {
    info!("[domestic_stock.list] リクエスト受信");
    let stocks = sqlx::query_as::<_, DomesticStock>(
        r#"
        SELECT id, user_id, trade_date, settlement_date, security_code, security_name,
               account, shares, asked_price, proceeds, purchase_price,
               realized_profit_and_loss, taxes, realized_profit_and_loss_after_tax,
               created_at, updated_at
        FROM domestic_stocks
        WHERE user_id = $1
        ORDER BY trade_date DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(stocks)
}

/// 国内株式取引を一括追加（全件挿入）
pub async fn bulk_create(
    pool: &PgPool,
    user_id: Uuid,
    items: &[CreateDomesticStockRequest],
) -> Result<BulkCreateResponse, ApiError> {
    let total = items.len();
    info!("[domestic_stock.bulk_create] リクエスト受信: {}件", total);
    let start = Instant::now();

    if items.is_empty() {
        return Ok(BulkCreateResponse {
            inserted: 0,
            skipped: 0,
        });
    }

    // 各フィールドを配列に変換
    let user_ids: Vec<Uuid> = vec![user_id; total];
    let trade_dates: Vec<chrono::NaiveDate> = items.iter().map(|i| i.trade_date).collect();
    let settlement_dates: Vec<chrono::NaiveDate> =
        items.iter().map(|i| i.settlement_date).collect();
    let security_codes: Vec<&str> = items.iter().map(|i| i.security_code.as_str()).collect();
    let security_names: Vec<&str> = items.iter().map(|i| i.security_name.as_str()).collect();
    let accounts: Vec<&str> = items.iter().map(|i| i.account.as_str()).collect();
    let shares: Vec<f64> = items.iter().map(|i| i.shares).collect();
    let asked_prices: Vec<f64> = items.iter().map(|i| i.asked_price).collect();
    let proceeds: Vec<f64> = items.iter().map(|i| i.proceeds).collect();
    let purchase_prices: Vec<f64> = items.iter().map(|i| i.purchase_price).collect();
    let realized_pls: Vec<f64> = items.iter().map(|i| i.realized_profit_and_loss).collect();
    let taxes: Vec<f64> = items.iter().map(|i| i.taxes).collect();
    let realized_pls_after_tax: Vec<f64> = items
        .iter()
        .map(|i| i.realized_profit_and_loss_after_tax)
        .collect();

    // UNNESTを使ったバルクINSERT（1回のクエリで全件挿入、ユニーク制約なしで重複行も全件保存）
    let result = sqlx::query(
        r#"
        INSERT INTO domestic_stocks (user_id, trade_date, settlement_date, security_code,
                                     security_name, account, shares, asked_price, proceeds,
                                     purchase_price, realized_profit_and_loss, taxes,
                                     realized_profit_and_loss_after_tax)
        SELECT * FROM UNNEST(
            $1::uuid[], $2::date[], $3::date[], $4::text[],
            $5::text[], $6::text[], $7::float8[], $8::float8[], $9::float8[],
            $10::float8[], $11::float8[], $12::float8[], $13::float8[]
        )
        "#,
    )
    .bind(&user_ids)
    .bind(&trade_dates)
    .bind(&settlement_dates)
    .bind(&security_codes)
    .bind(&security_names)
    .bind(&accounts)
    .bind(&shares)
    .bind(&asked_prices)
    .bind(&proceeds)
    .bind(&purchase_prices)
    .bind(&realized_pls)
    .bind(&taxes)
    .bind(&realized_pls_after_tax)
    .execute(pool)
    .await?;

    let inserted = result.rows_affected() as usize;
    let skipped = total - inserted;
    let elapsed = start.elapsed();

    info!(
        "[domestic_stock.bulk_create] 完了: inserted={}, skipped={}, 処理時間={:.2}ms",
        inserted,
        skipped,
        elapsed.as_secs_f64() * 1000.0
    );
    Ok(BulkCreateResponse { inserted, skipped })
}

/// 認証ユーザーの国内株式取引を全削除
pub async fn delete_all(pool: &PgPool, user_id: Uuid) -> Result<u64, ApiError> {
    info!("[domestic_stock.delete_all] リクエスト受信");
    let result = sqlx::query("DELETE FROM domestic_stocks WHERE user_id = $1")
        .bind(user_id)
        .execute(pool)
        .await?;

    let deleted = result.rows_affected();
    info!("[domestic_stock.delete_all] 完了: {}件削除", deleted);
    Ok(deleted)
}
