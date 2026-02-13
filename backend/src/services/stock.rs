use crate::errors::ApiError;
use crate::extractors::char_width_converter::halfwidth_to_fullwidth;
use crate::models::stock::Stock;
use sqlx::PgPool;

/// 銘柄コードまたは銘柄名で検索
pub async fn search(pool: &PgPool, query: &str) -> Result<Stock, ApiError> {
    let search_query: String = query.chars().map(halfwidth_to_fullwidth).collect();
    let stock = sqlx::query_as::<_, Stock>(
        "SELECT * FROM stock WHERE code = $1 OR name ILIKE $2 ORDER BY date DESC LIMIT 1",
    )
    .bind(&search_query)
    .bind(format!("%{search_query}%"))
    .fetch_optional(pool)
    .await?
    .ok_or(ApiError::NotFound)?;

    Ok(stock)
}

/// 銘柄情報を追加
pub async fn create(pool: &PgPool, data: &Stock) -> Result<Stock, ApiError> {
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
    .fetch_one(pool)
    .await?;

    Ok(stock)
}
