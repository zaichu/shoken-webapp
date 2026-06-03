use crate::errors::ApiError;
use crate::models::common::BulkCreateResponse;
use crate::models::csv_import::{CsvPreviewResponse, CsvRowError, CsvUploadResponse};
use crate::models::domestic_stock::{CreateDomesticStockRequest, DomesticStock};
use crate::services::csv_import::{build_csv_preview, run_csv_upload, validate_csv_rows};
use crate::services::csv_pipeline::{CsvParserConfig, CsvRow};
use crate::services::csv_util::{
    compute_taxes, normalize_security_name, parse_required_date_row, parse_required_number_row,
    parse_required_string_row,
};
use crate::services::shared::{user_ids_for_bulk_insert, BulkTimer};
use rust_decimal::Decimal;
use sqlx::PgPool;
use tracing::info;
use uuid::Uuid;

const DOMESTIC_STOCK_CSV_CONFIG: CsvParserConfig = CsvParserConfig {
    skip_header_rows: 0,
    exclude_row_fn: None,
    required_columns: &[
        "約定日",
        "受渡日",
        "銘柄コード",
        "銘柄名",
        "口座",
        "数量[株]",
        "売却/決済単価[円]",
        "売却/決済額[円]",
        "平均取得価額[円]",
        "実現損益[円]",
    ],
};

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
///
/// 再アップロード防止:
///   content_hash + グローバル occurrence_index（user_id + content_hash 単位の通し番号）により
///   DB に既存の行と同一内容・同一順序の行はスキップされる。
///   対象: 累積CSV（既存分を含む再アップロード）および増分CSV（新規分を追加したアップロード）。
///
/// 注意:
///   純増分CSV（新規分のみ）で既存行と同一ハッシュの行が含まれる場合は
///   batch_index <= existing_count でスキップされる。
///   この挙動を避けるには外部キー（取引ID等）による識別が別途必要。
pub async fn bulk_create(
    pool: &PgPool,
    user_id: Uuid,
    items: &[CreateDomesticStockRequest],
) -> Result<BulkCreateResponse, ApiError> {
    let total = items.len();
    let timer = BulkTimer::new("domestic_stock", total);

    if items.is_empty() {
        return Ok(timer.finish(0));
    }

    // 各フィールドを配列に変換
    let user_ids = user_ids_for_bulk_insert(user_id, total);
    let trade_dates: Vec<chrono::NaiveDate> = items.iter().map(|i| i.trade_date).collect();
    let settlement_dates: Vec<chrono::NaiveDate> =
        items.iter().map(|i| i.settlement_date).collect();
    let security_codes: Vec<&str> = items.iter().map(|i| i.security_code.as_str()).collect();
    let security_names: Vec<&str> = items.iter().map(|i| i.security_name.as_str()).collect();
    let accounts: Vec<&str> = items.iter().map(|i| i.account.as_str()).collect();
    let shares: Vec<Decimal> = items.iter().map(|i| i.shares).collect();
    let asked_prices: Vec<Decimal> = items.iter().map(|i| i.asked_price).collect();
    let proceeds: Vec<Decimal> = items.iter().map(|i| i.proceeds).collect();
    let purchase_prices: Vec<Decimal> = items.iter().map(|i| i.purchase_price).collect();
    let realized_pls: Vec<Decimal> = items.iter().map(|i| i.realized_profit_and_loss).collect();
    let taxes: Vec<Decimal> = items.iter().map(|i| i.taxes).collect();
    let realized_pls_after_tax: Vec<Decimal> = items
        .iter()
        .map(|i| i.realized_profit_and_loss_after_tax)
        .collect();

    // UNNESTを使ったバルクINSERT（1回のクエリで全件挿入）
    // content_hash は PostgreSQL md5 関数で算出（migration backfill と同一実装）
    // batch_occurrence_index はバッチ内での content_hash 別の連番（WITH ORDINALITY で入力順保持）
    // db_counts は既存 DB の (user_id, content_hash) 単位の件数（他ユーザーに引っ張られない）
    // batch_occurrence_index > existing_count の行のみ挿入し、ON CONFLICT で冪等性を保証
    let result = sqlx::query(
        r#"
        WITH batch_data AS (
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
                $5::text[], $6::text[], $7::numeric[], $8::numeric[], $9::numeric[],
                $10::numeric[], $11::numeric[], $12::numeric[], $13::numeric[]
            ) WITH ORDINALITY AS t(user_id, trade_date, settlement_date, security_code,
                   security_name, account, shares, asked_price, proceeds,
                   purchase_price, realized_profit_and_loss, taxes,
                   realized_profit_and_loss_after_tax, ordinality)
        ),
        batch_indexed AS (
            SELECT *,
                ROW_NUMBER() OVER (PARTITION BY content_hash ORDER BY ordinality)::int4
                    AS batch_occurrence_index
            FROM batch_data
        ),
        db_counts AS (
            SELECT content_hash, COUNT(*)::int4 AS existing_count
            FROM domestic_stocks
            WHERE user_id = $14::uuid
            GROUP BY content_hash
        )
        INSERT INTO domestic_stocks (user_id, trade_date, settlement_date, security_code,
                                     security_name, account, shares, asked_price, proceeds,
                                     purchase_price, realized_profit_and_loss, taxes,
                                     realized_profit_and_loss_after_tax,
                                     content_hash, occurrence_index)
        SELECT
            b.user_id, b.trade_date, b.settlement_date, b.security_code,
            b.security_name, b.account, b.shares, b.asked_price, b.proceeds,
            b.purchase_price, b.realized_profit_and_loss, b.taxes,
            b.realized_profit_and_loss_after_tax,
            b.content_hash,
            b.batch_occurrence_index
        FROM batch_indexed b
        LEFT JOIN db_counts d ON b.content_hash = d.content_hash
        WHERE b.batch_occurrence_index > COALESCE(d.existing_count, 0)
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
    .bind(user_id) // $14: スカラーのユーザーID（db_counts WHERE 句用）
    .execute(pool)
    .await?;

    let inserted = result.rows_affected() as usize;
    Ok(timer.finish(inserted))
}

/// CSV バイト列から国内株式取引をパースしてプレビュー情報を返す（DB 書き込みなし）
pub fn preview_csv(bytes: &[u8]) -> Result<CsvPreviewResponse, ApiError> {
    build_csv_preview(
        bytes,
        &DOMESTIC_STOCK_CSV_CONFIG,
        transform_domestic_stock_rows,
    )
}

/// CSV バイト列から国内株式取引をパースして一括挿入
pub async fn upload_csv(
    pool: &PgPool,
    user_id: Uuid,
    bytes: &[u8],
) -> Result<CsvUploadResponse, ApiError> {
    run_csv_upload(
        bytes,
        &DOMESTIC_STOCK_CSV_CONFIG,
        transform_domestic_stock_rows,
        |items| async move { bulk_create(pool, user_id, &items).await },
    )
    .await
}

fn transform_domestic_stock_rows(
    rows: &[CsvRow],
) -> (Vec<CreateDomesticStockRequest>, Vec<CsvRowError>) {
    validate_csv_rows(rows, transform_domestic_stock_row)
}

fn transform_domestic_stock_row(
    row: &CsvRow,
    row_num: usize,
) -> Result<CreateDomesticStockRequest, CsvRowError> {
    let trade_date = parse_required_date_row(row, "約定日", row_num)?;
    let settlement_date = parse_required_date_row(row, "受渡日", row_num)?;
    let account = parse_required_string_row(row, "口座", row_num)?;
    let realized_pnl = parse_required_number_row(row, "実現損益[円]", row_num)?;
    let (taxes, realized_pnl_after_tax) = compute_taxes(&account, realized_pnl);
    Ok(CreateDomesticStockRequest {
        trade_date,
        settlement_date,
        security_code: parse_required_string_row(row, "銘柄コード", row_num)?,
        security_name: normalize_security_name(&parse_required_string_row(row, "銘柄名", row_num)?),
        account,
        shares: parse_required_number_row(row, "数量[株]", row_num)?,
        asked_price: parse_required_number_row(row, "売却/決済単価[円]", row_num)?,
        proceeds: parse_required_number_row(row, "売却/決済額[円]", row_num)?,
        purchase_price: parse_required_number_row(row, "平均取得価額[円]", row_num)?,
        realized_profit_and_loss: realized_pnl,
        taxes,
        realized_profit_and_loss_after_tax: realized_pnl_after_tax,
    })
}

/// 認証ユーザーの国内株式取引を全削除
pub async fn delete_all(pool: &PgPool, user_id: Uuid) -> Result<u64, ApiError> {
    crate::services::shared::delete_all_for_user(pool, user_id, "domestic_stocks", "domestic_stock")
        .await
}
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use rust_decimal_macros::dec;

    const HEADER: &str =
        "約定日,受渡日,銘柄コード,銘柄名,口座,信用区分,取引,数量[株],売却/決済単価[円],売却/決済額[円],平均取得価額[円],実現損益[円]";
    const BASIC_ROW: &str =
        "\"2026/02/09\",\"2026/02/12\",\"5020\",\"ＥＮＥＯＳホールディングス\",\"特定\",\"-\",\"売付\",\"100\",\"1,441.0\",\"144,100\",\"1,350.00\",\"9,100\"";
    const NISA_ROW: &str =
        "\"2026/02/09\",\"2026/02/12\",\"9433\",\"K D D I\",\"NISA\",\"-\",\"売付\",\"100\",\"1441.0\",\"144100\",\"1350.00\",\"9100\"";
    const MISSING_NAME_HEADER: &str =
        "約定日,受渡日,銘柄コード,口座,信用区分,取引,数量[株],売却/決済単価[円],売却/決済額[円],平均取得価額[円],実現損益[円]";
    const MISSING_NAME_ROW: &str =
        "\"2026/02/09\",\"2026/02/12\",\"5020\",\"特定\",\"-\",\"売付\",\"100\",\"1,441.0\",\"144,100\",\"1,350.00\",\"9,100\"";
    const INVALID_PNL_ROW: &str =
        "\"2026/02/09\",\"2026/02/12\",\"5020\",\"ＥＮＥＯＳ\",\"特定\",\"-\",\"売付\",\"100\",\"1441.0\",\"144100\",\"1350.00\",\"N/A\"";

    fn preview_with_header(header: &str, row: &str) -> CsvPreviewResponse {
        preview_csv(format!("{header}\n{row}").as_bytes()).unwrap()
    }

    fn assert_preview_ok(row: &str) -> CsvPreviewResponse {
        let preview = preview_with_header(HEADER, row);

        assert_eq!(
            (
                preview.total_rows,
                preview.valid_rows,
                preview.rows.len(),
                preview.errors.is_empty(),
            ),
            (1, 1, 1, true),
            "unexpected errors: {:?}",
            preview.errors
        );

        preview
    }

    fn assert_preview_error(header: &str, row: &str, expected_message: &str) {
        let preview = preview_with_header(header, row);

        assert_eq!(
            (
                preview.total_rows,
                preview.valid_rows,
                preview.errors.len(),
                preview
                    .errors
                    .first()
                    .is_some_and(|error| error.message.contains(expected_message)),
            ),
            (1, 0, 1, true)
        );
    }

    #[test]
    fn test_preview_csv() {
        assert_eq!(
            assert_preview_ok(BASIC_ROW).rows[0]["security_name"],
            "ＥＮＥＯＳホールディングス"
        );

        let preview = assert_preview_ok(NISA_ROW);
        assert_eq!(
            (
                preview.rows[0]["security_name"].as_str(),
                preview.rows[0]["taxes"].as_f64(),
                preview.rows[0]["realized_profit_and_loss_after_tax"].as_f64(),
            ),
            (Some("KDDI"), Some(0.0), Some(9100.0))
        );

        for (header, row, expected_message) in [
            (MISSING_NAME_HEADER, MISSING_NAME_ROW, "銘柄名"),
            (HEADER, INVALID_PNL_ROW, "実現損益[円]"),
        ] {
            assert_preview_error(header, row, expected_message);
        }

        assert!(matches!(
            preview_csv(b""),
            Err(ApiError::ValidationError(_))
        ));
    }

    fn make_test_item() -> CreateDomesticStockRequest {
        CreateDomesticStockRequest {
            trade_date: NaiveDate::from_ymd_opt(2026, 2, 12).unwrap(),
            settlement_date: NaiveDate::from_ymd_opt(2026, 2, 16).unwrap(),
            security_code: "9508".to_string(),
            security_name: "九州電力".to_string(),
            account: "特定".to_string(),
            shares: dec!(100),
            asked_price: dec!(1880),
            proceeds: dec!(188000),
            purchase_price: dec!(1770),
            realized_profit_and_loss: dec!(11000),
            taxes: dec!(2234),
            realized_profit_and_loss_after_tax: dec!(8766),
        }
    }

    #[tokio::test]
    #[ignore = "requires Docker"]
    async fn test_bulk_create_reupload_deduplication() {
        use testcontainers::runners::AsyncRunner;
        use testcontainers_modules::postgres::Postgres;

        let container = Postgres::default().start().await.unwrap();
        let url = format!(
            "postgres://postgres:postgres@{}:{}/postgres",
            container.get_host().await.unwrap(),
            container.get_host_port_ipv4(5432).await.unwrap()
        );
        let pool = sqlx::PgPool::connect(&url).await.unwrap();

        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        let user_id = Uuid::new_v4();
        sqlx::query("INSERT INTO users (id, google_id, email) VALUES ($1, $2, $3)")
            .bind(user_id)
            .bind(format!("test_google_{user_id}"))
            .bind(format!("test_{user_id}@example.com"))
            .execute(&pool)
            .await
            .unwrap();

        let items = vec![make_test_item(); 5];

        let first = bulk_create(&pool, user_id, &items).await.unwrap();
        assert_eq!((first.inserted, first.skipped), (5, 0));

        let second = bulk_create(&pool, user_id, &items).await.unwrap();
        assert_eq!((second.inserted, second.skipped), (0, 5));
    }
}
