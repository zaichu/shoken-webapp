use super::*;
use crate::testing::block_on;

#[test]
fn fetch_dividend_batch_uses_shared_request_without_changing_wire_codes() {
    let codes = vec![
        "7203".to_string(),
        "7203".to_string(),
        "".to_string(),
        "7203-1".to_string(),
    ];
    let result = block_on(fetch_dividend_batch(
        &codes,
        |request: shared::dividend_per_share::DividendPerShareBatchRequest| {
            assert_eq!(
                serde_json::to_value(request).unwrap(),
                serde_json::json!({"security_codes":codes})
            );
            std::future::ready(Ok(DividendBatchResponse { items: vec![] }))
        },
    ));
    assert!(result.unwrap().items.is_empty());
}

#[test]
fn dividend_response_projection_keeps_partial_and_unknown_status_compatibility() {
    for value in [
        serde_json::json!({}),
        serde_json::json!({"items":[{"security_code":"7203","status":"ok","dividend_per_share":50}]}),
        serde_json::json!({"items":[{"security_code":"7203","status":"future","dividend_per_share":50,"is_stale":false}]}),
        serde_json::json!({"items":[{"security_code":"7203","dividend_per_share":50,"is_stale":false}]}),
    ] {
        assert!(serde_json::from_value::<DividendBatchResponse>(value.clone()).is_ok());
        assert!(
            serde_json::from_value::<shared::dividend_per_share::DividendPerShareBatchResponse>(
                value
            )
            .is_err()
        );
    }
}

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

#[test]
fn fetch_dividend_batch_forwards_codes_and_returns_response() {
    let captured = std::cell::RefCell::new(Vec::new());
    let codes = vec!["6758".to_string(), "7203".to_string(), "6758".to_string()];
    let result = block_on(fetch_dividend_batch(&codes, |request| {
        *captured.borrow_mut() = request
            .security_codes
            .into_iter()
            .map(String::from)
            .collect();
        std::future::ready(Ok(DividendBatchResponse {
            items: vec![estimate("7203", Some(50.0), "ok")],
        }))
    }));

    assert_eq!(*captured.borrow(), codes);
    let response = result.expect("モック応答をそのまま返すはず");
    assert_eq!(response.items.len(), 1);
    assert_eq!(response.items[0].security_code, "7203");
    assert_eq!(response.items[0].dividend_per_share, Some(50.0));
}

#[test]
fn fetch_dividend_batch_propagates_http_error() {
    let result = block_on(fetch_dividend_batch(&["7203".to_string()], |_request| {
        std::future::ready(Err(ApiError::Parse))
    }));
    assert_eq!(result.unwrap_err(), ApiError::Parse);
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
                if status == "ok"
                    && let Some(per_share) = item.dividend_per_share
                    && per_share > 0.0
                {
                    expected_per_share
                        .insert(item.security_code.clone(), per_share);
                }
            }
        }
        proptest::prop_assert_eq!(maps.per_share, expected_per_share);
        proptest::prop_assert_eq!(maps.status, expected_status);
        proptest::prop_assert_eq!(has_pending, expected_pending);
    }
}
