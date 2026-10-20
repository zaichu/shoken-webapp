use crate::db::{Bind, Db};
use crate::errors::ApiError;
use crate::models::stock::Stock;
use crate::services::domain::search_filters::escape_like_pattern;

/// 検索クエリの半角英字を全角に統一する。
/// 銘柄マスタの名前は全角英字を含むため、ユーザー入力の半角を補正して
/// ヒット率を上げる
fn halfwidth_to_fullwidth(c: char) -> char {
    match c {
        'A'..='Z' => char::from_u32(c as u32 - 0x0041 + 0xFF21).unwrap_or(c),
        'a'..='z' => char::from_u32(c as u32 - 0x0061 + 0xFF41).unwrap_or(c),
        _ => c,
    }
}

pub async fn search(pool: &Db, query: &str) -> Result<Stock, ApiError> {
    let search_query: String = query.chars().map(halfwidth_to_fullwidth).collect();
    let escaped = escape_like_pattern(&search_query);
    let stock = crate::db::query_as::<Stock>(
        r#"SELECT date, code, name, market_category,
                  industry_category_33, industry_category_17, size_category
           FROM stock WHERE code = $1 OR name ILIKE $2 ESCAPE '\' LIMIT 1"#,
        vec![Bind::from(search_query), Bind::from(escaped)],
    )
    .fetch_optional(pool)
    .await?
    .ok_or(ApiError::NotFound)?;

    Ok(stock)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_halfwidth_to_fullwidth() {
        for (input, expected) in [
            ('a', 'ａ'),
            ('z', 'ｚ'),
            ('A', 'Ａ'),
            ('Z', 'Ｚ'),
            ('1', '1'),
            ('!', '!'),
            ('あ', 'あ'),
        ] {
            assert_eq!(halfwidth_to_fullwidth(input), expected);
        }
    }
}
