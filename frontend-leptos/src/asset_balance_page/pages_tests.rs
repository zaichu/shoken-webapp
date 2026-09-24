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
fn short_last_page_is_not_truncated() {
    let mut pages = AssetBalancePages::with_limits(3, 2);
    assert!(!pages.push(page(0..2, 2)));

    assert!(!pages.finish().truncated);
}

#[test]
fn truncated_warning_uses_the_same_wording_as_react_db_warning() {
    assert_eq!(
        truncated_list_warning(),
        "一覧は最大100,000件まで表示しています。未表示の銘柄がある可能性があります。"
    );
}
