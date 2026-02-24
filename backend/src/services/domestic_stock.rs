use crate::errors::ApiError;
use crate::models::common::BulkCreateResponse;
use crate::models::domestic_stock::{CreateDomesticStockRequest, DomesticStock};
use sqlx::PgPool;
use std::collections::HashMap;
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

/// 取引内容から MD5 ハッシュを算出する
/// PostgreSQL migration の md5(fields::text || ...) と同一フォーマットで連結する
fn compute_content_hash(item: &CreateDomesticStockRequest) -> String {
    let input = format!(
        "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
        item.trade_date,
        item.settlement_date,
        item.security_code,
        item.security_name,
        item.account,
        item.shares,
        item.asked_price,
        item.proceeds,
        item.purchase_price,
        item.realized_profit_and_loss,
        item.taxes,
        item.realized_profit_and_loss_after_tax,
    );
    format!("{:x}", md5::compute(input))
}

/// 国内株式取引を一括追加（全件挿入）
/// 同一内容の行が複数ある場合も全件保存する。
/// 同一CSVを再アップロードした場合は content_hash + occurrence_index の
/// ユニーク制約により重複行がスキップされる。
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

    // ハッシュ + 出現回数を計算
    // 同一ハッシュが N 件あれば occurrence_index = 1, 2, ..., N を割り当てる
    let mut hash_occurrences: HashMap<String, i32> = HashMap::new();
    let mut content_hashes: Vec<String> = Vec::with_capacity(total);
    let mut occurrence_indices: Vec<i32> = Vec::with_capacity(total);

    for item in items {
        let hash = compute_content_hash(item);
        let count = hash_occurrences.entry(hash.clone()).or_insert(0);
        *count += 1;
        occurrence_indices.push(*count);
        content_hashes.push(hash);
    }

    // UNNESTを使ったバルクINSERT（1回のクエリで全件挿入）
    // ON CONFLICT により、同一ハッシュ・同一出現回数の行はスキップ（再アップロード防止）
    let result = sqlx::query(
        r#"
        INSERT INTO domestic_stocks (user_id, trade_date, settlement_date, security_code,
                                     security_name, account, shares, asked_price, proceeds,
                                     purchase_price, realized_profit_and_loss, taxes,
                                     realized_profit_and_loss_after_tax,
                                     content_hash, occurrence_index)
        SELECT * FROM UNNEST(
            $1::uuid[], $2::date[], $3::date[], $4::text[],
            $5::text[], $6::text[], $7::float8[], $8::float8[], $9::float8[],
            $10::float8[], $11::float8[], $12::float8[], $13::float8[],
            $14::text[], $15::int4[]
        )
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
    .bind(&content_hashes)
    .bind(&occurrence_indices)
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

    fn make_item(
        trade_date: NaiveDate,
        security_code: &str,
        shares: f64,
        proceeds: f64,
    ) -> CreateDomesticStockRequest {
        CreateDomesticStockRequest {
            trade_date,
            settlement_date: trade_date,
            security_code: security_code.to_string(),
            security_name: "テスト株式".to_string(),
            account: "特定".to_string(),
            shares,
            asked_price: proceeds / shares,
            proceeds,
            purchase_price: proceeds / shares - 10.0,
            realized_profit_and_loss: 10.0 * shares,
            taxes: 0.0,
            realized_profit_and_loss_after_tax: 10.0 * shares,
        }
    }

    #[test]
    fn test_compute_content_hash_deterministic() {
        // 同一内容は常に同じハッシュを返す
        let item = make_item(
            NaiveDate::from_ymd_opt(2026, 2, 12).unwrap(),
            "9508",
            100.0,
            188000.0,
        );
        let h1 = compute_content_hash(&item);
        let h2 = compute_content_hash(&item);
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 32); // MD5 は 32 文字の hex
    }

    #[test]
    fn test_compute_content_hash_differs_by_field() {
        // フィールドが異なれば別のハッシュになる
        let d = NaiveDate::from_ymd_opt(2026, 2, 12).unwrap();
        let a = make_item(d, "9508", 100.0, 188000.0);
        let b = make_item(d, "9509", 100.0, 188000.0); // security_code だけ違う
        assert_ne!(compute_content_hash(&a), compute_content_hash(&b));
    }

    #[test]
    fn test_occurrence_index_assignment_identical_rows() {
        // 5件の同一行 → occurrence_index = 1, 2, 3, 4, 5
        let d = NaiveDate::from_ymd_opt(2026, 2, 12).unwrap();
        let item = make_item(d, "9508", 100.0, 188000.0);
        let items = vec![item.clone(), item.clone(), item.clone(), item.clone(), item];

        let mut hash_occurrences: HashMap<String, i32> = HashMap::new();
        let mut occurrence_indices: Vec<i32> = Vec::new();
        for i in &items {
            let hash = compute_content_hash(i);
            let count = hash_occurrences.entry(hash).or_insert(0);
            *count += 1;
            occurrence_indices.push(*count);
        }
        assert_eq!(occurrence_indices, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_occurrence_index_assignment_mixed_items() {
        // A=3件、B=2件 の場合
        let d = NaiveDate::from_ymd_opt(2026, 2, 12).unwrap();
        let a = make_item(d, "9508", 100.0, 188000.0);
        let b = make_item(d, "9509", 200.0, 400000.0);
        let items = vec![a.clone(), b.clone(), a.clone(), b.clone(), a.clone()];

        let mut hash_occurrences: HashMap<String, i32> = HashMap::new();
        let mut occurrence_indices: Vec<i32> = Vec::new();
        for i in &items {
            let hash = compute_content_hash(i);
            let count = hash_occurrences.entry(hash).or_insert(0);
            *count += 1;
            occurrence_indices.push(*count);
        }
        // A:1, B:1, A:2, B:2, A:3
        assert_eq!(occurrence_indices, vec![1, 1, 2, 2, 3]);
    }
}
