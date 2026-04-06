use serde::Deserialize;
use utoipa::ToSchema;

/// 決算サマリー取得パラメータ（J-Quants API V2）
#[derive(Debug, Deserialize, ToSchema)]
pub struct FinSummaryQuery {
    pub code: String,
    pub from: Option<String>,
    pub to: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fin_summary_query() {
        let query = FinSummaryQuery {
            code: "7203".to_string(),
            from: Some("2023-01-01".to_string()),
            to: Some("2023-12-31".to_string()),
        };

        assert_eq!(query.code, "7203");
        assert_eq!(query.from.unwrap(), "2023-01-01");
        assert_eq!(query.to.unwrap(), "2023-12-31");

        let query = FinSummaryQuery {
            code: "7203".to_string(),
            from: None,
            to: None,
        };

        assert_eq!(query.code, "7203");
        assert!(query.from.is_none());
        assert!(query.to.is_none());
    }
}
