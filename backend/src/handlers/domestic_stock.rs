use crate::{
    errors::ApiError,
    extractors::auth::AuthenticatedUser,
    extractors::validated_json::ValidatedJson,
    models::dividend::BulkCreateResponse,
    models::domestic_stock::{BulkCreateDomesticStockRequest, DomesticStock},
    state::AppState,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use std::time::Instant;
use tracing::info;

/// 認証ユーザーの国内株式取引一覧を取得
pub async fn list(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<impl IntoResponse, ApiError> {
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
    .bind(auth_user.id())
    .fetch_all(&state.pool)
    .await?;

    Ok((StatusCode::OK, Json(stocks)))
}

/// 国内株式取引を一括追加（重複はスキップ）
pub async fn bulk_create(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    ValidatedJson(data): ValidatedJson<BulkCreateDomesticStockRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let total = data.items.len();
    info!("[domestic_stock.bulk_create] リクエスト受信: {}件", total);
    let start = Instant::now();

    if data.items.is_empty() {
        return Ok((
            StatusCode::CREATED,
            Json(BulkCreateResponse {
                inserted: 0,
                skipped: 0,
            }),
        ));
    }

    let user_id = auth_user.id();

    // 各フィールドを配列に変換
    let user_ids: Vec<uuid::Uuid> = vec![user_id; total];
    let trade_dates: Vec<chrono::NaiveDate> = data.items.iter().map(|i| i.trade_date).collect();
    let settlement_dates: Vec<chrono::NaiveDate> =
        data.items.iter().map(|i| i.settlement_date).collect();
    let security_codes: Vec<&str> = data
        .items
        .iter()
        .map(|i| i.security_code.as_str())
        .collect();
    let security_names: Vec<&str> = data
        .items
        .iter()
        .map(|i| i.security_name.as_str())
        .collect();
    let accounts: Vec<&str> = data.items.iter().map(|i| i.account.as_str()).collect();
    let shares: Vec<f64> = data.items.iter().map(|i| i.shares).collect();
    let asked_prices: Vec<f64> = data.items.iter().map(|i| i.asked_price).collect();
    let proceeds: Vec<f64> = data.items.iter().map(|i| i.proceeds).collect();
    let purchase_prices: Vec<f64> = data.items.iter().map(|i| i.purchase_price).collect();
    let realized_pls: Vec<f64> = data
        .items
        .iter()
        .map(|i| i.realized_profit_and_loss)
        .collect();
    let taxes: Vec<f64> = data.items.iter().map(|i| i.taxes).collect();
    let realized_pls_after_tax: Vec<f64> = data
        .items
        .iter()
        .map(|i| i.realized_profit_and_loss_after_tax)
        .collect();

    // UNNESTを使ったバルクINSERT（1回のクエリで全件挿入）
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
        ON CONFLICT (user_id, trade_date, security_code, shares, proceeds)
        DO NOTHING
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
    .execute(&state.pool)
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
    Ok((
        StatusCode::CREATED,
        Json(BulkCreateResponse { inserted, skipped }),
    ))
}

/// 認証ユーザーの国内株式取引を全削除
pub async fn delete_all(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<impl IntoResponse, ApiError> {
    info!("[domestic_stock.delete_all] リクエスト受信");
    let result = sqlx::query("DELETE FROM domestic_stocks WHERE user_id = $1")
        .bind(auth_user.id())
        .execute(&state.pool)
        .await?;

    info!(
        "[domestic_stock.delete_all] 完了: {}件削除",
        result.rows_affected()
    );
    Ok((
        StatusCode::OK,
        Json(serde_json::json!({"message": "全ての国内株式取引データを削除しました"})),
    ))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_compilation() {
        assert!(true);
    }
}
