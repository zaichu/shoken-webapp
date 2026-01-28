use crate::errors::ApiError;
use crate::models::jquants::{FinSummaryQuery, FinSummaryResponse};
use reqwest::Client;

pub struct JQuantsService;

impl JQuantsService {
    /// 決算サマリーを取得（J-Quants API V2）
    /// V2では fins/statements → fins/summary に変更
    pub async fn get_fin_summary(
        client: &Client,
        params: FinSummaryQuery,
        api_key: &str,
    ) -> Result<FinSummaryResponse, ApiError> {
        let mut url = format!(
            "https://api.jquants.com/v2/fins/summary?code={}",
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
                tracing::error!("決算サマリー取得ネットワークエラー: {}", e);
                ApiError::NetworkError(format!("決算サマリー取得エラー: {}", e))
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
                "JQuants決算サマリー取得エラー ({}): {}",
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

        let fin_summary_response: FinSummaryResponse =
            serde_json::from_str(&response_text).map_err(|e| {
                tracing::error!(
                    "決算サマリーレスポンス解析エラー: {} - 本文: {}",
                    e,
                    response_text
                );
                ApiError::NetworkError(format!("決算サマリーレスポンス解析エラー: {}", e))
            })?;

        Ok(fin_summary_response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::jquants::FinSummaryQuery;

    #[test]
    fn test_module_compilation() {
        // モジュールが正常にコンパイルされることを確認
        assert!(true);
    }

    /// 実際のJ-Quants APIを呼び出すテスト
    /// 実行には環境変数 JQUANTS_API_KEY が必要
    /// cargo test test_get_fin_summary_real_api -- --ignored
    #[tokio::test]
    #[ignore]
    async fn test_get_fin_summary_real_api() {
        let api_key = std::env::var("JQUANTS_API_KEY")
            .expect("JQUANTS_API_KEY 環境変数が設定されていません");

        let client = Client::new();
        let params = FinSummaryQuery {
            code: "7203".to_string(), // トヨタ自動車
            from: None,
            to: None,
        };

        let result = JQuantsService::get_fin_summary(&client, params, &api_key).await;

        match result {
            Ok(response) => {
                println!("取得件数: {}", response.data.len());
                if let Some(first) = response.data.first() {
                    println!("銘柄コード: {}", first.local_code);
                    println!("開示日: {}", first.disclosed_date);
                    println!("書類種別: {}", first.type_of_document);
                    println!("当期種別: {:?}", first.type_of_current_period);
                    println!("当期開始日: {:?}", first.current_period_start_date);
                    println!("当期終了日: {:?}", first.current_period_end_date);
                }
                assert!(!response.data.is_empty(), "データが取得できること");
            }
            Err(e) => {
                panic!("API呼び出しエラー: {:?}", e);
            }
        }
    }
}
