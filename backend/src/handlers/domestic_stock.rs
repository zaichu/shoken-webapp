use crate::{
    errors::ApiError,
    extractors::auth::AuthenticatedUser,
    extractors::validated_json::ValidatedJson,
    models::dividend::BulkCreateResponse,
    models::domestic_stock::{BulkCreateDomesticStockRequest, DomesticStock},
    state::AppState,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
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
    info!("[domestic_stock.bulk_create] リクエスト受信: {}件", data.items.len());
    let user_id = auth_user.id();
    let mut inserted = 0;
    let mut skipped = 0;

    for item in data.items {
        let result = sqlx::query(
            r#"
            INSERT INTO domestic_stocks (user_id, trade_date, settlement_date, security_code,
                                         security_name, account, shares, asked_price, proceeds,
                                         purchase_price, realized_profit_and_loss, taxes,
                                         realized_profit_and_loss_after_tax)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            ON CONFLICT (user_id, trade_date, security_code, shares, proceeds)
            DO NOTHING
            "#,
        )
        .bind(user_id)
        .bind(item.trade_date)
        .bind(item.settlement_date)
        .bind(&item.security_code)
        .bind(&item.security_name)
        .bind(&item.account)
        .bind(item.shares)
        .bind(item.asked_price)
        .bind(item.proceeds)
        .bind(item.purchase_price)
        .bind(item.realized_profit_and_loss)
        .bind(item.taxes)
        .bind(item.realized_profit_and_loss_after_tax)
        .execute(&state.pool)
        .await?;

        if result.rows_affected() > 0 {
            inserted += 1;
        } else {
            skipped += 1;
        }
    }

    info!("[domestic_stock.bulk_create] 完了: inserted={}, skipped={}", inserted, skipped);
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

    info!("[domestic_stock.delete_all] 完了: {}件削除", result.rows_affected());
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
