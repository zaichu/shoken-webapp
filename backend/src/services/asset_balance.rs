use crate::errors::ApiError;
use crate::models::asset_balance::{AssetBalance, CreateAssetBalanceRequest};
use crate::models::common::BulkCreateResponse;
use crate::models::csv_import::{CsvPreviewResponse, CsvUploadResponse};
use crate::services::csv_import::{finish_csv_upload, parse_csv};
use crate::services::csv_parse::{decode_bytes, parse_number, parse_optional_string};
use csv::StringRecord;
use sqlx::PgPool;
use std::collections::HashMap;
use std::time::Instant;
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
    info!("[asset_balance.bulk_create] リクエスト受信: {}件", total);
    let start = Instant::now();

    // 各フィールドを配列に変換
    let user_ids: Vec<Uuid> = vec![user_id; total];
    let security_codes: Vec<&str> = items.iter().map(|i| i.security_code.as_str()).collect();
    let security_names: Vec<&str> = items.iter().map(|i| i.security_name.as_str()).collect();
    let shares: Vec<f64> = items.iter().map(|i| i.shares).collect();
    let executing_shares: Vec<f64> = items.iter().map(|i| i.executing_shares).collect();
    let average_purchase_prices: Vec<f64> =
        items.iter().map(|i| i.average_purchase_price).collect();
    let total_purchase_amounts: Vec<f64> = items.iter().map(|i| i.total_purchase_amount).collect();
    let current_prices: Vec<f64> = items.iter().map(|i| i.current_price).collect();
    let daily_changes: Vec<f64> = items.iter().map(|i| i.daily_change).collect();
    let market_values: Vec<f64> = items.iter().map(|i| i.market_value).collect();
    let profit_loss_rates: Vec<f64> = items.iter().map(|i| i.profit_loss_rate).collect();

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
                $1::uuid[], $2::text[], $3::text[], $4::float8[], $5::float8[],
                $6::float8[], $7::float8[], $8::float8[], $9::float8[], $10::float8[], $11::float8[]
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

    let elapsed = start.elapsed();
    info!(
        "[asset_balance.bulk_create] 完了: inserted={}, 処理時間={:.2}ms",
        total,
        elapsed.as_secs_f64() * 1000.0
    );
    Ok(BulkCreateResponse {
        inserted: total,
        skipped: 0,
    })
}

/// CSV bytes をパースしてプレビュー情報を返す（DB 書き込みなし）
/// SBI証券形式: 先頭6行はメタデータのためスキップ
pub fn preview_csv(bytes: &[u8]) -> Result<CsvPreviewResponse, ApiError> {
    let content = decode_bytes(bytes);
    let stripped = strip_sbi_header(&content);
    let (items, errors) = parse_csv(stripped.as_bytes(), parse_asset_balance_row)?;
    let items: Vec<_> = items
        .into_iter()
        .filter(|i| !i.security_code.is_empty())
        .collect();
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
    let content = decode_bytes(bytes);
    let stripped = strip_sbi_header(&content);
    let (items, errors) = parse_csv(stripped.as_bytes(), parse_asset_balance_row)?;
    let items: Vec<_> = items
        .into_iter()
        .filter(|i| !i.security_code.is_empty())
        .collect();
    let result = bulk_create(pool, user_id, &items).await?;
    Ok(finish_csv_upload(result, errors))
}

/// SBI証券CSVの先頭6行（メタデータ）をスキップした文字列を返す
fn strip_sbi_header(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    if lines.len() > 6 {
        lines[6..].join("\n")
    } else {
        String::new()
    }
}

/// SBI証券CSVの1行をパースして CreateAssetBalanceRequest に変換
///
/// 数値パース失敗は CsvRowError として返す。
/// ただし以下の列は "-" / 空欄が仕様上ありうるため 0.0 フォールバックを維持する:
///   - 執行中: 執行中の注文がなければ "-" または空欄
///   - 現在値（前日比）: 変動なし時は 0 または "-"
///   - 評価損益（%）: NISA 等で表示されない場合に "-"
fn parse_asset_balance_row(
    record: &StringRecord,
    header_map: &HashMap<String, usize>,
    row_num: usize,
) -> Result<CreateAssetBalanceRequest, crate::models::csv_import::CsvRowError> {
    use crate::models::csv_import::CsvRowError;

    // 必須数値列: 空欄・"-" もエラー。パース失敗も CsvRowError に変換する
    let num = |col: &str| {
        let raw = parse_optional_string(record, header_map, col);
        let trimmed = raw.trim();
        if trimmed.is_empty() || trimmed == "-" {
            return Err(CsvRowError {
                row: row_num,
                message: format!("必須列 '{}' が空または値なし", col),
            });
        }
        parse_number(trimmed).map_err(|e| CsvRowError {
            row: row_num,
            message: format!("{}: {}", col, e),
        })
    };

    let security_code = parse_optional_string(record, header_map, "銘柄コード").replace('"', "");
    Ok(CreateAssetBalanceRequest {
        security_code,
        security_name: parse_optional_string(record, header_map, "銘柄名"),
        shares: num("保有数量［株］")?,
        // 執行中は "-" / 空欄が仕様上ありうるため 0.0 フォールバック
        executing_shares: parse_number(&parse_optional_string(record, header_map, "執行中［株］"))
            .unwrap_or(0.0),
        average_purchase_price: num("平均取得価額［円］")?,
        total_purchase_amount: num("取得総額［円］")?,
        current_price: num("現在値［円］")?,
        // 前日比は変動なし時に 0 または "-" が仕様上ありうるため 0.0 フォールバック
        daily_change: parse_number(&parse_optional_string(
            record,
            header_map,
            "現在値（前日比）［円］",
        ))
        .unwrap_or(0.0),
        market_value: num("時価評価額［円］")?,
        // 評価損益は NISA 等で表示されない場合に "-" が仕様上ありうるため 0.0 フォールバック
        profit_loss_rate: parse_number(&parse_optional_string(
            record,
            header_map,
            "評価損益［％］",
        ))
        .unwrap_or(0.0),
    })
}

/// 認証ユーザーの保有銘柄を全削除
pub async fn delete_all(pool: &PgPool, user_id: Uuid) -> Result<u64, ApiError> {
    crate::services::shared::delete_all_for_user(pool, user_id, "asset_balances", "asset_balance")
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::csv_import::parse_csv;

    fn make_csv(header: &str, row: &str) -> String {
        format!("{}\n{}\n", header, row)
    }

    const HEADER: &str =
        "銘柄コード,銘柄名,保有数量［株］,執行中［株］,平均取得価額［円］,取得総額［円］,現在値［円］,現在値（前日比）［円］,時価評価額［円］,評価損益［％］";

    #[test]
    fn test_parse_asset_balance_row_ok() {
        let csv = make_csv(
            HEADER,
            "1234,テスト株式会社,100,-,1500,150000,1600,10,160000,6.67",
        );
        let (items, errors) = parse_csv(csv.as_bytes(), parse_asset_balance_row).unwrap();
        assert!(errors.is_empty());
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].security_code, "1234");
        assert_eq!(items[0].shares, 100.0);
        assert_eq!(items[0].executing_shares, 0.0); // "-" → 0.0
        assert_eq!(items[0].average_purchase_price, 1500.0);
        assert_eq!(items[0].current_price, 1600.0);
    }

    #[test]
    fn test_parse_asset_balance_row_invalid_shares_is_error() {
        // 保有数量が不正値（N/A など）→ サイレント 0 埋めせず CsvRowError を返す
        let csv = make_csv(HEADER, "1234,テスト,N/A,-,1500,150000,1600,0,160000,0");
        let (items, errors) = parse_csv(csv.as_bytes(), parse_asset_balance_row).unwrap();
        assert!(items.is_empty(), "不正行はアイテムに含まれてはいけない");
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("保有数量"));
    }

    #[test]
    fn test_parse_asset_balance_row_dash_in_required_col_is_error() {
        // 保有数量が "-"（値なし）→ 必須列なので CsvRowError
        let csv = make_csv(HEADER, "1234,テスト,-,-,1500,150000,1600,0,160000,0");
        let (items, errors) = parse_csv(csv.as_bytes(), parse_asset_balance_row).unwrap();
        assert!(
            items.is_empty(),
            "必須列が '-' の行はアイテムに含まれてはいけない"
        );
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("保有数量"));
    }

    #[test]
    fn test_parse_asset_balance_row_invalid_price_is_error() {
        // 現在値が不正値（N/A など）→ CsvRowError
        let csv = make_csv(HEADER, "1234,テスト,100,-,1500,150000,N/A,0,160000,0");
        let (items, errors) = parse_csv(csv.as_bytes(), parse_asset_balance_row).unwrap();
        assert!(items.is_empty());
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("現在値"));
    }

    #[test]
    fn test_parse_asset_balance_row_optional_zero_fields() {
        // 前日比・評価損益が "-" でも 0.0 として許容
        let csv = make_csv(HEADER, "5678,ファンド,50,-,2000,100000,2100,-,105000,-");
        let (items, errors) = parse_csv(csv.as_bytes(), parse_asset_balance_row).unwrap();
        assert!(errors.is_empty());
        assert_eq!(items[0].daily_change, 0.0);
        assert_eq!(items[0].profit_loss_rate, 0.0);
    }
}
