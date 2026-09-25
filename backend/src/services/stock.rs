use crate::errors::ApiError;
use crate::extractors::char_width_converter::halfwidth_to_fullwidth;
use crate::models::stock::Stock;
use sqlx::PgPool;

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
