//! `/api/v1/dividend-per-share-estimates` のバッチ取得。
//! 資産管理ページと取引明細の配当タブで共有するため切り出した。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::api::{ApiClient, ApiError};

pub(crate) const DIVIDEND_RETRY_DELAY_MS: u32 = 15_000;
pub(crate) const DIVIDEND_NETWORK_MAX_RETRIES: u32 = 3;
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

#[derive(Clone, Debug, Serialize)]
struct DividendBatchRequest {
    security_codes: Vec<String>,
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

pub(crate) async fn fetch_dividend_batch(
    codes: &[String],
) -> Result<DividendBatchResponse, ApiError> {
    ApiClient::default_client()
        .post_json::<DividendBatchRequest, DividendBatchResponse>(
            "/api/v1/dividend-per-share-estimates",
            &DividendBatchRequest {
                security_codes: codes.to_vec(),
            },
        )
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unique_sorted_codes_dedupes() {
        assert_eq!(
            unique_sorted_codes(&["6758".to_string(), "7203".to_string(), "6758".to_string()]),
            vec!["6758".to_string(), "7203".to_string()]
        );
        assert!(unique_sorted_codes(&[]).is_empty());
    }

    #[test]
    fn dividend_pending_max_retries_matches_hook() {
        assert_eq!(dividend_pending_max_retries(0), 3);
        assert_eq!(dividend_pending_max_retries(1), 4);
        assert_eq!(dividend_pending_max_retries(2), 5);
        assert_eq!(dividend_pending_max_retries(100), 83);
        assert_eq!(dividend_pending_max_retries(usize::MAX), 100);
    }

    fn estimate(code: &str, per_share: Option<f64>, status: &str) -> DividendEstimateItem {
        DividendEstimateItem {
            security_code: code.to_string(),
            dividend_per_share: per_share,
            status: Some(status.to_string()),
        }
    }

    #[test]
    fn dividend_batch_maps_match_hook() {
        let batch = DividendBatchResponse {
            items: vec![
                estimate("7203", Some(50.0), "ok"),
                estimate("6758", Some(0.0), "ok"),
                estimate("0001", None, "pending"),
                estimate("0002", None, "error"),
                estimate("0003", Some(10.0), "zero"),
                estimate("0004", None, "ok"),
            ],
        };
        let (maps, has_pending) = dividend_maps_from_batch(&batch);
        assert!(has_pending);
        assert_eq!(maps.per_share.get("7203"), Some(&50.0));
        assert!(!maps.per_share.contains_key("6758"));
        assert!(!maps.per_share.contains_key("0003"));
        assert!(!maps.per_share.contains_key("0004"));
        assert_eq!(maps.status.get("0001").map(String::as_str), Some("pending"));
        assert_eq!(maps.status.get("0002").map(String::as_str), Some("error"));
        assert_eq!(maps.status.get("0003").map(String::as_str), Some("zero"));

        let settled = DividendBatchResponse {
            items: vec![estimate("7203", Some(50.0), "ok")],
        };
        let (_, settled_pending) = dividend_maps_from_batch(&settled);
        assert!(!settled_pending);
        let (_, empty_pending) = dividend_maps_from_batch(&DividendBatchResponse { items: vec![] });
        assert!(!empty_pending);
    }

    proptest::proptest! {
        #[test]
        fn prop_unique_sorted_codes_matches_set(
            codes in proptest::collection::vec("[0-9A-Za-z]{0,8}", 0..32usize),
        ) {
            let expected: Vec<String> = codes
                .iter()
                .cloned()
                .collect::<std::collections::BTreeSet<_>>()
                .into_iter()
                .collect();
            proptest::prop_assert_eq!(unique_sorted_codes(&codes), expected);
        }

        #[test]
        fn prop_pending_max_retries_bounds_and_monotone(count in 0usize..10_000) {
            let retries = dividend_pending_max_retries(count);
            proptest::prop_assert!(
                (DIVIDEND_BASE_RETRIES..=DIVIDEND_PENDING_MAX_RETRIES).contains(&retries)
            );
            proptest::prop_assert!(
                retries <= dividend_pending_max_retries(count.saturating_add(1))
            );
        }

        #[test]
        fn prop_maps_from_batch_matches_naive(
            items in proptest::collection::vec(
                (
                    "[0-9]{4}",
                    proptest::option::of(-100.0f64..1000.0f64),
                    proptest::sample::select(vec!["ok", "pending", "error", "zero"]),
                ),
                0..16usize
            ),
        ) {
            let items: Vec<DividendEstimateItem> = items
                .into_iter()
                .map(|(code, per_share, status)| DividendEstimateItem {
                    security_code: code.to_string(),
                    dividend_per_share: per_share,
                    status: Some(status.to_string()),
                })
                .collect();
            let batch = DividendBatchResponse { items: items.clone() };
            let (maps, has_pending) = dividend_maps_from_batch(&batch);

            let mut expected_per_share = HashMap::new();
            let mut expected_status = HashMap::new();
            let mut expected_pending = false;
            for item in &items {
                if let Some(status) = &item.status {
                    expected_status.insert(item.security_code.clone(), status.clone());
                    if status == "pending" {
                        expected_pending = true;
                    }
                    if status == "ok" {
                        if let Some(per_share) = item.dividend_per_share {
                            if per_share > 0.0 {
                                expected_per_share
                                    .insert(item.security_code.clone(), per_share);
                            }
                        }
                    }
                }
            }
            proptest::prop_assert_eq!(maps.per_share, expected_per_share);
            proptest::prop_assert_eq!(maps.status, expected_status);
            proptest::prop_assert_eq!(has_pending, expected_pending);
        }
    }
}
