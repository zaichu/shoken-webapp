use crate::{
    errors::ApiError,
    extractors::auth::AuthenticatedUser,
    extractors::validated_json::ValidatedJson,
    models::dividend::BulkCreateResponse,
    models::mutualfund::{BulkCreateMutualfundRequest, Mutualfund},
    state::AppState,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

/// 認証ユーザーの投資信託一覧を取得
pub async fn list(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<impl IntoResponse, ApiError> {
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
    .bind(auth_user.id())
    .fetch_all(&state.pool)
    .await?;

    Ok((StatusCode::OK, Json(funds)))
}

/// 投資信託を一括追加（重複はスキップ）
pub async fn bulk_create(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    ValidatedJson(data): ValidatedJson<BulkCreateMutualfundRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let user_id = auth_user.id();
    let mut inserted = 0;
    let mut skipped = 0;

    for item in data.items {
        let result = sqlx::query(
            r#"
            INSERT INTO mutualfunds (user_id, trade_date, settlement_date, fund_name, dividends,
                                     account, shares, exchange_rate, cancellation_unit_price_yen,
                                     cancellation_amount_yen, average_acquisition_price_yen,
                                     realized_profit_and_loss, taxes, realized_profit_and_loss_after_tax)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            ON CONFLICT (user_id, trade_date, fund_name, shares, cancellation_amount_yen)
            DO NOTHING
            "#,
        )
        .bind(user_id)
        .bind(item.trade_date)
        .bind(item.settlement_date)
        .bind(&item.fund_name)
        .bind(&item.dividends)
        .bind(&item.account)
        .bind(item.shares)
        .bind(item.exchange_rate)
        .bind(item.cancellation_unit_price_yen)
        .bind(item.cancellation_amount_yen)
        .bind(item.average_acquisition_price_yen)
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

    Ok((
        StatusCode::CREATED,
        Json(BulkCreateResponse { inserted, skipped }),
    ))
}

/// 認証ユーザーの投資信託を全削除
pub async fn delete_all(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<impl IntoResponse, ApiError> {
    sqlx::query("DELETE FROM mutualfunds WHERE user_id = $1")
        .bind(auth_user.id())
        .execute(&state.pool)
        .await?;

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({"message": "全ての投資信託データを削除しました"})),
    ))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_compilation() {
        assert!(true);
    }
}
