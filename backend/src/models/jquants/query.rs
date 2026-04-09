use serde::Deserialize;
use utoipa::ToSchema;

/// 決算サマリー取得パラメータ（J-Quants API V2）
#[derive(Debug, Deserialize, ToSchema)]
pub struct FinSummaryQuery {
    pub code: String,
    pub from: Option<String>,
    pub to: Option<String>,
}

#[cfg(test)] #[rustfmt::skip] mod tests {
    use super::*;
    #[test]
    fn test_fin_summary_query() {
        let query = FinSummaryQuery { code: "7203".to_string(), from: Some("2023-01-01".to_string()), to: Some("2023-12-31".to_string()) };
        assert_eq!((query.code.as_str(), query.from.as_deref(), query.to.as_deref()), ("7203", Some("2023-01-01"), Some("2023-12-31")));
        let query = FinSummaryQuery { code: "7203".to_string(), from: None, to: None };
        assert_eq!((query.code.as_str(), query.from.as_deref(), query.to.as_deref()), ("7203", None, None));
    }
}
