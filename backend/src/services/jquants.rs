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
        let base_url = "https://api.jquants.com/v2/fins/summary";

        // クエリパラメータを構築（reqwest が自動でエンコード）
        let mut query_params: Vec<(&str, &str)> = vec![("code", &params.code)];
        let from_str;
        let to_str;

        if let Some(ref from) = params.from {
            from_str = from.clone();
            query_params.push(("from", &from_str));
        }

        if let Some(ref to) = params.to {
            to_str = to.clone();
            query_params.push(("to", &to_str));
        }

        tracing::info!("JQuants API V2 リクエスト: code={}", params.code);

        let response = client
            .get(base_url)
            .query(&query_params)
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

        // デシリアライズ前にレスポンス本文を取得（デバッグ用）
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
#[cfg(test)] #[rustfmt::skip] mod tests {
    use super::*;
    async fn fetch_fin_summary(code: &str) -> FinSummaryResponse { let api_key = std::env::var("JQUANTS_API_KEY").expect("JQUANTS_API_KEY 環境変数が設定されていません"); let params = FinSummaryQuery { code: code.to_string(), from: None, to: None }; JQuantsService::get_fin_summary(&Client::new(), params, &api_key).await.unwrap_or_else(|e| panic!("API呼び出しエラー: {:?}", e)) }
    fn log_summary_overview(response: &FinSummaryResponse) { println!("取得件数: {}", response.data.len()); if let Some(first) = response.data.first() { println!("銘柄コード: {}", first.local_code); println!("開示日: {}", first.disclosed_date); println!("書類種別: {}", first.type_of_document); println!("当期種別: {:?}", first.type_of_current_period); println!("当期開始日: {:?}", first.current_period_start_date); println!("当期終了日: {:?}", first.current_period_end_date); } }
    fn log_dividend_summaries(response: &FinSummaryResponse) { println!("取得件数: {}", response.data.len()); for summary in &response.data { println!("---"); println!("開示日: {}", summary.disclosed_date); println!("書類種別: {}", summary.type_of_document); println!("年間配当実績(DivAnn): {:?}", summary.result_dividend_per_share_annual); println!("年間配当予想(FDivAnn): {:?}", summary.forecast_dividend_per_share_annual); println!("年間配当来期予想(NxFDivAnn): {:?}", summary.next_year_forecast_dividend_per_share_annual); } }
    #[tokio::test] #[ignore = "requires JQUANTS_API_KEY env var (real external API call)"] async fn test_get_fin_summary_real_api() { let response = fetch_fin_summary("7203").await; log_summary_overview(&response); assert!(!response.data.is_empty(), "データが取得できること"); }
    #[tokio::test] #[ignore = "requires JQUANTS_API_KEY env var (real external API call)"] async fn test_get_nintendo_dividend() { let response = fetch_fin_summary("7974").await; log_dividend_summaries(&response); assert!(!response.data.is_empty(), "データが取得できること"); }
}
