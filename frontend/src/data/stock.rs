use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Deserialize, Serialize, Default, Debug)]
pub struct StockData {
    pub date: String,
    pub code: String,
    pub name: String,
    pub market_category: String,
    pub industry_code_33: Option<String>,
    pub industry_category_33: Option<String>,
    pub industry_code_17: Option<String>,
    pub industry_category_17: Option<String>,
    pub size_code: Option<String>,
    pub size_category: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stock_data_serialization() {
        let stock = StockData {
            date: "2025-03-24".into(),
            code: "1234".into(),
            name: "テスト株式会社".into(),
            market_category: "プライム".into(),
            industry_code_33: Some("123".into()),
            industry_category_33: Some("情報・通信業".into()),
            industry_code_17: Some("12".into()),
            industry_category_17: Some("情報通信".into()),
            size_code: Some("10".into()),
            size_category: Some("大型株".into()),
        };

        let serialized = serde_json::to_string(&stock).unwrap();
        let deserialized: StockData = serde_json::from_str(&serialized).unwrap();

        assert_eq!(stock, deserialized);
    }

    #[test]
    fn test_stock_data_default() {
        let default_stock = StockData::default();
        
        assert!(default_stock.date.is_empty());
        assert!(default_stock.code.is_empty());
        assert!(default_stock.name.is_empty());
        assert!(default_stock.market_category.is_empty());
        assert!(default_stock.industry_code_33.is_none());
        assert!(default_stock.industry_category_33.is_none());
        assert!(default_stock.industry_code_17.is_none());
        assert!(default_stock.industry_category_17.is_none());
        assert!(default_stock.size_code.is_none());
        assert!(default_stock.size_category.is_none());
    }
}
