use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

#[derive(Serialize, Deserialize, FromRow, Validate)]
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
