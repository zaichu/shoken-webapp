pub mod client;
pub mod dto;

pub use crate::api::dto::{CsvKindStatsResponse, Stock};
pub use client::{ApiClient, ApiError};

pub async fn fetch_stock(query: &str) -> Result<Stock, ApiError> {
    ApiClient::read_client()
        .get_json::<Stock>("/api/v1/stocks", &[("query", query)])
        .await
}

pub async fn fetch_csv_kind_stats() -> Result<CsvKindStatsResponse, ApiError> {
    ApiClient::read_client()
        .get_json::<CsvKindStatsResponse>("/api/v1/csv-kinds/stats", &[])
        .await
}
