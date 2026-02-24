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
/// 同一内容の行が複数ある場合も全件保存する。
/// 同一CSVを再アップロードした場合は content_hash + occurrence_index の
/// ユニーク制約により重複行がスキップされる。
/// content_hash と occurrence_index はSQL側で算出し、Rust/SQL間の表現差異を排除する。
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

    // UNNESTを使ったバルクINSERT（1回のクエリで全件挿入）
    // content_hash はPostgreSQL md5関数で算出（migration backfillと同一実装）
    // occurrence_index はWITH ORDINALITYで入力順を保持しROW_NUMBERで連番付与
    // ON CONFLICT により、同一ハッシュ・同一出現回数の行はスキップ（再アップロード防止）
    let result = sqlx::query(
        r#"
        INSERT INTO domestic_stocks (user_id, trade_date, settlement_date, security_code,
                                     security_name, account, shares, asked_price, proceeds,
                                     purchase_price, realized_profit_and_loss, taxes,
                                     realized_profit_and_loss_after_tax,
                                     content_hash, occurrence_index)
        SELECT
            user_id, trade_date, settlement_date, security_code,
            security_name, account, shares, asked_price, proceeds,
            purchase_price, realized_profit_and_loss, taxes,
            realized_profit_and_loss_after_tax,
            content_hash,
            ROW_NUMBER() OVER (PARTITION BY user_id, content_hash ORDER BY ordinality)::int4
        FROM (
            SELECT
                user_id, trade_date, settlement_date, security_code,
                security_name, account, shares, asked_price, proceeds,
                purchase_price, realized_profit_and_loss, taxes,
                realized_profit_and_loss_after_tax,
                ordinality,
                md5(
                    trade_date::text || '|' || settlement_date::text || '|' ||
                    security_code || '|' || security_name || '|' || account || '|' ||
                    shares::text || '|' || asked_price::text || '|' || proceeds::text || '|' ||
                    purchase_price::text || '|' || realized_profit_and_loss::text || '|' ||
                    taxes::text || '|' || realized_profit_and_loss_after_tax::text
                ) AS content_hash
            FROM UNNEST(
                $1::uuid[], $2::date[], $3::date[], $4::text[],
                $5::text[], $6::text[], $7::float8[], $8::float8[], $9::float8[],
                $10::float8[], $11::float8[], $12::float8[], $13::float8[]
            ) WITH ORDINALITY AS t(user_id, trade_date, settlement_date, security_code,
                   security_name, account, shares, asked_price, proceeds,
                   purchase_price, realized_profit_and_loss, taxes,
                   realized_profit_and_loss_after_tax, ordinality)
        ) subq
        ON CONFLICT (user_id, content_hash, occurrence_index) DO NOTHING
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn make_test_item() -> CreateDomesticStockRequest {
        CreateDomesticStockRequest {
            trade_date: NaiveDate::from_ymd_opt(2026, 2, 12).unwrap(),
            settlement_date: NaiveDate::from_ymd_opt(2026, 2, 16).unwrap(),
            security_code: "9508".to_string(),
            security_name: "九州電力".to_string(),
            account: "特定".to_string(),
            shares: 100.0,
            asked_price: 1880.0,
            proceeds: 188000.0,
            purchase_price: 1770.0,
            realized_profit_and_loss: 11000.0,
            taxes: 2234.0,
            realized_profit_and_loss_after_tax: 8766.0,
        }
    }

    /// 1回目アップロード → 全件挿入、2回目同一CSV → 全件スキップ（再アップロード防止）
    #[tokio::test]
    async fn test_bulk_create_reupload_deduplication() {
        use testcontainers::runners::AsyncRunner;
        use testcontainers_modules::postgres::Postgres;

        let container = Postgres::default().start().await.unwrap();
        let url = format!(
            "postgres://postgres:postgres@{}:{}/postgres",
            container.get_host().await.unwrap(),
            container.get_host_port_ipv4(5432).await.unwrap(),
        );
        let pool = sqlx::PgPool::connect(&url).await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        // FK制約のためユーザーを事前作成
        let user_id = Uuid::new_v4();
        sqlx::query("INSERT INTO users (id, google_id, email) VALUES ($1, $2, $3)")
            .bind(user_id)
            .bind(format!("test_google_{user_id}"))
            .bind(format!("test_{user_id}@example.com"))
            .execute(&pool)
            .await
            .unwrap();

        let items = vec![make_test_item(); 5];

        // 1回目: 全件挿入
        let first = bulk_create(&pool, user_id, &items).await.unwrap();
        assert_eq!(first.inserted, 5);
        assert_eq!(first.skipped, 0);

        // 2回目（同一CSV再アップロード）: 全件スキップ
        let second = bulk_create(&pool, user_id, &items).await.unwrap();
        assert_eq!(second.inserted, 0);
        assert_eq!(second.skipped, 5);
    }
}
