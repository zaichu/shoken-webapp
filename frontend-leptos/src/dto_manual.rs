// 試作のためページ未接続。接続時に外す
#![allow(dead_code)]

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct DomesticStockItem {
    pub id: String,
    pub trade_date: String,
    pub settlement_date: String,
    pub security_code: String,
    pub security_name: String,
    pub account: String,
    pub shares: Decimal,
    pub asked_price: Decimal,
    pub proceeds: Decimal,
    pub purchase_price: Decimal,
    pub realized_profit_and_loss: Decimal,
    pub taxes: Decimal,
    pub realized_profit_and_loss_after_tax: Decimal,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct DomesticStockSummary {
    pub total_realized_profit_and_loss: Decimal,
    pub total_taxes: Decimal,
    pub total_realized_profit_and_loss_after_tax: Decimal,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct FacetOption {
    pub value: String,
    pub label: String,
    pub count: Option<i64>,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct SearchFacets {
    #[serde(default)]
    pub accounts: Option<Vec<FacetOption>>,
    #[serde(default)]
    pub funds: Option<Vec<FacetOption>>,
    #[serde(default)]
    pub products: Option<Vec<FacetOption>>,
    #[serde(default)]
    pub securities: Option<Vec<FacetOption>>,
    #[serde(default)]
    pub year_months: Option<Vec<FacetOption>>,
    #[serde(default)]
    pub years: Option<Vec<FacetOption>>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct DomesticStockListResponse {
    pub data: Vec<DomesticStockItem>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    #[serde(default)]
    pub summary: Option<DomesticStockSummary>,
    #[serde(default)]
    pub facets: Option<SearchFacets>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use std::collections::BTreeSet;

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

    const OPENAPI: &str = include_str!("../../docs/openapi.json");

    const WRAPPER: &str = "PaginatedSearchResponse_DomesticStock_DomesticStockSummary_SearchFacets";

    fn schemas() -> serde_json::Map<String, serde_json::Value> {
        serde_json::from_str::<serde_json::Value>(OPENAPI).expect("openapi.json parses")
            ["components"]["schemas"]
            .as_object()
            .expect("components/schemas")
            .clone()
    }

    fn keys(value: &serde_json::Value) -> BTreeSet<String> {
        value.as_object().expect("object").keys().cloned().collect()
    }

    fn required(value: &serde_json::Value) -> BTreeSet<String> {
        value["required"]
            .as_array()
            .expect("required")
            .iter()
            .map(|v| v.as_str().expect("string").to_string())
            .collect()
    }

    fn set(names: &[&str]) -> BTreeSet<String> {
        names.iter().map(ToString::to_string).collect()
    }

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

    #[test]
    fn contract_matches_openapi() {
        let schemas = schemas();
        let item_fields = set(&[
            "id",
            "trade_date",
            "settlement_date",
            "security_code",
            "security_name",
            "account",
            "shares",
            "asked_price",
            "proceeds",
            "purchase_price",
            "realized_profit_and_loss",
            "taxes",
            "realized_profit_and_loss_after_tax",
            "created_at",
            "updated_at",
        ]);
        let item = &schemas["DomesticStock"];
        assert_eq!(required(item), item_fields);
        assert_eq!(keys(&item["properties"]), item_fields);

        for field in [
            "shares",
            "asked_price",
            "proceeds",
            "purchase_price",
            "realized_profit_and_loss",
            "taxes",
            "realized_profit_and_loss_after_tax",
        ] {
            assert_eq!(item["properties"][field]["type"], "number", "{field}");
        }

        let summary_fields = set(&[
            "total_realized_profit_and_loss",
            "total_taxes",
            "total_realized_profit_and_loss_after_tax",
        ]);
        let summary = &schemas["DomesticStockSummary"];
        assert_eq!(required(summary), summary_fields);
        assert_eq!(keys(&summary["properties"]), summary_fields);

        let wrapper = &schemas[WRAPPER];
        assert_eq!(
            required(wrapper),
            set(&["data", "total", "page", "per_page"])
        );
        assert_eq!(
            keys(&wrapper["properties"]),
            set(&["data", "total", "page", "per_page", "summary", "facets"])
        );
        assert_eq!(
            required(&wrapper["properties"]["data"]["items"]),
            item_fields
        );

        let facet = &schemas["FacetOption"];
        assert_eq!(required(facet), set(&["value", "label"]));
    }
}
