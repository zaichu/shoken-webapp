use crate::db::{Bind, Db};
use crate::errors::ApiError;
use crate::extractors::char_width_converter::halfwidth_to_fullwidth;
use crate::models::stock::Stock;
use crate::services::domain::search_filters::escape_like_pattern;

pub async fn search(pool: &Db, query: &str) -> Result<Stock, ApiError> {
    let search_query: String = query.chars().map(halfwidth_to_fullwidth).collect();
    let stock = crate::db::query_as::<Stock>(
        r#"SELECT date, code, name, market_category,
                  industry_code_33, industry_category_33, industry_code_17, industry_category_17,
                  size_code, size_category
           FROM stock WHERE code = $1 OR name ILIKE $2 ESCAPE '\' ORDER BY date DESC LIMIT 1"#,
        vec![
            Bind::from(search_query.clone()),
            Bind::from(escape_like_pattern(&search_query)),
        ],
    )
    .fetch_optional(pool)
    .await?
    .ok_or(ApiError::NotFound)?;

    Ok(stock)
}
