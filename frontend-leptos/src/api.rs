use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Deserialize)]
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
