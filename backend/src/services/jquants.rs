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
            .map_err(|e| {
                tracing::error!("財務諸表取得ネットワークエラー: {}", e);
                ApiError::NetworkError(format!("財務諸表取得エラー: {}", e))
            })?;

        let status = response.status();
        tracing::info!("JQuants API レスポンスステータス: {}", status);

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            tracing::error!(
                "JQuants API エラー - ステータス: {}, 本文: {}",
                status,
                error_text
            );
            return Err(ApiError::ApiError(format!(
                "JQuants財務諸表取得エラー ({}): {}",
                status,
                error_text
            )));
        }

        // デシリアライズ前にレスポンス本文を取得（デバッグ用）
        let response_text = response.text().await.map_err(|e| {
            tracing::error!("レスポンス本文取得エラー: {}", e);
            ApiError::NetworkError(format!("レスポンス読み取りエラー: {}", e))
        })?;

        tracing::debug!("JQuants API レスポンス本文: {}", response_text);

        let statements_response: StatementsResponse =
            serde_json::from_str(&response_text).map_err(|e| {
                tracing::error!(
                    "財務諸表レスポンス解析エラー: {} - 本文: {}",
                    e,
                    response_text
                );
                ApiError::NetworkError(format!("財務諸表レスポンス解析エラー: {}", e))
            })?;

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
