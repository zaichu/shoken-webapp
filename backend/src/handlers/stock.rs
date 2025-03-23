use crate::{
    errors::ApiError,
    extractors::{char_width_converter::halfwidth_to_fullwidth, validated_json::ValidatedJson},
    models::stock::Stock,
    AppState,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

pub async fn select_stock_info(
    Path(code_or_name): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let code_or_name = code_or_name
        .chars()
        .map(|c| halfwidth_to_fullwidth(c))
        .collect::<String>();
    let stock = sqlx::query_as::<_, Stock>(
        "SELECT * FROM stock WHERE code = $1 OR name ILIKE $2 ORDER BY date DESC LIMIT 1",
    )
    .bind(&code_or_name)
    .bind(format!("%{code_or_name}%"))
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound)?;

    Ok((StatusCode::OK, Json(stock)))
}

pub async fn add_stock_info(
    State(state): State<AppState>,
    ValidatedJson(data): ValidatedJson<Stock>,
) -> Result<impl IntoResponse, ApiError> {
    let stock = sqlx::query_as::<_, Stock>(
        "INSERT INTO stock (date, code, name, market_category, industry_code_33, industry_category_33, industry_code_17, industry_category_17, size_code, size_category) 
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) 
         RETURNING *"
    )
    .bind(data.date)
    .bind(&data.code)
    .bind(&data.name)
    .bind(&data.market_category)
    .bind(&data.industry_code_33)
    .bind(&data.industry_category_33)
    .bind(&data.industry_code_17)
    .bind(&data.industry_category_17)
    .bind(&data.size_code)
    .bind(&data.size_category)
    .fetch_one(&state.pool)
    .await?;

    Ok((StatusCode::CREATED, Json(stock)))
}
