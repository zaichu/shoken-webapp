use crate::errors::ApiError;
use crate::models::common::BulkCreateResponse;
use crate::models::csv_import::{CsvPreviewResponse, CsvRowError, CsvUploadResponse};
use crate::models::mutualfund::{CreateMutualfundRequest, Mutualfund};
use crate::services::csv_import::{build_preview, finish_csv_upload, parse_csv};
use crate::services::csv_util::{
    compute_taxes, get_cell, parse_required_date, parse_required_number, parse_required_string,
};
use crate::services::shared::BulkTimer;
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::collections::HashMap;
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
    let timer = BulkTimer::new("mutualfund", total);

    if items.is_empty() {
        return Ok(timer.finish(0));
    }

    // 各フィールドを配列に変換
    let user_ids: Vec<Uuid> = vec![user_id; total];
    let trade_dates: Vec<chrono::NaiveDate> = items.iter().map(|i| i.trade_date).collect();
    let settlement_dates: Vec<chrono::NaiveDate> =
        items.iter().map(|i| i.settlement_date).collect();
    let fund_names: Vec<&str> = items.iter().map(|i| i.fund_name.as_str()).collect();
    let dividends: Vec<Option<&str>> = items.iter().map(|i| i.dividends.as_deref()).collect();
    let accounts: Vec<&str> = items.iter().map(|i| i.account.as_str()).collect();
    let shares: Vec<Decimal> = items.iter().map(|i| i.shares).collect();
    let exchange_rates: Vec<Decimal> = items.iter().map(|i| i.exchange_rate).collect();
    let cancellation_unit_prices: Vec<Decimal> = items
        .iter()
        .map(|i| i.cancellation_unit_price_yen)
        .collect();
    let cancellation_amounts: Vec<Decimal> =
        items.iter().map(|i| i.cancellation_amount_yen).collect();
    let avg_acquisition_prices: Vec<Decimal> = items
        .iter()
        .map(|i| i.average_acquisition_price_yen)
        .collect();
    let realized_pls: Vec<Decimal> = items.iter().map(|i| i.realized_profit_and_loss).collect();
    let taxes: Vec<Decimal> = items.iter().map(|i| i.taxes).collect();
    let realized_pls_after_tax: Vec<Decimal> = items
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
            $6::text[], $7::numeric[], $8::numeric[], $9::numeric[],
            $10::numeric[], $11::numeric[], $12::numeric[], $13::numeric[], $14::numeric[]
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
    Ok(timer.finish(inserted))
}

/// CSV バイト列から投資信託をパースしてプレビュー情報を返す（DB 書き込みなし）
pub fn preview_csv(bytes: &[u8]) -> Result<CsvPreviewResponse, ApiError> {
    if bytes.is_empty() {
        return Err(ApiError::ValidationError("CSVが空です".to_string()));
    }
    build_preview(bytes, parse_mutualfund_row)
}

/// CSV バイト列から投資信託をパースして一括挿入
pub async fn upload_csv(
    pool: &PgPool,
    user_id: Uuid,
    bytes: &[u8],
) -> Result<CsvUploadResponse, ApiError> {
    let (items, errors) = parse_csv(bytes, parse_mutualfund_row)?;
    let result = bulk_create(pool, user_id, &items).await?;
    Ok(finish_csv_upload(result, errors))
}

fn parse_mutualfund_row(
    record: &csv::StringRecord,
    header_map: &HashMap<String, usize>,
    row_num: usize,
) -> Result<CreateMutualfundRequest, CsvRowError> {
    let trade_date = parse_required_date(record, header_map, "約定日", row_num)?;
    let settlement_date = parse_required_date(record, header_map, "受渡日", row_num)?;
    let account = parse_required_string(record, header_map, "口座", row_num)?;
    let realized_pnl = parse_required_number(record, header_map, "実現損益［円］", row_num)?;
    let (taxes, realized_pnl_after_tax) = compute_taxes(&account, realized_pnl);
    // 分配金フィールドは空文字を None に変換
    let dividends_raw = get_cell(record, header_map, "分配金");
    let dividends = if dividends_raw.trim().is_empty() {
        None
    } else {
        Some(dividends_raw.to_string())
    };
    Ok(CreateMutualfundRequest {
        trade_date,
        settlement_date,
        fund_name: parse_required_string(record, header_map, "ファンド名", row_num)?,
        dividends,
        account,
        shares: parse_required_number(record, header_map, "数量[口]", row_num)?,
        exchange_rate: parse_required_number(record, header_map, "為替レート［円］", row_num)?,
        cancellation_unit_price_yen: parse_required_number(
            record,
            header_map,
            "解約単価［円］",
            row_num,
        )?,
        cancellation_amount_yen: parse_required_number(
            record,
            header_map,
            "解約額［円］",
            row_num,
        )?,
        average_acquisition_price_yen: parse_required_number(
            record,
            header_map,
            "平均取得価額［円］",
            row_num,
        )?,
        realized_profit_and_loss: realized_pnl,
        taxes,
        realized_profit_and_loss_after_tax: realized_pnl_after_tax,
    })
}

/// 認証ユーザーの投資信託を全削除
pub async fn delete_all(pool: &PgPool, user_id: Uuid) -> Result<u64, ApiError> {
    crate::services::shared::delete_all_for_user(pool, user_id, "mutualfunds", "mutualfund").await
}
#[cfg(test)] #[rustfmt::skip] mod tests { use super::*; #[test] fn test_preview_csv_basic() { let csv = concat!("約定日,受渡日,ファンド名,分配金,口座,取引,数量[口],為替レート［円］,解約単価［円］,解約額［円］,平均取得価額［円］,実現損益［円］\n", "\"2022/10/28\",\"2022/11/2\",\"eMAXIS Slim 米国株式(S&P500)\",\"再投資型\",\"特定\",\"解約\",\"3,721,147\",\"-\",\"19,661\",\"7,316,147\",\"18,005.20\",\"615,849\""); let preview = preview_csv(csv.as_bytes()).unwrap(); assert_eq!((preview.total_rows, preview.valid_rows, preview.rows.len(), preview.errors.is_empty(), matches!(preview_csv(b""), Err(ApiError::ValidationError(_)))), (1, 1, 1, true, true)); let csv = concat!("約定日,受渡日,ファンド名,分配金,口座,取引,数量[口],為替レート［円］,解約単価［円］,解約額［円］,平均取得価額［円］,実現損益［円］\n", "\"2022/10/28\",\"2022/11/2\",\"eMAXIS Slim\",\"\",\"特定\",\"解約\",\"1000\",\"1\",\"12000\",\"12000000\",\"10000\",\"615849\""); let preview = preview_csv(csv.as_bytes()).unwrap(); assert_eq!((preview.valid_rows, preview.rows[0]["dividends"].is_null()), (1, true)); } }
