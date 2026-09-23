#[allow(dead_code)]
mod generated {
    include!(concat!(env!("OUT_DIR"), "/dto_gen.rs"));
}

#[cfg(test)]
mod tests {
    use super::generated::*;
    use rust_decimal_macros::dec;

    const WITH_SUMMARY: &str = r#"{
        "data": [
            {
                "id": "550e8400-e29b-41d4-a716-446655440000",
                "trade_date": "2024-01-15",
                "settlement_date": "2024-01-17",
                "security_code": "1301",
                "security_name": "極洋",
                "account": "特定",
                "shares": 100,
                "asked_price": 1500.5,
                "proceeds": 123456789.123456789,
                "purchase_price": 1400.25,
                "realized_profit_and_loss": 10000.1,
                "taxes": 2031.5,
                "realized_profit_and_loss_after_tax": 7968.6,
                "created_at": "2024-01-15T01:23:45Z",
                "updated_at": "2024-01-16T01:23:45Z"
            }
        ],
        "total": 1,
        "page": 1,
        "per_page": 200,
        "summary": {
            "total_realized_profit_and_loss": 10000.1,
            "total_taxes": 2031.5,
            "total_realized_profit_and_loss_after_tax": 7968.6
        }
    }"#;

    const WITH_FACETS: &str = r#"{
        "data": [],
        "total": 0,
        "page": 1,
        "per_page": 200,
        "facets": {
            "accounts": [{"value": "特定", "label": "特定", "count": 3}],
            "securities": [{"value": "1301", "label": "極洋"}],
            "years": null
        }
    }"#;

    #[test]
    fn decimal_precision_without_user_id() {
        let res: DomesticStockListResponse =
            serde_json::from_str(WITH_SUMMARY).expect("deserialize");
        assert_eq!(res.total, 1);
        let item = res.data.first().expect("one row");
        assert_eq!(item.proceeds, dec!(123456789.123456789));
        assert_eq!(item.asked_price, dec!(1500.5));
        assert_eq!(item.shares, dec!(100));
        let summary = res.summary.as_ref().expect("summary present");
        assert_eq!(summary.total_taxes, dec!(2031.5));
        assert!(res.facets.is_none());

        let back = serde_json::to_value(&res).expect("serialize");
        assert!(back["data"][0].get("user_id").is_none());
    }

    #[test]
    fn optional_summary_and_facets() {
        let res: DomesticStockListResponse =
            serde_json::from_str(WITH_FACETS).expect("deserialize");
        assert!(res.summary.is_none());
        let facets = res.facets.as_ref().expect("facets present");
        let accounts = facets.accounts.as_ref().expect("accounts present");
        assert_eq!(accounts[0].value, "特定");
        assert_eq!(accounts[0].count, Some(3));
        let securities = facets.securities.as_ref().expect("securities present");
        assert_eq!(securities[0].count, None);
        assert!(facets.years.is_none());
    }
}
