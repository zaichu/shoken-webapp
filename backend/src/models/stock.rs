use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use validator::Validate;

#[derive(Serialize, Deserialize, FromRow, Validate, ToSchema)]
pub struct Stock {
    pub date: NaiveDate,
    #[validate(length(min = 1, max = 10))]
    pub code: String,
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[validate(length(min = 1, max = 50))]
    pub market_category: String,
    #[validate(length(max = 10))]
    pub industry_code_33: Option<String>,
    #[validate(length(max = 100))]
    pub industry_category_33: Option<String>,
    #[validate(length(max = 10))]
    pub industry_code_17: Option<String>,
    #[validate(length(max = 100))]
    pub industry_category_17: Option<String>,
    #[validate(length(max = 10))]
    pub size_code: Option<String>,
    #[validate(length(max = 50))]
    pub size_category: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_stock_validation_valid() {
        let stock = Stock {
            date: NaiveDate::from_ymd_opt(2025, 3, 24).unwrap(),
            code: "1234".to_string(),
            name: "テスト株式会社".to_string(),
            market_category: "プライム".to_string(),
            industry_code_33: Some("123".to_string()),
            industry_category_33: Some("情報・通信業".to_string()),
            industry_code_17: Some("12".to_string()),
            industry_category_17: Some("情報通信".to_string()),
            size_code: Some("10".to_string()),
            size_category: Some("大型株".to_string()),
        };

        assert!(stock.validate().is_ok());
    }

    #[test]
    fn test_stock_validation_invalid_code() {
        let stock = Stock {
            date: NaiveDate::from_ymd_opt(2025, 3, 24).unwrap(),
            code: "".to_string(),
            name: "テスト株式会社".to_string(),
            market_category: "プライム".to_string(),
            industry_code_33: Some("123".to_string()),
            industry_category_33: Some("情報・通信業".to_string()),
            industry_code_17: Some("12".to_string()),
            industry_category_17: Some("情報通信".to_string()),
            size_code: Some("10".to_string()),
            size_category: Some("大型株".to_string()),
        };

        assert!(stock.validate().is_err());
    }

    #[test]
    fn test_stock_validation_invalid_name() {
        let stock = Stock {
            date: NaiveDate::from_ymd_opt(2025, 3, 24).unwrap(),
            code: "1234".to_string(),
            name: "".to_string(),
            market_category: "プライム".to_string(),
            industry_code_33: Some("123".to_string()),
            industry_category_33: Some("情報・通信業".to_string()),
            industry_code_17: Some("12".to_string()),
            industry_category_17: Some("情報通信".to_string()),
            size_code: Some("10".to_string()),
            size_category: Some("大型株".to_string()),
        };

        assert!(stock.validate().is_err());
    }

    #[test]
    fn test_stock_validation_invalid_market_category() {
        let stock = Stock {
            date: NaiveDate::from_ymd_opt(2025, 3, 24).unwrap(),
            code: "1234".to_string(),
            name: "テスト株式会社".to_string(),
            market_category: "".to_string(),
            industry_code_33: Some("123".to_string()),
            industry_category_33: Some("情報・通信業".to_string()),
            industry_code_17: Some("12".to_string()),
            industry_category_17: Some("情報通信".to_string()),
            size_code: Some("10".to_string()),
            size_category: Some("大型株".to_string()),
        };

        assert!(stock.validate().is_err());
    }
}
