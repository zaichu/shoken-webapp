use crate::api::dto::{AssetBalance, AssetBalanceListResponse, AssetBalanceSummary, SessionUser};
use crate::features::asset_balance::{
    AssetBalanceCsvStore, AssetBalanceLookupStore, BalanceSlot, DataOps, FundSlot,
};
use crate::features::dividend_per_share::DividendMaps;
use crate::session::SessionStore;
use crate::support::pagination::LIST_PER_PAGE;
use leptos::prelude::*;
use rust_decimal::Decimal;

pub(crate) fn user(id: &str) -> SessionUser {
    SessionUser {
        id: id.to_string(),
        email: format!("{id}@example.com"),
        name: None,
    }
}

pub(crate) fn balance_row(id: usize) -> AssetBalance {
    AssetBalance {
        id: format!("id-{id}").into(),
        security_code: format!("{id:04}").parse().unwrap(),
        security_name: "銘柄".to_string(),
        account: None,
        shares: rust_decimal_macros::dec!(100),
        average_purchase_price: rust_decimal_macros::dec!(2500),
        total_purchase_amount: rust_decimal_macros::dec!(250000),
        current_price: rust_decimal_macros::dec!(2600),
        daily_change: rust_decimal_macros::dec!(50),
        price_as_of: Some("2024-01-16".to_string()),
        prev_close: Some(rust_decimal_macros::dec!(2550)),
        created_at: String::new(),
        updated_at: String::new(),
    }
}

pub(crate) fn balance_page(
    range: std::ops::Range<usize>,
    total: i64,
    summary_total: Option<Decimal>,
) -> AssetBalanceListResponse {
    AssetBalanceListResponse {
        data: range.map(balance_row).collect(),
        total,
        page: 1,
        per_page: LIST_PER_PAGE as i64,
        summary: summary_total.map(|total_purchase_amount| AssetBalanceSummary {
            total_market_value: rust_decimal_macros::dec!(0),
            total_purchase_amount,
            total_daily_change: rust_decimal_macros::dec!(0),
            priced_market_value: rust_decimal_macros::dec!(0),
            priced_purchase_amount: total_purchase_amount,
            has_missing_daily_change: false,
        }),
        facets: None,
    }
}

pub(crate) fn csv_store(
    session: &SessionStore,
    balances: RwSignal<BalanceSlot>,
    dividends: RwSignal<DividendMaps>,
) -> AssetBalanceCsvStore {
    AssetBalanceCsvStore::new(
        *session,
        balances,
        RwSignal::<FundSlot>::new(None),
        dividends,
        RwSignal::new(AssetBalanceLookupStore::new()),
        RwSignal::new(DataOps::default()),
    )
}
