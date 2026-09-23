pub mod client;

pub use client::{ApiClient, ApiError};
pub use crate::dto::Stock;

pub async fn fetch_stock(query: &str) -> Result<Stock, String> {
    let client = ApiClient::read_client();
    match client
        .get_json::<Stock>("/api/v1/stocks", &[("query", query)])
        .await
    {
        Ok(stock) => Ok(stock),
        Err(ApiError::Http { status: 404 }) => {
            Err("指定された銘柄が見つかりませんでした。".to_string())
        }
        Err(ApiError::Parse) => Err(
            "銘柄データの読み込みに失敗しました。データ形式が変更された可能性があります。"
                .to_string(),
        ),
        Err(_) => Err("銘柄情報の取得に失敗しました。".to_string()),
    }
}
