use crate::{
    errors::ApiError,
    extractors::auth::AuthenticatedUser,
    extractors::validated_json::ValidatedJson,
    models::dividend::{BulkCreateDividendRequest, BulkCreateResponse, Dividend},
    state::AppState,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

/// 認証ユーザーの配当金一覧を取得
pub async fn list(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<impl IntoResponse, ApiError> {
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
    .bind(auth_user.id())
    .fetch_all(&state.pool)
    .await?;

    Ok((StatusCode::OK, Json(dividends)))
}

/// 配当金を一括追加（重複はスキップ）
pub async fn bulk_create(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    ValidatedJson(data): ValidatedJson<BulkCreateDividendRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let user_id = auth_user.id();
    let mut inserted = 0;
    let mut skipped = 0;

    for item in data.items {
        let result = sqlx::query(
            r#"
            INSERT INTO dividends (user_id, settlement_date, product, account, security_code,
                                   security_name, unit_price, shares, dividends_before_tax,
                                   taxes, net_amount_received)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            ON CONFLICT (user_id, settlement_date, security_code, shares, dividends_before_tax)
            DO NOTHING
            "#,
        )
        .bind(user_id)
        .bind(item.settlement_date)
        .bind(&item.product)
        .bind(&item.account)
        .bind(&item.security_code)
        .bind(&item.security_name)
        .bind(item.unit_price)
        .bind(item.shares)
        .bind(item.dividends_before_tax)
        .bind(item.taxes)
        .bind(item.net_amount_received)
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

/// 認証ユーザーの配当金を全削除
pub async fn delete_all(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<impl IntoResponse, ApiError> {
    sqlx::query("DELETE FROM dividends WHERE user_id = $1")
        .bind(auth_user.id())
        .execute(&state.pool)
        .await?;

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({"message": "全ての配当金データを削除しました"})),
    ))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_compilation() {
        assert!(true);
    }
}
