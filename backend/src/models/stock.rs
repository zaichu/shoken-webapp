use crate::models::common::validate_length_field;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use shared::value::SecurityCode;
use utoipa::ToSchema;
use validator::{Validate, ValidationErrors};

#[derive(Serialize, Deserialize, ToSchema)]
pub struct Stock {
    pub date: NaiveDate,
    #[schema(value_type = String)]
    pub code: SecurityCode,
    pub name: String,
    pub market_category: String,
    pub industry_code_33: Option<String>,
    pub industry_category_33: Option<String>,
    pub industry_code_17: Option<String>,
    pub industry_category_17: Option<String>,
    pub size_code: Option<String>,
    pub size_category: Option<String>,
}

impl crate::db::FromRow for Stock {
    fn from_row(row: &crate::db::Row) -> Result<Self, crate::db::DbError> {
        Ok(Self {
            date: row.try_get("date")?,
            code: row.try_get("code")?,
            name: row.try_get("name")?,
            market_category: row.try_get("market_category")?,
            industry_code_33: row.try_get("industry_code_33")?,
            industry_category_33: row.try_get("industry_category_33")?,
            industry_code_17: row.try_get("industry_code_17")?,
            industry_category_17: row.try_get("industry_category_17")?,
            size_code: row.try_get("size_code")?,
            size_category: row.try_get("size_category")?,
        })
    }
}

impl Validate for Stock {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();
        validate_length_field(&mut errors, "name", &self.name, Some(1), Some(100));
        validate_length_field(
            &mut errors,
            "market_category",
            &self.market_category,
            Some(1),
            Some(50),
        );
        validate_length_field(
            &mut errors,
            "industry_code_33",
            &self.industry_code_33,
            None,
            Some(10),
        );
        validate_length_field(
            &mut errors,
            "industry_category_33",
            &self.industry_category_33,
            None,
            Some(100),
        );
        validate_length_field(
            &mut errors,
            "industry_code_17",
            &self.industry_code_17,
            None,
            Some(10),
        );
        validate_length_field(
            &mut errors,
            "industry_category_17",
            &self.industry_category_17,
            None,
            Some(100),
        );
        validate_length_field(&mut errors, "size_code", &self.size_code, None, Some(10));
        validate_length_field(
            &mut errors,
            "size_category",
            &self.size_category,
            None,
            Some(50),
        );
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
#[cfg(test)]
mod tests {
    use {super::*, chrono::NaiveDate};
    fn make_stock(code: &str, name: &str, market_category: &str) -> Stock {
        Stock {
            date: NaiveDate::from_ymd_opt(2025, 3, 24).expect("有効な日付 2025-03-24"),
            code: code.parse().expect("有効な銘柄コード"),
            name: name.to_string(),
            market_category: market_category.to_string(),
            industry_code_33: Some("123".to_string()),
            industry_category_33: Some("情報・通信業".to_string()),
            industry_code_17: Some("12".to_string()),
            industry_category_17: Some("情報通信".to_string()),
            size_code: Some("10".to_string()),
            size_category: Some("大型株".to_string()),
        }
    }
    #[test]
    fn test_stock_validation() {
        assert!(
            make_stock("1234", "テスト株式会社", "プライム")
                .validate()
                .is_ok()
        );
        // code の形式は SecurityCode の serde(try_from) が検証する
        assert!("".parse::<SecurityCode>().is_err());
        assert!(make_stock("1234", "", "プライム").validate().is_err());
        assert!(make_stock("1234", "テスト株式会社", "").validate().is_err());
    }
}
