//! Workers 側の J-Quants クライアント。HTTP は `worker::Fetch` を使う。

use worker::{Fetch, Headers, Method, Request, RequestInit};

use super::{
    EquitiesMasterResponse, FinSummaryResponse, FinancialStatementsQuery, JQuantsClient,
    decode_error_summary, upstream_error_summary,
};
use crate::errors::{ApiError, UpstreamError};

impl JQuantsClient {
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

        tracing::info!("JQuants fins/summary リクエスト");
        let response_text = self.fetch_text(url.as_str()).await?;
        serde_json::from_str(&response_text).map_err(|e| {
            tracing::error!(
                "{}: {e}",
                decode_error_summary("決算サマリー", response_text.len())
            );
            UpstreamError::Decode(e).into()
        })
    }

    /// 上場銘柄一覧を全件取得する。ページネーション・クエリパラメータは無い
    pub async fn get_equities_master(&self) -> Result<EquitiesMasterResponse, ApiError> {
        tracing::info!("JQuants equities/master リクエスト");
        let response_text = self.fetch_text(&self.master_url).await?;
        serde_json::from_str(&response_text).map_err(|e| {
            tracing::error!(
                "{}: {e}",
                decode_error_summary("上場銘柄一覧", response_text.len())
            );
            UpstreamError::Decode(e).into()
        })
    }

    /// x-api-key 付き GET を送り、2xx 本文を返す。429 は RateLimited、
    /// その他の非 2xx は Http、通信失敗は Transport に写す
    async fn fetch_text(&self, url: &str) -> Result<String, ApiError> {
        let headers = Headers::new();
        headers.set("x-api-key", &self.api_key).map_err(|e| {
            tracing::error!("JQuants リクエストヘッダ設定エラー: {}", e);
            ApiError::Internal("header construction failed")
        })?;
        let mut init = RequestInit::new();
        init.with_method(Method::Get).with_headers(headers);
        let request = Request::new_with_init(url, &init).map_err(UpstreamError::Transport)?;

        // axum ハンドラは Future: Send が必要なため、JsFuture 系の待ち合わせは SendFuture で包む
        let fetch = Fetch::Request(request);
        let mut response = worker::send::SendFuture::new(fetch.send())
            .await
            .map_err(|e| {
                tracing::error!("JQuants API ネットワークエラー: {}", e);
                UpstreamError::Transport(e)
            })?;

        let status = response.status_code();
        tracing::info!("JQuants レスポンス status={status}");

        if !(200..300).contains(&status) {
            let error_text = worker::send::SendFuture::new(response.text())
                .await
                .unwrap_or_default();
            tracing::error!("{}", upstream_error_summary(status, error_text.len()));
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
                tracing::error!("JQuants レスポンス取得エラー: {e}");
                UpstreamError::Transport(e)
            })?;
        tracing::debug!("JQuants レスポンス len={}", response_text.len());
        Ok(response_text)
    }
}
