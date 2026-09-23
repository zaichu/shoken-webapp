use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Stock {
    pub code: String,
    pub name: String,
    pub date: String,
    pub market_category: String,
    #[serde(default)]
    pub industry_code_33: Option<String>,
    #[serde(default)]
    pub industry_category_33: Option<String>,
    #[serde(default)]
    pub industry_code_17: Option<String>,
    #[serde(default)]
    pub industry_category_17: Option<String>,
    #[serde(default)]
    pub size_code: Option<String>,
    #[serde(default)]
    pub size_category: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct DomesticStock {
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
pub struct DomesticStockListResponse {
    pub data: Vec<DomesticStock>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    #[serde(default)]
    pub summary: Option<DomesticStockSummary>,
    #[serde(default)]
    pub facets: Option<SearchFacets>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Dividend {
    pub id: String,
    pub settlement_date: String,
    pub product: String,
    pub account: String,
    pub security_code: String,
    pub security_name: String,
    pub unit_price: Decimal,
    pub shares: Decimal,
    pub dividends_before_tax: Decimal,
    pub taxes: Decimal,
    pub net_amount_received: Decimal,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct DividendSummary {
    pub total_dividends_before_tax: Decimal,
    pub total_taxes: Decimal,
    pub total_net_amount_received: Decimal,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct DividendListResponse {
    pub data: Vec<Dividend>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    #[serde(default)]
    pub summary: Option<DividendSummary>,
    #[serde(default)]
    pub facets: Option<SearchFacets>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Mutualfund {
    pub id: String,
    pub trade_date: String,
    pub settlement_date: String,
    pub fund_name: String,
    pub account: String,
    pub shares: Decimal,
    pub exchange_rate: Decimal,
    pub cancellation_unit_price_yen: Decimal,
    pub cancellation_amount_yen: Decimal,
    pub average_acquisition_price_yen: Decimal,
    pub realized_profit_and_loss: Decimal,
    pub taxes: Decimal,
    pub realized_profit_and_loss_after_tax: Decimal,
    #[serde(default)]
    pub dividends: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct MutualfundSummary {
    pub total_realized_profit_and_loss: Decimal,
    pub total_taxes: Decimal,
    pub total_realized_profit_and_loss_after_tax: Decimal,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct MutualfundListResponse {
    pub data: Vec<Mutualfund>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    #[serde(default)]
    pub summary: Option<MutualfundSummary>,
    #[serde(default)]
    pub facets: Option<SearchFacets>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct AssetBalance {
    pub id: String,
    pub security_code: String,
    pub security_name: String,
    pub shares: Decimal,
    pub executing_shares: Decimal,
    pub average_purchase_price: Decimal,
    pub total_purchase_amount: Decimal,
    pub current_price: Decimal,
    pub daily_change: Decimal,
    pub market_value: Decimal,
    pub profit_loss_rate: Decimal,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct AssetBalanceSummary {
    pub total_market_value: Decimal,
    pub total_purchase_amount: Decimal,
    pub total_daily_change: Decimal,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct AssetBalanceListResponse {
    pub data: Vec<AssetBalance>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    #[serde(default)]
    pub summary: Option<AssetBalanceSummary>,
    #[serde(default)]
    pub facets: Option<SearchFacets>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct SessionUser {
    pub id: String,
    pub email: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub picture_url: Option<String>,
}

impl SessionUser {
    pub fn display_name(&self) -> String {
        match &self.name {
            Some(name) if !name.is_empty() => name.clone(),
            _ if !self.email.is_empty() => self.email.clone(),
            _ => self.id.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct FacetOption {
    pub value: String,
    pub label: String,
    #[serde(default)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use std::collections::BTreeSet;

    fn roundtrip<T>(value: serde_json::Value) -> Result<serde_json::Value, serde_json::Error>
    where
        T: for<'de> Deserialize<'de> + Serialize,
    {
        let parsed: T = serde_json::from_value(value)?;
        serde_json::to_value(&parsed)
    }

    fn schemas() -> serde_json::Map<String, serde_json::Value> {
        serde_json::from_str::<serde_json::Value>(include_str!("../../docs/openapi.json"))
            .expect("openapi.json parses")["components"]["schemas"]
            .as_object()
            .expect("components/schemas")
            .clone()
    }

    fn resolve<'a>(
        schemas: &'a serde_json::Map<String, serde_json::Value>,
        schema: &'a serde_json::Value,
    ) -> &'a serde_json::Value {
        match schema.get("$ref").and_then(serde_json::Value::as_str) {
            Some(reference) => {
                let name = reference
                    .strip_prefix("#/components/schemas/")
                    .expect("local schema ref");
                &schemas[name]
            }
            None => schema,
        }
    }

    fn sample_primitive(schema: &serde_json::Value) -> serde_json::Value {
        let format = schema.get("format").and_then(serde_json::Value::as_str);
        match schema.get("type").and_then(serde_json::Value::as_str) {
            Some("string") if format == Some("date") => {
                serde_json::Value::String("2024-01-15".into())
            }
            Some("string") if format == Some("date-time") => {
                serde_json::Value::String("2024-01-15T01:23:45Z".into())
            }
            Some("string") if format == Some("uuid") => {
                serde_json::Value::String("550e8400-e29b-41d4-a716-446655440000".into())
            }
            Some("string") => serde_json::Value::String("x".into()),
            Some("number") => serde_json::from_str("123456789.123456789").expect("fraction"),
            Some("integer") => serde_json::json!(9007199254740993i64),
            Some("boolean") => serde_json::Value::Bool(true),
            _ => panic!("unexpected primitive: {schema}"),
        }
    }

    fn sample_value(
        schemas: &serde_json::Map<String, serde_json::Value>,
        schema: &serde_json::Value,
    ) -> serde_json::Value {
        let schema = resolve(schemas, schema);
        let types = base_types(schema);
        if types.contains(&"object") || schema.get("properties").is_some() {
            let mut object = serde_json::Map::new();
            for (name, property) in schema["properties"].as_object().expect("properties") {
                object.insert(name.clone(), sample_value(schemas, property));
            }
            return serde_json::Value::Object(object);
        }
        if types.contains(&"array") {
            let item = sample_value(schemas, &schema["items"]);
            return serde_json::Value::Array(vec![item]);
        }
        let non_null = types.iter().find(|t| **t != "null").expect("non-null type");
        sample_primitive(&serde_json::json!({"type": non_null, "format": schema.get("format")}))
    }

    fn base_types(schema: &serde_json::Value) -> Vec<&str> {
        match &schema["type"] {
            serde_json::Value::String(single) => vec![single.as_str()],
            serde_json::Value::Array(multiple) => multiple
                .iter()
                .map(|v| v.as_str().expect("type name"))
                .collect(),
            _ => panic!("no type: {schema}"),
        }
    }

    fn check_contract(
        schemas: &serde_json::Map<String, serde_json::Value>,
        name: &str,
        roundtrip: fn(serde_json::Value) -> Result<serde_json::Value, serde_json::Error>,
    ) {
        let schema = resolve(schemas, &schemas[name]);
        let sample = sample_value(schemas, schema);
        assert_eq!(
            roundtrip(sample.clone()).expect("roundtrip"),
            sample,
            "{name} roundtrip"
        );

        let required: BTreeSet<&str> = schema
            .get("required")
            .map(|required| {
                required
                    .as_array()
                    .expect("required")
                    .iter()
                    .map(|v| v.as_str().expect("string"))
                    .collect()
            })
            .unwrap_or_default();
        for (field, property) in schema["properties"].as_object().expect("properties") {
            let mut partial = sample.clone();
            partial.as_object_mut().expect("object").remove(field);
            assert_eq!(
                roundtrip(partial).is_ok(),
                !required.contains(field.as_str()),
                "{name}.{field} requiredness"
            );
            if base_types(resolve(schemas, property)).contains(&"integer") {
                let mut fraction = sample.clone();
                fraction[field] = serde_json::json!(1.5);
                assert!(
                    roundtrip(fraction).is_err(),
                    "{name}.{field} rejects fraction"
                );
            }
        }
    }

    type Roundtrip = fn(serde_json::Value) -> Result<serde_json::Value, serde_json::Error>;

    const TABLE: &[(&str, Roundtrip)] = &[
        ("Stock", roundtrip::<Stock>),
        ("DomesticStock", roundtrip::<DomesticStock>),
        ("DomesticStockSummary", roundtrip::<DomesticStockSummary>),
        (
            "PaginatedSearchResponse_DomesticStock_DomesticStockSummary_SearchFacets",
            roundtrip::<DomesticStockListResponse>,
        ),
        ("Dividend", roundtrip::<Dividend>),
        ("DividendSummary", roundtrip::<DividendSummary>),
        (
            "PaginatedSearchResponse_Dividend_DividendSummary_SearchFacets",
            roundtrip::<DividendListResponse>,
        ),
        ("Mutualfund", roundtrip::<Mutualfund>),
        ("MutualfundSummary", roundtrip::<MutualfundSummary>),
        (
            "PaginatedSearchResponse_Mutualfund_MutualfundSummary_SearchFacets",
            roundtrip::<MutualfundListResponse>,
        ),
        ("AssetBalance", roundtrip::<AssetBalance>),
        ("AssetBalanceSummary", roundtrip::<AssetBalanceSummary>),
        (
            "PaginatedSearchResponse_AssetBalance_AssetBalanceSummary_SearchFacets",
            roundtrip::<AssetBalanceListResponse>,
        ),
        ("UserResponse", roundtrip::<SessionUser>),
        ("FacetOption", roundtrip::<FacetOption>),
        ("SearchFacets", roundtrip::<SearchFacets>),
    ];

    #[test]
    fn contract_matches_openapi() {
        let schemas = schemas();
        for (name, roundtrip) in TABLE {
            check_contract(&schemas, name, *roundtrip);
        }
        for (wrapper, item, summary) in [
            (
                "PaginatedSearchResponse_DomesticStock_DomesticStockSummary_SearchFacets",
                "DomesticStock",
                "DomesticStockSummary",
            ),
            (
                "PaginatedSearchResponse_Dividend_DividendSummary_SearchFacets",
                "Dividend",
                "DividendSummary",
            ),
            (
                "PaginatedSearchResponse_Mutualfund_MutualfundSummary_SearchFacets",
                "Mutualfund",
                "MutualfundSummary",
            ),
            (
                "PaginatedSearchResponse_AssetBalance_AssetBalanceSummary_SearchFacets",
                "AssetBalance",
                "AssetBalanceSummary",
            ),
        ] {
            let properties = &schemas[wrapper]["properties"];
            for (inline, named) in [
                (&properties["data"]["items"], item),
                (&properties["summary"], summary),
                (&properties["facets"], "SearchFacets"),
            ] {
                assert_eq!(
                    sample_value(&schemas, inline),
                    sample_value(&schemas, &schemas[named]),
                    "{wrapper} inline matches {named}"
                );
            }
        }
    }

    #[test]
    fn decimal_precision_without_user_id() {
        let res: DomesticStockListResponse = serde_json::from_str(
            r#"{
                "data": [{
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
                }],
                "total": 1,
                "page": 1,
                "per_page": 200,
                "summary": {
                    "total_realized_profit_and_loss": 10000.1,
                    "total_taxes": 2031.5,
                    "total_realized_profit_and_loss_after_tax": 7968.6
                }
            }"#,
        )
        .expect("deserialize");
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
        let res: DomesticStockListResponse = serde_json::from_str(
            r#"{
                "data": [],
                "total": 0,
                "page": 1,
                "per_page": 200,
                "facets": {
                    "accounts": [{"value": "特定", "label": "特定", "count": 3}],
                    "securities": [{"value": "1301", "label": "極洋"}],
                    "years": null
                }
            }"#,
        )
        .expect("deserialize");
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
    fn other_endpoints_deserialize_numbers() {
        let dividends: DividendListResponse = serde_json::from_str(
            r#"{
                "data": [{
                    "id": "550e8400-e29b-41d4-a716-446655440000",
                    "settlement_date": "2024-03-01",
                    "product": "特定口座",
                    "account": "SBI証券",
                    "security_code": "7203",
                    "security_name": "トヨタ自動車",
                    "unit_price": 30.0,
                    "shares": 100,
                    "dividends_before_tax": 3000,
                    "taxes": 609,
                    "net_amount_received": 2391,
                    "created_at": "2024-03-01T00:00:00Z",
                    "updated_at": "2024-03-01T00:00:00Z"
                }],
                "total": 1,
                "page": 1,
                "per_page": 1
            }"#,
        )
        .expect("dividends");
        let row = dividends.data.first().expect("one row");
        assert_eq!(row.unit_price, dec!(30.0));
        assert_eq!(row.net_amount_received, dec!(2391));

        let funds: MutualfundListResponse = serde_json::from_str(
            r#"{
                "data": [{
                    "id": "550e8400-e29b-41d4-a716-446655440000",
                    "trade_date": "2024-01-15",
                    "settlement_date": "2024-01-17",
                    "fund_name": "eMAXIS Slim 全世界株式",
                    "account": "楽天証券",
                    "shares": 10000,
                    "exchange_rate": 150.25,
                    "cancellation_unit_price_yen": 12345,
                    "cancellation_amount_yen": 120000,
                    "average_acquisition_price_yen": 11000.5,
                    "realized_profit_and_loss": 12000,
                    "taxes": 2437,
                    "realized_profit_and_loss_after_tax": 9563,
                    "created_at": "2024-01-15T01:23:45Z",
                    "updated_at": "2024-01-16T01:23:45Z"
                }],
                "total": 1,
                "page": 1,
                "per_page": 1
            }"#,
        )
        .expect("mutualfunds");
        let fund = funds.data.first().expect("one row");
        assert_eq!(fund.exchange_rate, dec!(150.25));
        assert!(fund.dividends.is_none());

        let balances: AssetBalanceListResponse = serde_json::from_str(
            r#"{
                "data": [{
                    "id": "550e8400-e29b-41d4-a716-446655440000",
                    "security_code": "7974",
                    "security_name": "任天堂",
                    "shares": 100,
                    "executing_shares": 0,
                    "average_purchase_price": 5000.5,
                    "total_purchase_amount": 500050,
                    "current_price": 6000,
                    "daily_change": 100.25,
                    "market_value": 600000,
                    "profit_loss_rate": 20.5,
                    "created_at": "2024-01-15T01:23:45Z",
                    "updated_at": "2024-01-16T01:23:45Z"
                }],
                "total": 1,
                "page": 1,
                "per_page": 200,
                "summary": {
                    "total_market_value": 600000,
                    "total_purchase_amount": 500050,
                    "total_daily_change": 100.25
                }
            }"#,
        )
        .expect("asset-balances");
        let balance = balances.data.first().expect("one row");
        assert_eq!(balance.market_value, dec!(600000));
        assert_eq!(balance.profit_loss_rate, dec!(20.5));
        assert_eq!(
            balances
                .summary
                .as_ref()
                .expect("summary")
                .total_daily_change,
            dec!(100.25)
        );

        let stock: Stock = serde_json::from_str(
            r#"{
                "date": "2024-03-01",
                "code": "7974",
                "name": "任天堂",
                "market_category": "プライム",
                "industry_code_33": "37",
                "industry_category_33": "情報・通信業"
            }"#,
        )
        .expect("stock");
        assert_eq!(stock.code, "7974");
        assert!(stock.size_code.is_none());

        let user: SessionUser = serde_json::from_str(
            r#"{"id": "00000000-0000-0000-0000-000000000002", "email": "test@example.com", "name": "テストユーザー"}"#,
        )
        .expect("session");
        assert_eq!(user.display_name(), "テストユーザー");
        assert!(user.picture_url.is_none());
    }
}
