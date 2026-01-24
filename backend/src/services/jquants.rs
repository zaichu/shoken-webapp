use crate::errors::ApiError;
use crate::models::jquants::{StatementsQuery, StatementsResponse};
use reqwest::Client;

pub struct JQuantsService;

impl JQuantsService {
    /// 財務諸表を取得（J-Quants API V2）
    pub async fn get_statements(
        client: &Client,
        params: StatementsQuery,
        api_key: &str,
    ) -> Result<StatementsResponse, ApiError> {
        let mut url = format!(
            "https://api.jquants.com/v2/fins/statements?code={}",
            params.code
        );

        if let Some(from) = params.from {
            url.push_str(&format!("&from={}", from));
        }

        if let Some(to) = params.to {
            url.push_str(&format!("&to={}", to));
        }

        tracing::info!("JQuants API V2 URL: {}", url);

        let response = client
            .get(&url)
            .header("x-api-key", api_key)
            .send()
            .await
            .map_err(|e| ApiError::NetworkError(format!("財務諸表取得エラー: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ApiError::ApiError(format!(
                "JQuants財務諸表取得エラー: {}",
                error_text
            )));
        }

        let statements_response = response
            .json::<StatementsResponse>()
            .await
            .map_err(|e| ApiError::NetworkError(format!("財務諸表レスポンス解析エラー: {}", e)))?;

        Ok(statements_response)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_compilation() {
        // モジュールが正常にコンパイルされることを確認
        assert!(true);
    }
}
