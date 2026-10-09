mod response;

#[cfg(target_arch = "wasm32")]
mod client_worker;

pub use response::{FinSummaryData, FinSummaryResponse};

#[cfg(not(target_arch = "wasm32"))]
use crate::errors::ApiError;
#[cfg(not(target_arch = "wasm32"))]
use crate::models::market_data::financial_statement::FinancialStatementsQuery;

const FIN_SUMMARY_URL: &str = "https://api.jquants.com/v2/fins/summary";

#[derive(Clone)]
pub struct JQuantsClient {
    // フィールドを読むのは Worker 側の fetch 実装のみ
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    pub(crate) api_key: String,
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    pub(crate) base_url: String,
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
}
