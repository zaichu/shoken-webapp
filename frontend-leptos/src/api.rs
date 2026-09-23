pub use crate::dto::Stock;

pub async fn fetch_stock(query: &str) -> Result<Stock, String> {
    let url = format!("/api/v1/stocks?query={}", urlencoding::encode(query));
    let response = gloo_net::http::Request::get(&url)
        .send()
        .await
        .map_err(|_| "銘柄情報の取得に失敗しました。".to_string())?;
    if response.status() == 404 {
        return Err("指定された銘柄が見つかりませんでした。".to_string());
    }
    if !response.ok() {
        return Err("銘柄情報の取得に失敗しました。".to_string());
    }
    response.json::<Stock>().await.map_err(|_| {
        "銘柄データの読み込みに失敗しました。データ形式が変更された可能性があります。".to_string()
    })
}
