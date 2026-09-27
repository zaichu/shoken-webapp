use serde::{Deserialize, Serialize};

// FacetOption はテスト（cfg(test)）からのみ参照されるため bin クレートでは unused 警告が出る
#[allow(unused_imports)]
pub use shared::common::FacetOption;
pub use shared::common::{MessageResponse, PaginatedSearchResponse, SearchFacets};
pub use shared::csv_import::{CsvPreviewResponse, CsvRowError, CsvUploadResponse};
pub use shared::domain::{
    AssetBalance, AssetBalanceSummary, Dividend, DividendSummary, DomesticStock,
    DomesticStockSummary, Mutualfund,
};

// wire 形が DomesticStockSummary と同一なので、serde の実装を wasm に増やさないよう使い回す
pub type MutualfundSummary = DomesticStockSummary;

pub type DomesticStockListResponse =
    PaginatedSearchResponse<DomesticStock, DomesticStockSummary, SearchFacets>;
pub type DividendListResponse = PaginatedSearchResponse<Dividend, DividendSummary, SearchFacets>;
pub type MutualfundListResponse =
    PaginatedSearchResponse<Mutualfund, MutualfundSummary, SearchFacets>;
pub type AssetBalanceListResponse =
    PaginatedSearchResponse<AssetBalance, AssetBalanceSummary, SearchFacets>;

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

fn deserialize_string_id<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<String, D::Error> {
    match serde_json::Value::deserialize(deserializer)? {
        serde_json::Value::String(id) => Ok(id),
        serde_json::Value::Number(id) => Ok(id.to_string()),
        value => Err(serde::de::Error::custom(format!(
            "expected string or number id, got {value}"
        ))),
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct SessionUser {
    #[serde(deserialize_with = "deserialize_string_id")]
    pub id: String,
    pub email: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub picture_url: Option<String>,
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

    // cargo-mutants はパッケージ単体をコピーしてテストを実行するため、コピー内では
    // パッケージ外の docs/openapi.json が存在しない。その場合だけスキップし、
    // 通常実行での欠落・移動は失敗として検出する。
    // コピー先は cargo-mutants 27.x では <tempdir>/cargo-mutants-<crate>-*.tmp で、
    // CARGO_MUTANTS 環境変数も設定されないため、その形状に限定してパスで判定する
    // (cargo-mutants-* を名前に含む通常の checkout と誤判定しないため)。
    fn schemas() -> Option<serde_json::Map<String, serde_json::Value>> {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../docs/openapi.json");
        let Ok(text) = std::fs::read_to_string(path) else {
            let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
            let is_mutants_tmpdir = manifest_dir.parent() == Some(std::env::temp_dir().as_path())
                && manifest_dir.file_name().is_some_and(|name| {
                    let name = name.to_string_lossy();
                    name.starts_with("cargo-mutants-") && name.ends_with(".tmp")
                });
            let in_mutants_sandbox = std::env::var_os("CARGO_MUTANTS").is_some()
                || env!("CARGO_MANIFEST_DIR").contains("mutants.out")
                || is_mutants_tmpdir;
            assert!(in_mutants_sandbox, "docs/openapi.json を読めない: {path}");
            return None;
        };
        Some(
            serde_json::from_str::<serde_json::Value>(&text).expect("openapi.json parses")
                ["components"]["schemas"]
                .as_object()
                .expect("openapi.json に components.schemas がない")
                .clone(),
        )
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
            if let Some(properties) = schema
                .get("properties")
                .and_then(serde_json::Value::as_object)
            {
                for (name, property) in properties {
                    object.insert(name.clone(), sample_value(schemas, property));
                }
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
        ("MessageResponse", roundtrip::<MessageResponse>),
        ("FacetOption", roundtrip::<FacetOption>),
        ("SearchFacets", roundtrip::<SearchFacets>),
        ("CsvRowError", roundtrip::<CsvRowError>),
        ("CsvUploadResponse", roundtrip::<CsvUploadResponse>),
        ("CsvPreviewResponse", roundtrip::<CsvPreviewResponse>),
    ];

    #[test]
    fn contract_matches_openapi() {
        let Some(schemas) = schemas() else {
            eprintln!("docs/openapi.json が見つからないため契約テストをスキップ");
            return;
        };
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
        assert_eq!(user.name.as_deref(), Some("テストユーザー"));
        assert!(user.picture_url.is_none());
    }

    #[test]
    fn session_user_accepts_numeric_id() {
        // 共有E2E（a11y spec）のモックは id を数値で返す
        let user: SessionUser = serde_json::from_str(
            r#"{"id": 1, "email": "test@example.com", "name": "テストユーザー"}"#,
        )
        .expect("numeric id");
        assert_eq!(user.id, "1");
    }
}
