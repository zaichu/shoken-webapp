use crate::errors::ApiError;
use crate::models::market_data::financial_statement::FinancialStatementsQuery;
use crate::models::market_data::providers::jquants::FinSummaryResponse;
use reqwest::Client;

const FIN_SUMMARY_URL: &str = "https://api.jquants.com/v2/fins/summary";

#[derive(Clone)]
pub struct JQuantsClient {
    client: Client,
    api_key: String,
    base_url: String,
}

impl JQuantsClient {
    pub fn new(client: Client, api_key: String) -> Self {
        Self {
            client,
            api_key,
            base_url: FIN_SUMMARY_URL.to_string(),
        }
    }

    #[cfg(test)]
    pub fn with_base_url(client: Client, api_key: String, base_url: String) -> Self {
        Self {
            client,
            api_key,
            base_url,
        }
    }

    pub async fn get_fin_summary(
        &self,
        params: FinancialStatementsQuery,
    ) -> Result<FinSummaryResponse, ApiError> {
        let mut query_params: Vec<(&str, &str)> = vec![("code", &params.code)];
        if let Some(from) = params.from.as_deref() {
            query_params.push(("from", from));
        }
        if let Some(to) = params.to.as_deref() {
            query_params.push(("to", to));
        }

        tracing::info!("JQuants API V2 リクエスト: code={}", params.code);

        let response = self
            .client
            .get(&self.base_url)
            .query(&query_params)
            .header("x-api-key", &self.api_key)
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
            if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                return Err(ApiError::RateLimitError(format!(
                    "J-Quants APIのレート制限に達しました: {}",
                    error_text
                )));
            }
            return Err(ApiError::ApiError(format!(
                "JQuants決算サマリー取得エラー ({}): {}",
                status, error_text
            )));
        }

        let response_text = response.text().await.map_err(|e| {
            tracing::error!("レスポンス本文取得エラー: {}", e);
            ApiError::NetworkError(format!("レスポンス読み取りエラー: {}", e))
        })?;

        tracing::debug!("JQuants API レスポンス本文: {}", response_text);

        let fin_summary_response: FinSummaryResponse = serde_json::from_str(&response_text)
            .map_err(|e| {
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
    use serde_json::json;
    use wiremock::{
        matchers::{header, method, path, query_param},
        Mock, MockServer, ResponseTemplate,
    };

    async fn fetch_fin_summary(base_url: &str, code: &str) -> FinSummaryResponse {
        let api_key =
            std::env::var("JQUANTS_API_KEY").expect("JQUANTS_API_KEY 環境変数が設定されていません");
        let params = FinancialStatementsQuery {
            code: code.to_string(),
            from: None,
            to: None,
        };

        JQuantsClient::with_base_url(Client::new(), api_key, base_url.to_string())
            .get_fin_summary(params)
            .await
            .unwrap_or_else(|e| panic!("API呼び出しエラー: {:?}", e))
    }

    async fn fetch_mock_fin_summary(server: &MockServer) -> Result<FinSummaryResponse, ApiError> {
        let params = FinancialStatementsQuery {
            code: "7203".to_string(),
            from: None,
            to: None,
        };
        let base_url = format!("{}/v2/fins/summary", server.uri());

        JQuantsClient::with_base_url(Client::new(), "test-api-key".to_string(), base_url)
            .get_fin_summary(params)
            .await
    }

    fn log_summary_overview(response: &FinSummaryResponse) {
        println!("取得件数: {}", response.data.len());
        if let Some(first) = response.data.first() {
            println!("銘柄コード: {}", first.local_code);
            println!("開示日: {}", first.disclosed_date);
            println!("書類種別: {}", first.type_of_document);
        }
    }

    fn log_dividend_summaries(response: &FinSummaryResponse) {
        println!("取得件数: {}", response.data.len());
        for summary in &response.data {
            println!("---");
            println!("開示日: {}", summary.disclosed_date);
            println!("書類種別: {}", summary.type_of_document);
            println!(
                "年間配当実績(DivAnn): {:?}",
                summary.result_dividend_per_share_annual
            );
            println!(
                "年間配当予想(FDivAnn): {:?}",
                summary.forecast_dividend_per_share_annual
            );
            println!(
                "年間配当来期予想(NxFDivAnn): {:?}",
                summary.next_year_forecast_dividend_per_share_annual
            );
        }
    }

    #[tokio::test]
    async fn test_get_fin_summary_date_query_and_dividend() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v2/fins/summary"))
            .and(query_param("code", "7203"))
            .and(query_param("from", "2024-01-01"))
            .and(query_param("to", "2024-12-31"))
            .and(header("x-api-key", "test-api-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "data": [{
                    "DiscDate": "2024-05-10", "Code": "7203", "DocType": "FY",
                    "Sales": "123456789", "CurPerType": "FY",
                    "NxFDivAnn": "", "FDivAnn": "45.25", "DivAnn": "40.00",
                    "UnknownField": {"nested": [1, 2]}
                }],
                "pagination_key": "next-page"
            })))
            .expect(1)
            .mount(&server)
            .await;
        let client = JQuantsClient::with_base_url(
            Client::new(),
            "test-api-key".to_string(),
            format!("{}/v2/fins/summary", server.uri()),
        );
        let response = client
            .get_fin_summary(FinancialStatementsQuery {
                code: "7203".to_string(),
                from: Some("2024-01-01".to_string()),
                to: Some("2024-12-31".to_string()),
            })
            .await
            .expect("配当レスポンス");
        assert_eq!(response.pagination_key.as_deref(), Some("next-page"));
        assert_eq!(
            response.data[0]
                .forecast_dividend_per_share_annual
                .as_deref(),
            Some("45.25")
        );
    }

    #[tokio::test]
    async fn test_get_fin_summary_rate_limit() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v2/fins/summary"))
            .and(query_param("code", "7203"))
            .respond_with(ResponseTemplate::new(429).set_body_string("rate limited"))
            .mount(&server)
            .await;

        let error = fetch_mock_fin_summary(&server)
            .await
            .expect_err("429 では RateLimitError を返すこと");

        assert!(matches!(
            error,
            ApiError::RateLimitError(message) if message.contains("rate limited")
        ));
    }

    #[tokio::test]
    async fn test_get_fin_summary_api_error() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v2/fins/summary"))
            .and(query_param("code", "7203"))
            .respond_with(ResponseTemplate::new(500).set_body_string("upstream failed"))
            .mount(&server)
            .await;

        let error = fetch_mock_fin_summary(&server)
            .await
            .expect_err("非 2xx では ApiError を返すこと");

        assert!(matches!(
            error,
            ApiError::ApiError(message) if message.contains("upstream failed")
        ));
    }

    #[tokio::test]
    async fn test_get_fin_summary_deserialize_error() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v2/fins/summary"))
            .and(query_param("code", "7203"))
            .respond_with(ResponseTemplate::new(200).set_body_string("{invalid"))
            .mount(&server)
            .await;

        let error = fetch_mock_fin_summary(&server)
            .await
            .expect_err("不正 JSON では NetworkError を返すこと");

        assert!(matches!(
            error,
            ApiError::NetworkError(message) if message.contains("解析エラー")
        ));
    }

    #[tokio::test]
    #[ignore = "requires JQUANTS_API_KEY env var (real external API call)"]
    async fn test_get_fin_summary_real_api() {
        for code in ["7203", "7974"] {
            let response =
                fetch_fin_summary("https://api.jquants.com/v2/fins/summary", code).await;
            log_summary_overview(&response);
            log_dividend_summaries(&response);
            assert!(!response.data.is_empty(), "データが取得できること: {code}");
        }
    }
}
