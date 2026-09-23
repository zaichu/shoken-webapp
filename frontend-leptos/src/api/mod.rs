pub mod client;

pub use client::{ApiClient, ApiError};
pub use crate::dto::Stock;

pub async fn fetch_stock(query: &str) -> Result<Stock, ApiError> {
    ApiClient::read_client()
        .get_json::<Stock>("/api/v1/stocks", &[("query", query)])
        .await
}
