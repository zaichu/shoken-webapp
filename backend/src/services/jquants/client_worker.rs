//! Workers 側の J-Quants クライアント。HTTP は `worker::Fetch` を使う。

use worker::{Fetch, Headers, Method, Request, RequestInit};

use super::response::FinSummaryResponse;
use crate::errors::{ApiError, UpstreamError};
use crate::models::market_data::financial_statement::FinancialStatementsQuery;

const FIN_SUMMARY_URL: &str = "https://api.jquants.com/v2/fins/summary";

#[derive(Clone)]
pub struct JQuantsClient {
    api_key: String,
    base_url: String,
}

impl JQuantsClient {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: FIN_SUMMARY_URL.to_string(),
        }
    }

    /// dev/検証用にエンドポイントを差し替える。
    /// vars でのみ注入する想定で、未設定時は本番エンドポイントのままにする
    pub fn with_base_url(api_key: String, base_url: String) -> Self {
        Self { api_key, base_url }
    }

    pub async fn get_fin_summary(
        &self,
        params: FinancialStatementsQuery,
    ) -> Result<FinSummaryResponse, ApiError> {
        let mut url =
            url::Url::parse(&self.base_url).map_err(crate::errors::ConfigError::UrlParse)?;
        url.query_pairs_mut().append_pair("code", &params.code);
        if let Some(from) = params.from.as_deref() {
            url.query_pairs_mut().append_pair("from", from);
        }
        if let Some(to) = params.to.as_deref() {
            url.query_pairs_mut().append_pair("to", to);
        }

        tracing::info!("JQuants API V2 リクエスト: code={}", params.code);

        let headers = Headers::new();
        headers.set("x-api-key", &self.api_key).map_err(|e| {
            tracing::error!("JQuants リクエストヘッダ設定エラー: {}", e);
            ApiError::Internal("header construction failed")
        })?;
        let mut init = RequestInit::new();
        init.with_method(Method::Get).with_headers(headers);
        let request =
            Request::new_with_init(url.as_str(), &init).map_err(UpstreamError::Transport)?;

        // axum ハンドラは Future: Send が必要なため、JsFuture 系の待ち合わせは SendFuture で包む
        let fetch = Fetch::Request(request);
        let mut response = worker::send::SendFuture::new(fetch.send())
            .await
            .map_err(|e| {
                tracing::error!("決算サマリー取得ネットワークエラー: {}", e);
                UpstreamError::Transport(e)
            })?;

        let status = response.status_code();
        tracing::info!("JQuants API レスポンスステータス: {}", status);

        if !(200..300).contains(&status) {
            let error_text = worker::send::SendFuture::new(response.text())
                .await
                .unwrap_or_default();
            tracing::error!(
                "JQuants API エラー - ステータス: {}, 本文: {}",
                status,
                error_text
            );
            if status == 429 {
                return Err(UpstreamError::RateLimited.into());
            }
            return Err(UpstreamError::Http {
                status,
                body: error_text,
            }
            .into());
        }

        let response_text = worker::send::SendFuture::new(response.text())
            .await
            .map_err(|e| {
                tracing::error!("レスポンス本文取得エラー: {}", e);
                UpstreamError::Transport(e)
            })?;

        tracing::debug!("JQuants API レスポンス本文: {}", response_text);

        let fin_summary_response: FinSummaryResponse = serde_json::from_str(&response_text)
            .map_err(|e| {
                tracing::error!(
                    "決算サマリーレスポンス解析エラー: {} - 本文: {}",
                    e,
                    response_text
                );
                UpstreamError::Decode(e)
            })?;

        Ok(fin_summary_response)
    }
}
