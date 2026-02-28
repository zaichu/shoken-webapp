use crate::errors::ApiError;
use crate::models::common::BulkCreateResponse;
use crate::models::csv_import::{CsvRowError, CsvUploadResponse};
use crate::models::dividend::{CreateDividendRequest, Dividend};
use crate::services::csv_import::{decode_bytes, parse_date, parse_number};
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

/// CSV バイト列から配当金をパースして一括挿入
pub async fn upload_csv(
    pool: &PgPool,
    user_id: Uuid,
    bytes: &[u8],
) -> Result<CsvUploadResponse, ApiError> {
    let content = decode_bytes(bytes);
    let mut reader = csv::Reader::from_reader(content.as_bytes());

    // ヘッダー名 → 列インデックスのマップを構築
    let header_map: std::collections::HashMap<String, usize> = reader
        .headers()
        .map_err(|e| {
            ApiError::ValidationError(format!("CSVヘッダーの読み込みに失敗しました: {}", e))
        })?
        .iter()
        .enumerate()
        .map(|(i, h)| (h.trim().to_string(), i))
        .collect();

    let mut items: Vec<CreateDividendRequest> = Vec::new();
    let mut errors: Vec<CsvRowError> = Vec::new();

    for (row_idx, result) in reader.records().enumerate() {
        let row_num = row_idx + 1;
        let record = match result {
            Ok(r) => r,
            Err(e) => {
                errors.push(CsvRowError {
                    row: row_num,
                    message: format!("CSV行の読み込みに失敗しました: {}", e),
                });
                continue;
            }
        };

        let get = |name: &str| -> &str {
            header_map
                .get(name)
                .and_then(|&i| record.get(i))
                .unwrap_or("")
        };

        // 必須文字列フィールド: 空の場合はエラーとして行をスキップ
        macro_rules! require_str {
            ($col:expr) => {{
                let val = get($col);
                if val.is_empty() {
                    errors.push(CsvRowError {
                        row: row_num,
                        message: format!("必須列 '{}' が空または存在しません", $col),
                    });
                    continue;
                }
                val.to_string()
            }};
        }

        let settlement_date = match parse_date(get("入金日")) {
            Ok(d) => d,
            Err(e) => {
                errors.push(CsvRowError {
                    row: row_num,
                    message: format!("入金日: {}", e),
                });
                continue;
            }
        };

        macro_rules! parse_num {
            ($col:expr) => {{
                let raw = get($col);
                if raw.is_empty() {
                    errors.push(CsvRowError {
                        row: row_num,
                        message: format!("必須列 '{}' が空または存在しません", $col),
                    });
                    continue;
                }
                match parse_number(raw) {
                    Ok(v) => v,
                    Err(e) => {
                        errors.push(CsvRowError {
                            row: row_num,
                            message: format!("{}: {}", $col, e),
                        });
                        continue;
                    }
                }
            }};
        }

        items.push(CreateDividendRequest {
            settlement_date,
            product: require_str!("商品"),
            account: require_str!("口座"),
            security_code: require_str!("銘柄コード"),
            security_name: require_str!("銘柄"),
            unit_price: parse_num!("単価[円/現地通貨]"),
            shares: parse_num!("数量[株/口]"),
            dividends_before_tax: parse_num!("配当・分配金合計（税引前）[円/現地通貨]"),
            taxes: parse_num!("税額合計[円/現地通貨]"),
            net_amount_received: parse_num!("受取金額[円/現地通貨]"),
        });
    }

    let result = bulk_create(pool, user_id, &items).await?;
    Ok(CsvUploadResponse {
        inserted: result.inserted,
        skipped: result.skipped,
        errors,
    })
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
