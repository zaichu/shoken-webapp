mod response;

#[cfg(target_arch = "wasm32")]
mod client_worker;

pub use response::{
    EquitiesMasterData, EquitiesMasterResponse, FinSummaryData, FinSummaryResponse,
};

#[cfg(not(target_arch = "wasm32"))]
use crate::errors::ApiError;
#[cfg(not(target_arch = "wasm32"))]
use crate::models::market_data::financial_statement::FinancialStatementsQuery;

pub(crate) const FIN_SUMMARY_URL: &str = "https://api.jquants.com/v2/fins/summary";
pub(crate) const EQUITIES_MASTER_URL: &str = "https://api.jquants.com/v2/equities/master";

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

    /// dev/検証用にエンドポイントを差し替える。
    /// vars でのみ注入する想定で、未設定時は本番エンドポイントのままにする
    pub fn with_base_url(api_key: String, base_url: String) -> Self {
        Self::with_urls(api_key, base_url, EQUITIES_MASTER_URL.to_string())
    }

    /// dev/検証用に fins/summary と equities/master の両エンドポイントを差し替える
    pub fn with_urls(api_key: String, fin_summary_url: String, master_url: String) -> Self {
        Self {
            api_key,
            base_url: fin_summary_url,
            master_url,
        }
    }
}

// openapi.json はホストで生成されるためハンドラ経由で本メソッドもホストでコンパイル
// される必要がある。J-Quants への問い合わせ経路は Worker にしか存在しない
#[cfg(not(target_arch = "wasm32"))]
impl JQuantsClient {
    pub async fn get_fin_summary(
        &self,
        _params: FinancialStatementsQuery,
    ) -> Result<FinSummaryResponse, ApiError> {
        Err(ApiError::Internal(
            "J-Quants API の呼び出しは Workers 環境でのみ利用できます",
        ))
    }

    pub async fn get_equities_master(&self) -> Result<EquitiesMasterResponse, ApiError> {
        Err(ApiError::Internal(
            "J-Quants API の呼び出しは Workers 環境でのみ利用できます",
        ))
    }
}
