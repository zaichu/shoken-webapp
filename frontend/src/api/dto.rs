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

#[cfg(test)]
pub type DomesticStockListResponse =
    PaginatedSearchResponse<DomesticStock, DomesticStockSummary, SearchFacets>;
pub type DividendListResponse = PaginatedSearchResponse<Dividend, DividendSummary, SearchFacets>;
#[cfg(test)]
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
mod tests;
