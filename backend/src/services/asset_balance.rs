use crate::errors::ApiError;
use crate::models::asset_balance::{AssetBalance, CreateAssetBalanceRequest};
use crate::models::common::BulkCreateResponse;
use crate::models::csv_import::{CsvPreviewResponse, CsvRowError, CsvUploadResponse};
use crate::services::csv_import::{finish_csv_upload, parse_csv};
use crate::services::csv_util::{
    decode_bytes, normalize_security_name, parse_number, parse_optional_string,
};
use crate::services::shared::BulkTimer;
use csv::StringRecord;
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::collections::HashMap;
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
    if bytes.is_empty() {
        return Err(ApiError::ValidationError("CSVが空です".to_string()));
    }
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

/// 保有銘柄 CSV bytes をデコード・ヘッダースキップ・パース・フィルタして返す
/// preview / upload の共通前処理経路
pub(crate) fn parse_asset_balance_csv(
    bytes: &[u8],
) -> Result<(Vec<CreateAssetBalanceRequest>, Vec<CsvRowError>), ApiError> {
    let content = decode_bytes(bytes);
    let stripped = strip_asset_balance_csv_metadata(&content);
    let filtered = strip_account_summary_rows(&stripped);
    let (items, errors) = parse_csv(filtered.as_bytes(), parse_asset_balance_row)?;
    let items = items
        .into_iter()
        .filter(|i| !i.security_code.is_empty())
        .collect();
    Ok((items, errors))
}

/// 保有銘柄 CSV の先頭6行（メタデータ）をスキップした文字列を返す
fn strip_asset_balance_csv_metadata(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    if lines.len() > 6 {
        lines[6..].join("\n")
    } else {
        String::new()
    }
}

/// 保有銘柄 CSV に混ざる「特定口座合計」などの口座集計行を除外する
///
/// 先頭フィールド（銘柄コード）が空の行かつ「口座合計」を含む行のみ除外する。
/// 銘柄名に「口座合計」を含む銘柄を誤除外しないよう、先頭が空であることを条件とする。
fn strip_account_summary_rows(content: &str) -> String {
    content
        .lines()
        .filter(|line| !(line.starts_with(',') && line.contains("口座合計")))
        .collect::<Vec<_>>()
        .join("\n")
}

/// 保有銘柄 CSV の1行をパースして CreateAssetBalanceRequest に変換
///
/// 数値パース失敗は CsvRowError として返す。
/// ただし以下の列は "-" / 空欄が仕様上ありうるため 0.0 フォールバックを維持する:
///   - 執行中: 執行中の注文がなければ "-" または空欄
///   - 現在値（前日比）: 変動なし時は 0 または "-"
///   - 評価損益（%）: NISA 等で表示されない場合に "-"
pub(crate) fn parse_asset_balance_row(
    record: &StringRecord,
    header_map: &HashMap<String, usize>,
    row_num: usize,
) -> Result<CreateAssetBalanceRequest, CsvRowError> {
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
        security_name: normalize_security_name(&parse_optional_string(
            record,
            header_map,
            "銘柄名",
        )),
        shares: num("保有数量［株］")?,
        // 執行中は "-" / 空欄が仕様上ありうるため 0.0 フォールバック
        executing_shares: parse_number(&parse_optional_string(record, header_map, "執行中［株］"))
            .unwrap_or(Decimal::ZERO),
        average_purchase_price: num("平均取得価額［円］")?,
        total_purchase_amount: num("取得総額［円］")?,
        current_price: num("現在値［円］")?,
        // 前日比は変動なし時に 0 または "-" が仕様上ありうるため 0.0 フォールバック
        daily_change: parse_number(&parse_optional_string(
            record,
            header_map,
            "現在値（前日比）［円］",
        ))
        .unwrap_or(Decimal::ZERO),
        market_value: num("時価評価額［円］")?,
        // 評価損益は NISA 等で表示されない場合に "-" が仕様上ありうるため 0.0 フォールバック
        profit_loss_rate: parse_number(&parse_optional_string(
            record,
            header_map,
            "評価損益［％］",
        ))
        .unwrap_or(Decimal::ZERO),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    const HEADER: &str =
        "銘柄コード,銘柄名,保有数量［株］,執行中［株］,平均取得価額［円］,取得総額［円］,現在値［円］,現在値（前日比）［円］,時価評価額［円］,評価損益［％］";
    const ASSET_BALANCE_CSV_HEADER: &str = "銘柄コード,銘柄名,保有数量［株］,執行中［株］,(内訳　通常数量[株]),(内訳　積立数量[株]),平均取得価額［円］,取得総額［円］,現在値［円］,現在値（前日比）［円］,時価評価額［円］,評価損益［％］";
    const TEST_ROW_1: &str = "1234,テスト株式会社,100,0,100,0,1500,150000,1600,10,160000,6.67";
    const TEST_ROW_2: &str = "5678,サンプル株式会社,200,0,200,0,1800,360000,1900,15,380000,5.56";
    const INPEX_ROW: &str =
        "\"1605\",\"ＩＮＰＥＸ\",\"200\",\"0\",\"200\",\"0\",\"2,355.00\",\"471,000\",\"3,685.0\",\"65.0\",\"737,000\",\"56.47\"";
    const NINTENDO_ROW: &str =
        "\"7974\",\"任天堂\",\"1,000\",\"0\",\"1,000\",\"0\",\"5,997.60\",\"5,997,600\",\"8,737.0\",\"223.0\",\"8,737,000\",\"45.67\"";
    const ACCOUNT_SUMMARY_ROW: &str =
        ",,,,,,特定口座合計,\"11,245,249\",,,\"14,517,240\",\"29.09\"";

    fn parse_row_csv(row: &str) -> (Vec<CreateAssetBalanceRequest>, Vec<CsvRowError>) {
        parse_csv(
            format!("{HEADER}\n{row}\n").as_bytes(),
            parse_asset_balance_row,
        )
        .unwrap()
    }

    fn make_asset_balance_csv(rows: &[&str]) -> String {
        format!(
            "■現在の評価額合計［円］,,\"9,474,000\"\n■評価損益合計,前日比［円］,\"288,000\"\n,前月比［円］,\"-120,000\"\n,評価損益［円］,\"2,005,900\"\n■特定口座\n\n{ASSET_BALANCE_CSV_HEADER}\n{}",
            rows.join("\n")
        )
    }

    fn assert_row_ok(
        row: &str,
        expected: (&str, Decimal, Decimal, Decimal, Decimal, Decimal, Decimal),
    ) {
        let (items, errors) = parse_row_csv(row);
        assert!(errors.is_empty(), "unexpected errors for {row}: {errors:?}");
        assert_eq!(items.len(), 1, "row should produce one item: {row}");
        let item = &items[0];
        assert_eq!(
            (
                item.security_code.as_str(),
                item.shares,
                item.executing_shares,
                item.average_purchase_price,
                item.current_price,
                item.daily_change,
                item.profit_loss_rate,
            ),
            expected
        );
    }

    fn assert_row_error(row: &str, expected_message: &str) {
        let (items, errors) = parse_row_csv(row);
        assert!(items.is_empty(), "invalid row must not be parsed: {row}");
        assert_eq!(errors.len(), 1, "row should produce one error: {row}");
        assert!(errors[0].message.contains(expected_message), "{errors:?}");
    }

    #[test]
    fn test_parse_asset_balance_row() {
        for (row, expected) in [
            (
                "1234,テスト株式会社,100,-,1500,150000,1600,10,160000,6.67",
                (
                    "1234",
                    dec!(100),
                    Decimal::ZERO,
                    dec!(1500),
                    dec!(1600),
                    dec!(10),
                    dec!(6.67),
                ),
            ),
            (
                "5678,ファンド,50,-,2000,100000,2100,-,105000,-",
                (
                    "5678",
                    dec!(50),
                    Decimal::ZERO,
                    dec!(2000),
                    dec!(2100),
                    Decimal::ZERO,
                    Decimal::ZERO,
                ),
            ),
        ] {
            assert_row_ok(row, expected);
        }

        for (row, expected_message) in [
            ("1234,テスト,N/A,-,1500,150000,1600,0,160000,0", "保有数量"),
            ("1234,テスト,-,-,1500,150000,1600,0,160000,0", "保有数量"),
            ("1234,テスト,100,-,1500,150000,N/A,0,160000,0", "現在値"),
        ] {
            assert_row_error(row, expected_message);
        }

        let csv = make_asset_balance_csv(&[INPEX_ROW, NINTENDO_ROW, ACCOUNT_SUMMARY_ROW]);
        let preview = preview_csv(csv.as_bytes()).unwrap();
        assert_eq!(
            (preview.total_rows, preview.valid_rows, preview.rows.len()),
            (2, 2, 2)
        );
        assert!(
            preview.errors.is_empty(),
            "unexpected errors: {:?}",
            preview.errors
        );

        assert!(matches!(
            preview_csv(b""),
            Err(ApiError::ValidationError(_))
        ));

        for (rows, expected_codes) in [
            (
                &[TEST_ROW_1, ",,,,,,,,,,,", TEST_ROW_2][..],
                &["1234", "5678"][..],
            ),
            (
                &[INPEX_ROW, NINTENDO_ROW, ACCOUNT_SUMMARY_ROW][..],
                &["1605", "7974"][..],
            ),
        ] {
            let csv = make_asset_balance_csv(rows);
            let (items, errors) = parse_asset_balance_csv(csv.as_bytes()).unwrap();
            assert!(errors.is_empty(), "unexpected errors: {errors:?}");
            assert_eq!(
                items
                    .iter()
                    .map(|item| item.security_code.as_str())
                    .collect::<Vec<_>>(),
                expected_codes
            );
        }
    }
}
