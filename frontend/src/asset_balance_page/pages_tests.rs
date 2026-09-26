use super::*;

fn balance(id: usize) -> AssetBalance {
    AssetBalance {
        id: format!("id-{id}"),
        security_code: format!("{id:04}"),
        security_name: "銘柄".to_string(),
        shares: rust_decimal_macros::dec!(100),
        executing_shares: rust_decimal_macros::dec!(0),
        average_purchase_price: rust_decimal_macros::dec!(2500),
        total_purchase_amount: rust_decimal_macros::dec!(250000),
        current_price: rust_decimal_macros::dec!(2600),
        daily_change: rust_decimal_macros::dec!(50),
        market_value: rust_decimal_macros::dec!(260000),
        profit_loss_rate: rust_decimal_macros::dec!(4),
        created_at: String::new(),
        updated_at: String::new(),
    }
}

fn page(range: std::ops::Range<usize>, total: i64) -> AssetBalanceListResponse {
    AssetBalanceListResponse {
        data: range.map(balance).collect(),
        total,
        page: 1,
        per_page: ASSET_BALANCE_LIST_PER_PAGE as i64,
        summary: None,
        facets: None,
    }
}

#[test]
fn page_cap_marks_loaded_balances_truncated() {
    let mut pages = AssetBalancePages::with_limits(3, 2);
    assert!(pages.push(page(0..3, 100)));
    assert!(!pages.push(page(3..6, 100)));

    let loaded = pages.finish();
    assert!(loaded.truncated);
    assert_eq!(loaded.rows.len(), 6);
}

#[test]
fn truncated_warning_uses_the_same_wording_as_react_db_warning() {
    assert_eq!(
        truncated_list_warning(),
        "一覧は最大100,000件まで表示しています。未表示の銘柄がある可能性があります。"
    );
}

fn loaded(rows: Vec<AssetBalance>, truncated: bool) -> LoadedAssetBalances {
    LoadedAssetBalances {
        total: rows.len(),
        rows,
        summary: Some(AssetBalanceSummary {
            total_purchase_amount: rust_decimal_macros::dec!(250000),
            total_market_value: rust_decimal_macros::dec!(260000),
            total_daily_change: rust_decimal_macros::dec!(0),
        }),
        facets: Some(SearchFacets {
            securities: Some(vec![crate::dto::FacetOption {
                value: "7203".to_string(),
                label: "トヨタ自動車".to_string(),
                count: Some(1),
            }]),
            ..Default::default()
        }),
        truncated,
    }
}

fn preview_state() -> CsvTabState<AssetBalanceCsvRow> {
    CsvTabState {
        file_name: Some("asset.csv".to_string()),
        preview: Some(crate::csv_flow::CsvPreview {
            total_rows: 1,
            valid_rows: 1,
            errors: vec![],
            rows: vec![AssetBalanceCsvRow {
                security_code: "9999".to_string(),
                ..Default::default()
            }],
        }),
        ..Default::default()
    }
}

#[test]
fn resolve_loading_for_missing_or_stale_slot() {
    let state = CsvTabState::default();
    let empty: BalanceSlot = None;
    assert!(resolve_asset_balance(1, &empty, &state).is_none());

    let stale: BalanceSlot = Some((1, Ok(loaded(vec![balance(1)], false))));
    assert!(resolve_asset_balance(2, &stale, &state).is_none());
}

#[test]
fn resolve_ready_uses_db_rows_and_carries_aggregates() {
    let state = CsvTabState::default();
    let slot: BalanceSlot = Some((7, Ok(loaded(vec![balance(1), balance(2)], true))));
    let Some(resolved) = resolve_asset_balance(7, &slot, &state) else {
        panic!("expected ready");
    };
    assert_eq!(resolved.rows.len(), 2);
    assert!(resolved.summary.is_some());
    assert!(resolved.facets.is_some());
    assert_eq!(resolved.warning, Some(truncated_list_warning()));
    assert!(!resolved.has_csv_file);
}

#[test]
fn resolve_without_truncation_has_no_warning() {
    let state = CsvTabState::default();
    let slot: BalanceSlot = Some((7, Ok(loaded(vec![balance(1)], false))));
    let Some(resolved) = resolve_asset_balance(7, &slot, &state) else {
        panic!("expected ready");
    };
    assert!(resolved.warning.is_none());
}

#[test]
fn resolve_csv_preview_replaces_rows_and_drops_warning() {
    let state = preview_state();
    let slot: BalanceSlot = Some((3, Ok(loaded(vec![balance(1)], true))));
    let Some(resolved) = resolve_asset_balance(3, &slot, &state) else {
        panic!("expected ready");
    };
    assert_eq!(resolved.rows.len(), 1);
    assert_eq!(resolved.rows[0].security_code, "9999");
    assert!(resolved.warning.is_none());
    assert!(resolved.has_csv_file);
    assert!(resolved.summary.is_some());
}

#[test]
fn resolve_list_error_falls_back_to_preview_rows() {
    let state = preview_state();
    let slot: BalanceSlot = Some((5, Err("取得失敗".to_string())));
    let Some(resolved) = resolve_asset_balance(5, &slot, &state) else {
        panic!("expected ready");
    };
    assert_eq!(resolved.rows.len(), 1);
    assert_eq!(resolved.rows[0].security_code, "9999");
    assert!(resolved.summary.is_none());
    assert!(resolved.facets.is_none());
    assert!(resolved.warning.is_none());
}
