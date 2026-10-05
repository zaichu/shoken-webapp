//! `/api/v1/dividend-per-share-estimates` のバッチ取得。
//! 資産管理ページと取引明細の配当タブで共有するため切り出した。

use serde::Deserialize;
use shared::dividend_per_share::DividendPerShareBatchRequest;
use shared::value::SecurityCode;
use std::collections::HashMap;

use crate::api::{ApiClient, ApiError};

pub(crate) const DIVIDEND_RETRY_DELAY_MS: u32 = 15_000;
pub(crate) const DIVIDEND_NETWORK_MAX_RETRIES: u32 = 3;
const DIVIDEND_BATCH_PATH: &str = "/api/v1/dividend-per-share-estimates";
const DIVIDEND_PENDING_MAX_RETRIES: u32 = 100;
const DIVIDEND_BASE_RETRIES: u32 = 3;
const DIVIDEND_MILLIS_PER_CODE: u64 = 12_000;

#[derive(Clone, Debug, Default)]
pub(crate) struct DividendMaps {
    pub per_share: HashMap<String, f64>,
    pub status: HashMap<String, String>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct DividendEstimateItem {
    #[serde(default)]
    pub security_code: String,
    #[serde(default)]
    pub dividend_per_share: Option<f64>,
    #[serde(default)]
    pub status: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct DividendBatchResponse {
    #[serde(default)]
    pub items: Vec<DividendEstimateItem>,
}

pub(crate) fn unique_sorted_codes(codes: &[String]) -> Vec<String> {
    let mut unique: Vec<String> = codes.to_vec();
    unique.sort();
    unique.dedup();
    unique
}

pub(crate) fn dividend_pending_max_retries(unique_count: usize) -> u32 {
    let windows = (unique_count as u64)
        .saturating_mul(DIVIDEND_MILLIS_PER_CODE)
        .div_ceil(u64::from(DIVIDEND_RETRY_DELAY_MS));
    let dynamic = windows.saturating_add(u64::from(DIVIDEND_BASE_RETRIES));
    DIVIDEND_BASE_RETRIES.max(dynamic.min(u64::from(DIVIDEND_PENDING_MAX_RETRIES)) as u32)
}

pub(crate) fn dividend_maps_from_batch(batch: &DividendBatchResponse) -> (DividendMaps, bool) {
    let mut maps = DividendMaps::default();
    let mut has_pending = false;
    for item in &batch.items {
        if let Some(status) = &item.status {
            maps.status
                .insert(item.security_code.clone(), status.clone());
            if status == "pending" {
                has_pending = true;
            }
        }
        if item.status.as_deref() == Some("ok") {
            if let Some(per_share) = item.dividend_per_share {
                if per_share > 0.0 {
                    maps.per_share.insert(item.security_code.clone(), per_share);
                }
            }
        }
    }
    (maps, has_pending)
}

/// 本番の HTTP 実行。実ネットワーク境界のため mutation 評価は除外する
pub(crate) async fn post_dividend_batch(
    request: DividendPerShareBatchRequest,
) -> Result<DividendBatchResponse, ApiError> {
    ApiClient::default_client()
        .post_json::<DividendPerShareBatchRequest, DividendBatchResponse>(
            DIVIDEND_BATCH_PATH,
            &request,
        )
        .await
}

pub(crate) async fn fetch_dividend_batch<F, Fut>(
    codes: &[String],
    post_json: F,
) -> Result<DividendBatchResponse, ApiError>
where
    F: FnOnce(DividendPerShareBatchRequest) -> Fut,
    Fut: std::future::Future<Output = Result<DividendBatchResponse, ApiError>>,
{
    post_json(DividendPerShareBatchRequest {
        security_codes: codes.iter().cloned().map(SecurityCode::from_raw).collect(),
    })
    .await
}

#[cfg(test)]
mod tests;
