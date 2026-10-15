use crate::api::ApiError;
use crate::api::dto::{AssetBalance, AssetBalanceListResponse, AssetBalanceSummary};
use crate::features::asset_balance::store::csv::resolve_asset_balance;
use crate::features::asset_balance::store::{
    BalanceSlot, LoadedAssetBalances, truncated_list_warning,
};
use crate::session::Generation;
use crate::support::pagination::{LIST_PER_PAGE, collect_list_pages};
use crate::testing::block_on;
use std::future::{Ready, ready};

fn balance(id: usize) -> AssetBalance {
    AssetBalance {
        id: format!("id-{id}").into(),
        security_code: format!("{id:04}").parse().unwrap(),
        security_name: "銘柄".to_string(),
        shares: rust_decimal_macros::dec!(100),
        average_purchase_price: rust_decimal_macros::dec!(2500),
        total_purchase_amount: rust_decimal_macros::dec!(250000),
        current_price: rust_decimal_macros::dec!(2600),
        daily_change: rust_decimal_macros::dec!(50),
        price_as_of: Some("2024-01-16".to_string()),
        created_at: String::new(),
        updated_at: String::new(),
    }
}

fn page(range: std::ops::Range<usize>, total: i64) -> AssetBalanceListResponse {
    AssetBalanceListResponse {
        data: range.map(balance).collect(),
        total,
        page: 1,
        per_page: LIST_PER_PAGE as i64,
        summary: None,
        facets: None,
    }
}

#[test]
fn page_cap_marks_loaded_balances_truncated() {
    let fetch = |page_no: usize| -> Ready<Result<AssetBalanceListResponse, ApiError>> {
        ready(Ok(match page_no {
            1 => page(0..3, 100),
            _ => page(3..6, 100),
        }))
    };
    let loaded = block_on(collect_list_pages(3, 2, fetch)).expect("fetch");
    assert!(loaded.truncated);
    assert_eq!(loaded.rows.len(), 6);
}

#[test]
fn truncated_warning_uses_the_same_wording_as_db_warning() {
    assert_eq!(
        truncated_list_warning(),
        "一覧は最大100,000件まで表示しています。未表示の銘柄がある可能性があります。"
    );
}

fn loaded(rows: Vec<AssetBalance>, truncated: bool) -> LoadedAssetBalances {
    LoadedAssetBalances {
        total: rows.len(),
        rows,
        summary: None,
        truncated,
    }
}

#[test]
fn resolve_loading_for_missing_or_stale_slot() {
    let empty: BalanceSlot = None;
    assert!(resolve_asset_balance(Generation::new(1), &empty).is_none());

    let stale: BalanceSlot = Some((Generation::new(1), Ok(loaded(vec![balance(1)], false))));
    assert!(resolve_asset_balance(Generation::new(2), &stale).is_none());
}

#[test]
fn resolve_ready_uses_db_rows_and_warns_truncation() {
    let slot: BalanceSlot = Some((
        Generation::new(7),
        Ok(loaded(vec![balance(1), balance(2)], true)),
    ));
    let Some(resolved) = resolve_asset_balance(Generation::new(7), &slot) else {
        panic!("expected ready");
    };
    assert_eq!(resolved.rows.len(), 2);
    assert_eq!(resolved.warning, Some(truncated_list_warning()));
}

#[test]
fn resolve_without_truncation_has_no_warning() {
    let slot: BalanceSlot = Some((Generation::new(7), Ok(loaded(vec![balance(1)], false))));
    let Some(resolved) = resolve_asset_balance(Generation::new(7), &slot) else {
        panic!("expected ready");
    };
    assert!(resolved.warning.is_none());
}

#[test]
fn resolve_truncated_uses_summary_as_fallback_totals() {
    let summary = AssetBalanceSummary {
        total_market_value: rust_decimal_macros::dec!(700000),
        total_purchase_amount: rust_decimal_macros::dec!(500000),
        total_daily_change: rust_decimal_macros::dec!(0),
        priced_market_value: rust_decimal_macros::dec!(700000),
        priced_purchase_amount: rust_decimal_macros::dec!(500000),
    };
    let mut loaded_balances = loaded(vec![balance(1)], true);
    loaded_balances.summary = Some(summary.clone());
    let slot: BalanceSlot = Some((Generation::new(7), Ok(loaded_balances)));
    let Some(resolved) = resolve_asset_balance(Generation::new(7), &slot) else {
        panic!("expected ready");
    };
    assert_eq!(resolved.fallback_totals, Some(summary));
}

#[test]
fn resolve_full_rows_skips_fallback_totals() {
    // 行が揃う場合は個別カードとの一致を優先して行から計算するため fallback は使わない
    let mut loaded_balances = loaded(vec![balance(1)], false);
    loaded_balances.summary = Some(AssetBalanceSummary {
        total_market_value: rust_decimal_macros::dec!(700000),
        total_purchase_amount: rust_decimal_macros::dec!(500000),
        total_daily_change: rust_decimal_macros::dec!(0),
        priced_market_value: rust_decimal_macros::dec!(700000),
        priced_purchase_amount: rust_decimal_macros::dec!(500000),
    });
    let slot: BalanceSlot = Some((Generation::new(7), Ok(loaded_balances)));
    let Some(resolved) = resolve_asset_balance(Generation::new(7), &slot) else {
        panic!("expected ready");
    };
    assert!(resolved.fallback_totals.is_none());
}

#[test]
fn resolve_list_error_returns_no_rows() {
    let slot: BalanceSlot = Some((Generation::new(5), Err("取得失敗".to_string())));
    let Some(resolved) = resolve_asset_balance(Generation::new(5), &slot) else {
        panic!("expected ready");
    };
    assert!(resolved.rows.is_empty());
    assert!(resolved.warning.is_none());
}
