use crate::{
    errors::ApiError,
    extractors::auth::AuthenticatedUser,
    extractors::validated_json::ValidatedJson,
    models::asset_balance::{AssetBalance, BulkCreateAssetBalanceRequest, BulkCreateResponse},
    state::AppState,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use std::time::Instant;
use tracing::info;

/// 認証ユーザーの保有銘柄一覧を取得
pub async fn list(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<impl IntoResponse, ApiError> {
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
    .bind(auth_user.id())
    .fetch_all(&state.pool)
    .await?;

    Ok((StatusCode::OK, Json(balances)))
}

/// 保有銘柄を一括追加（既存は更新）
pub async fn bulk_create(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    ValidatedJson(data): ValidatedJson<BulkCreateAssetBalanceRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let total = data.items.len();
    info!("[asset_balance.bulk_create] リクエスト受信: {}件", total);
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
    let shares: Vec<f64> = data.items.iter().map(|i| i.shares).collect();
    let executing_shares: Vec<f64> = data.items.iter().map(|i| i.executing_shares).collect();
    let average_purchase_prices: Vec<f64> = data
        .items
        .iter()
        .map(|i| i.average_purchase_price)
        .collect();
    let total_purchase_amounts: Vec<f64> =
        data.items.iter().map(|i| i.total_purchase_amount).collect();
    let current_prices: Vec<f64> = data.items.iter().map(|i| i.current_price).collect();
    let daily_changes: Vec<f64> = data.items.iter().map(|i| i.daily_change).collect();
    let market_values: Vec<f64> = data.items.iter().map(|i| i.market_value).collect();
    let profit_loss_rates: Vec<f64> = data.items.iter().map(|i| i.profit_loss_rate).collect();

    // UNNESTを使ったバルクUPSERT（1回のクエリで全件挿入/更新）
    let result = sqlx::query(
        r#"
        INSERT INTO asset_balances (user_id, security_code, security_name, shares, executing_shares,
                                    average_purchase_price, total_purchase_amount, current_price,
                                    daily_change, market_value, profit_loss_rate)
        SELECT * FROM UNNEST(
            $1::uuid[], $2::text[], $3::text[], $4::float8[], $5::float8[],
            $6::float8[], $7::float8[], $8::float8[], $9::float8[], $10::float8[], $11::float8[]
        )
        ON CONFLICT (user_id, security_code)
        DO UPDATE SET
            security_name = EXCLUDED.security_name,
            shares = EXCLUDED.shares,
            executing_shares = EXCLUDED.executing_shares,
            average_purchase_price = EXCLUDED.average_purchase_price,
            total_purchase_amount = EXCLUDED.total_purchase_amount,
            current_price = EXCLUDED.current_price,
            daily_change = EXCLUDED.daily_change,
            market_value = EXCLUDED.market_value,
            profit_loss_rate = EXCLUDED.profit_loss_rate,
            updated_at = NOW()
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
    .execute(&state.pool)
    .await?;

    let inserted = result.rows_affected() as usize;
    let skipped = 0; // UPSERT のため skipped は常に 0
    let elapsed = start.elapsed();

    info!(
        "[asset_balance.bulk_create] 完了: upserted={}, 処理時間={:.2}ms",
        inserted,
        elapsed.as_secs_f64() * 1000.0
    );
    Ok((
        StatusCode::CREATED,
        Json(BulkCreateResponse { inserted, skipped }),
    ))
}

/// 認証ユーザーの保有銘柄を全削除
pub async fn delete_all(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<impl IntoResponse, ApiError> {
    info!("[asset_balance.delete_all] リクエスト受信");
    let result = sqlx::query("DELETE FROM asset_balances WHERE user_id = $1")
        .bind(auth_user.id())
        .execute(&state.pool)
        .await?;

    info!(
        "[asset_balance.delete_all] 完了: {}件削除",
        result.rows_affected()
    );
    Ok((
        StatusCode::OK,
        Json(serde_json::json!({"message": "全ての保有銘柄データを削除しました"})),
    ))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_compilation() {
        assert!(true);
    }
}
