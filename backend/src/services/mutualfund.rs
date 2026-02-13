use crate::errors::ApiError;
use crate::models::common::BulkCreateResponse;
use crate::models::mutualfund::{CreateMutualfundRequest, Mutualfund};
use sqlx::PgPool;
use std::time::Instant;
use tracing::info;
use uuid::Uuid;

/// 認証ユーザーの投資信託一覧を取得
pub async fn list(pool: &PgPool, user_id: Uuid) -> Result<Vec<Mutualfund>, ApiError> {
    info!("[mutualfund.list] リクエスト受信");
    let funds = sqlx::query_as::<_, Mutualfund>(
        r#"
        SELECT id, user_id, trade_date, settlement_date, fund_name, dividends, account,
               shares, exchange_rate, cancellation_unit_price_yen, cancellation_amount_yen,
               average_acquisition_price_yen, realized_profit_and_loss, taxes,
               realized_profit_and_loss_after_tax, created_at, updated_at
        FROM mutualfunds
        WHERE user_id = $1
        ORDER BY trade_date DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(funds)
}

/// 投資信託を一括追加（重複はスキップ）
pub async fn bulk_create(
    pool: &PgPool,
    user_id: Uuid,
    items: &[CreateMutualfundRequest],
) -> Result<BulkCreateResponse, ApiError> {
    let total = items.len();
    info!("[mutualfund.bulk_create] リクエスト受信: {}件", total);
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
    let fund_names: Vec<&str> = items.iter().map(|i| i.fund_name.as_str()).collect();
    let dividends: Vec<Option<&str>> = items.iter().map(|i| i.dividends.as_deref()).collect();
    let accounts: Vec<&str> = items.iter().map(|i| i.account.as_str()).collect();
    let shares: Vec<f64> = items.iter().map(|i| i.shares).collect();
    let exchange_rates: Vec<f64> = items.iter().map(|i| i.exchange_rate).collect();
    let cancellation_unit_prices: Vec<f64> = items
        .iter()
        .map(|i| i.cancellation_unit_price_yen)
        .collect();
    let cancellation_amounts: Vec<f64> = items.iter().map(|i| i.cancellation_amount_yen).collect();
    let avg_acquisition_prices: Vec<f64> = items
        .iter()
        .map(|i| i.average_acquisition_price_yen)
        .collect();
    let realized_pls: Vec<f64> = items.iter().map(|i| i.realized_profit_and_loss).collect();
    let taxes: Vec<f64> = items.iter().map(|i| i.taxes).collect();
    let realized_pls_after_tax: Vec<f64> = items
        .iter()
        .map(|i| i.realized_profit_and_loss_after_tax)
        .collect();

    // UNNESTを使ったバルクINSERT（1回のクエリで全件挿入）
    let result = sqlx::query(
        r#"
        INSERT INTO mutualfunds (user_id, trade_date, settlement_date, fund_name, dividends,
                                 account, shares, exchange_rate, cancellation_unit_price_yen,
                                 cancellation_amount_yen, average_acquisition_price_yen,
                                 realized_profit_and_loss, taxes, realized_profit_and_loss_after_tax)
        SELECT * FROM UNNEST(
            $1::uuid[], $2::date[], $3::date[], $4::text[], $5::text[],
            $6::text[], $7::float8[], $8::float8[], $9::float8[],
            $10::float8[], $11::float8[], $12::float8[], $13::float8[], $14::float8[]
        )
        ON CONFLICT (user_id, trade_date, fund_name, shares, cancellation_amount_yen)
        DO NOTHING
        "#,
    )
    .bind(&user_ids)
    .bind(&trade_dates)
    .bind(&settlement_dates)
    .bind(&fund_names)
    .bind(&dividends)
    .bind(&accounts)
    .bind(&shares)
    .bind(&exchange_rates)
    .bind(&cancellation_unit_prices)
    .bind(&cancellation_amounts)
    .bind(&avg_acquisition_prices)
    .bind(&realized_pls)
    .bind(&taxes)
    .bind(&realized_pls_after_tax)
    .execute(pool)
    .await?;

    let inserted = result.rows_affected() as usize;
    let skipped = total - inserted;
    let elapsed = start.elapsed();

    info!(
        "[mutualfund.bulk_create] 完了: inserted={}, skipped={}, 処理時間={:.2}ms",
        inserted,
        skipped,
        elapsed.as_secs_f64() * 1000.0
    );
    Ok(BulkCreateResponse { inserted, skipped })
}

/// 認証ユーザーの投資信託を全削除
pub async fn delete_all(pool: &PgPool, user_id: Uuid) -> Result<u64, ApiError> {
    info!("[mutualfund.delete_all] リクエスト受信");
    let result = sqlx::query("DELETE FROM mutualfunds WHERE user_id = $1")
        .bind(user_id)
        .execute(pool)
        .await?;

    let deleted = result.rows_affected();
    info!("[mutualfund.delete_all] 完了: {}件削除", deleted);
    Ok(deleted)
}
