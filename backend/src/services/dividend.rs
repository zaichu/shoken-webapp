use crate::errors::ApiError;
use crate::models::common::{BulkCreateResponse, PaginatedResponse, PaginationParams};
use crate::models::csv_import::{CsvPreviewResponse, CsvRowError, CsvUploadResponse};
use crate::models::dividend::{CreateDividendRequest, Dividend};
use crate::services::csv_import::{build_csv_preview, run_csv_upload, validate_csv_rows};
use crate::services::csv_pipeline::{CsvParserConfig, CsvRow};
use crate::services::csv_util::{
    normalize_security_name, parse_optional_string_row, parse_required_date_row,
    parse_required_number_row, parse_required_string_row,
};
use crate::services::shared::{
    delete_all_for_user, user_ids_for_bulk_insert, BulkTimer, DeleteTarget,
};
use rust_decimal::Decimal;
use sqlx::PgPool;
use tracing::info;
use uuid::Uuid;

const DIVIDEND_CSV_CONFIG: CsvParserConfig = CsvParserConfig {
    skip_header_rows: 0,
    exclude_row_fn: None,
    required_columns: &[
        "入金日",
        "商品",
        "口座",
        "銘柄コード",
        "銘柄",
        "単価[円/現地通貨]",
        "数量[株/口]",
        "配当・分配金合計（税引前）[円/現地通貨]",
        "税額合計[円/現地通貨]",
        "受取金額[円/現地通貨]",
    ],
};

/// 認証ユーザーの配当金一覧を取得（ページネーション対応）
pub async fn list(
    pool: &PgPool,
    user_id: Uuid,
    params: &PaginationParams,
) -> Result<PaginatedResponse<Dividend>, ApiError> {
    info!("[dividend.list] リクエスト受信");
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM dividends WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(pool)
        .await?;

    let data = sqlx::query_as::<_, Dividend>(
        r#"
        SELECT id, user_id, settlement_date, product, account, security_code, security_name,
               unit_price, shares, dividends_before_tax, taxes, net_amount_received,
               created_at, updated_at
        FROM dividends
        WHERE user_id = $1
        ORDER BY settlement_date DESC, id DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(user_id)
    .bind(params.per_page())
    .bind(params.offset())
    .fetch_all(pool)
    .await?;

    Ok(PaginatedResponse { data, total, page: params.page(), per_page: params.per_page() })
}

/// 配当金を一括追加（重複はスキップ）
pub async fn bulk_create(
    pool: &PgPool,
    user_id: Uuid,
    items: &[CreateDividendRequest],
) -> Result<BulkCreateResponse, ApiError> {
    let timer = match BulkTimer::new_with_guard("dividend", items) {
        Ok(t) => t,
        Err(empty) => return Ok(empty),
    };

    // 各フィールドを配列に変換
    let user_ids = user_ids_for_bulk_insert(user_id, items.len());
    let settlement_dates: Vec<chrono::NaiveDate> =
        items.iter().map(|i| i.settlement_date).collect();
    let products: Vec<&str> = items.iter().map(|i| i.product.as_str()).collect();
    let accounts: Vec<&str> = items.iter().map(|i| i.account.as_str()).collect();
    let security_codes: Vec<&str> = items.iter().map(|i| i.security_code.as_str()).collect();
    let security_names: Vec<&str> = items.iter().map(|i| i.security_name.as_str()).collect();
    let unit_prices: Vec<Decimal> = items.iter().map(|i| i.unit_price).collect();
    let shares: Vec<Decimal> = items.iter().map(|i| i.shares).collect();
    let dividends_before_taxes: Vec<Decimal> =
        items.iter().map(|i| i.dividends_before_tax).collect();
    let taxes: Vec<Decimal> = items.iter().map(|i| i.taxes).collect();
    let net_amounts: Vec<Decimal> = items.iter().map(|i| i.net_amount_received).collect();

    // UNNESTを使ったバルクINSERT（1回のクエリで全件挿入）
    let result = sqlx::query(
        r#"
        INSERT INTO dividends (user_id, settlement_date, product, account, security_code,
                               security_name, unit_price, shares, dividends_before_tax,
                               taxes, net_amount_received)
        SELECT * FROM UNNEST(
            $1::uuid[], $2::date[], $3::text[], $4::text[], $5::text[],
            $6::text[], $7::numeric[], $8::numeric[], $9::numeric[],
            $10::numeric[], $11::numeric[]
        )
        ON CONFLICT (user_id, settlement_date, security_code, security_name, shares, dividends_before_tax)
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

    timer.finish_from_result(result)
}

/// CSV バイト列から配当金をパースしてプレビュー情報を返す（DB 書き込みなし）
pub fn preview_csv(bytes: &[u8]) -> Result<CsvPreviewResponse, ApiError> {
    build_csv_preview(bytes, &DIVIDEND_CSV_CONFIG, transform_dividend_rows)
}

/// CSV バイト列から配当金をパースして一括挿入
pub async fn upload_csv(
    pool: &PgPool,
    user_id: Uuid,
    bytes: &[u8],
) -> Result<CsvUploadResponse, ApiError> {
    run_csv_upload(
        bytes,
        &DIVIDEND_CSV_CONFIG,
        transform_dividend_rows,
        |items| async move { bulk_create(pool, user_id, &items).await },
    )
    .await
}

fn transform_dividend_rows(rows: &[CsvRow]) -> (Vec<CreateDividendRequest>, Vec<CsvRowError>) {
    validate_csv_rows(rows, transform_dividend_row)
}

fn transform_dividend_row(
    row: &CsvRow,
    row_num: usize,
) -> Result<CreateDividendRequest, CsvRowError> {
    Ok(CreateDividendRequest {
        settlement_date: parse_required_date_row(row, "入金日", row_num)?,
        product: parse_required_string_row(row, "商品", row_num)?,
        account: parse_required_string_row(row, "口座", row_num)?,
        security_code: parse_optional_string_row(row, "銘柄コード"),
        security_name: normalize_security_name(&parse_required_string_row(row, "銘柄", row_num)?),
        unit_price: parse_required_number_row(row, "単価[円/現地通貨]", row_num)?,
        shares: parse_required_number_row(row, "数量[株/口]", row_num)?,
        dividends_before_tax: parse_required_number_row(
            row,
            "配当・分配金合計（税引前）[円/現地通貨]",
            row_num,
        )?,
        taxes: parse_required_number_row(row, "税額合計[円/現地通貨]", row_num)?,
        net_amount_received: parse_required_number_row(row, "受取金額[円/現地通貨]", row_num)?,
    })
}

/// 認証ユーザーの配当金を全削除
pub async fn delete_all(pool: &PgPool, user_id: Uuid) -> Result<u64, ApiError> {
    delete_all_for_user(pool, user_id, DeleteTarget::Dividends).await
}
#[cfg(test)]
mod tests {
    use super::*;

    const HEADER: &str =
        "入金日,商品,口座,銘柄コード,銘柄,受取通貨,単価[円/現地通貨],数量[株/口],配当・分配金合計（税引前）[円/現地通貨],税額合計[円/現地通貨],受取金額[円/現地通貨]";

    fn preview_from_lines(lines: &[&str]) -> CsvPreviewResponse {
        preview_csv(lines.join("\n").as_bytes()).unwrap()
    }

    #[test]
    fn test_preview_csv_basic() {
        let preview = preview_from_lines(&[
            HEADER,
            "\"2025/12/09\",\"国内株式\",\"特定・一般\",\"8591\",\"オリックス\",\"円\",\"93.76\",\"200\",\"18,752\",\"3,808\",\"14,944\"",
            "\"2025/12/10\",\"国内株式\",\"特定・一般\",\"8306\",\"三菱ＵＦＪフィナンシャル・グループ\",\"円\",\"50.00\",\"300\",\"15,000\",\"3,047\",\"11,953\"",
        ]);
        assert_eq!(
            (
                preview.total_rows,
                preview.valid_rows,
                preview.errors.is_empty(),
                preview.rows.len(),
            ),
            (2, 2, true, 2)
        );

        assert!(matches!(
            preview_csv(b""),
            Err(ApiError::ValidationError(_))
        ));

        let preview = preview_from_lines(&[
            HEADER,
            "\"2025/12/09\",\"国内株式\",\"特定・一般\",\"\",\"K D D I\",\"円\",\"93.76\",\"200\",\"18752\",\"3808\",\"14944\"",
        ]);
        assert_eq!(
            (
                preview.valid_rows,
                preview.rows[0]["security_code"].as_str(),
                preview.rows[0]["security_name"].as_str(),
            ),
            (1, Some(""), Some("KDDI"))
        );

        for (header, row, expected_message) in [
            (
                "入金日,商品,口座,銘柄コード,受取通貨,単価[円/現地通貨],数量[株/口],配当・分配金合計（税引前）[円/現地通貨],税額合計[円/現地通貨],受取金額[円/現地通貨]",
                "\"2025/12/09\",\"国内株式\",\"特定・一般\",\"8591\",\"円\",\"93.76\",\"200\",\"18,752\",\"3,808\",\"14,944\"",
                "銘柄",
            ),
            (
                HEADER,
                "\"2025/13/09\",\"国内株式\",\"特定・一般\",\"8591\",\"オリックス\",\"円\",\"93.76\",\"200\",\"18752\",\"3808\",\"14944\"",
                "入金日",
            ),
        ] {
            let preview = preview_from_lines(&[header, row]);
            assert_eq!(
                (
                    preview.total_rows,
                    preview.valid_rows,
                    matches!(preview.errors.as_slice(), [error] if error.message.contains(expected_message)),
                ),
                (1, 0, true)
            );
        }
    }
}
