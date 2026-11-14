mod response;

#[cfg(not(target_arch = "wasm32"))]
use std::future::Future;

#[cfg(target_arch = "wasm32")]
mod client_worker;

pub use response::{
    EquitiesMasterData, EquitiesMasterResponse, FinSummaryData, FinSummaryResponse,
};

#[cfg(not(target_arch = "wasm32"))]
use crate::errors::ApiError;

pub(crate) const FIN_SUMMARY_URL: &str = "https://api.jquants.com/v2/fins/summary";
pub(crate) const EQUITIES_MASTER_URL: &str = "https://api.jquants.com/v2/equities/master";

/// 決算サマリー取得パラメータ
#[derive(Debug)]
pub struct FinancialStatementsQuery {
    pub code: String,
    pub from: Option<String>,
    pub to: Option<String>,
}

#[derive(Clone)]
pub struct JQuantsClient {
    // フィールドを読むのは Worker 側の fetch 実装のみ
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    pub(crate) api_key: String,
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    pub(crate) base_url: String,
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    pub(crate) master_url: String,
}

impl JQuantsClient {
    pub fn new(api_key: String) -> Self {
        Self::with_urls(
            api_key,
            FIN_SUMMARY_URL.to_string(),
            EQUITIES_MASTER_URL.to_string(),
        )
    }

    /// dev/検証用に fins/summary と equities/master の両エンドポイントを差し替える
    pub const fn with_urls(api_key: String, fin_summary_url: String, master_url: String) -> Self {
        Self {
            api_key,
            base_url: fin_summary_url,
            master_url,
        }
    }
}

/// 上流の応答本文を含めない要約ログ。処理名・状態・長さだけにする。
#[cfg(any(target_arch = "wasm32", test))]
pub(crate) fn upstream_error_summary(status: u16, body_len: usize) -> String {
    format!("JQuants API エラー status={status} len={body_len}")
}

/// 解析失敗の要約ログ。本文は載せず長さだけにする。
#[cfg(any(target_arch = "wasm32", test))]
pub(crate) fn decode_error_summary(op: &'static str, body_len: usize) -> String {
    format!("{op}解析エラー len={body_len}")
}

// openapi.json はホストで生成されるためハンドラ経由で本メソッドもホストでコンパイル
// される必要がある。J-Quants への問い合わせ経路は Worker にしか存在しない
#[cfg(not(target_arch = "wasm32"))]
impl JQuantsClient {
    pub fn get_fin_summary(
        &self,
        _params: FinancialStatementsQuery,
    ) -> impl Future<Output = Result<FinSummaryResponse, ApiError>> {
        std::future::ready(Err(ApiError::Internal(
            "J-Quants API の呼び出しは Workers 環境でのみ利用できます",
        )))
    }

    pub fn get_equities_master(
        &self,
    ) -> impl Future<Output = Result<EquitiesMasterResponse, ApiError>> {
        std::future::ready(Err(ApiError::Internal(
            "J-Quants API の呼び出しは Workers 環境でのみ利用できます",
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upstream_log_summaries_exclude_body() {
        let body = "secret-body-xyz-123";
        let error_log = upstream_error_summary(500, body.len());
        assert!(error_log.contains("500"));
        assert!(!error_log.contains(body));
        let decode_log = decode_error_summary("決算サマリー", body.len());
        assert!(decode_log.contains("決算サマリー"));
        assert!(!decode_log.contains(body));
    }
}
