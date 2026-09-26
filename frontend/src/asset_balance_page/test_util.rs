use super::csv::AssetBalanceCsvStore;
use super::data::{BalanceSlot, DataOps, ASSET_BALANCE_LIST_PER_PAGE};
use crate::asset_balance_lookup::AssetBalanceLookupStore;
use crate::dividend_per_share::DividendMaps;
use crate::dto::{
    AssetBalance, AssetBalanceListResponse, AssetBalanceSummary, CsvPreviewResponse,
    CsvUploadResponse, SessionUser,
};
use crate::session::SessionStore;
use leptos::prelude::*;
use rust_decimal::Decimal;

pub(crate) fn user(id: &str) -> SessionUser {
    SessionUser {
        id: id.to_string(),
        email: format!("{id}@example.com"),
        name: None,
        picture_url: None,
    }
}

pub(crate) fn balance_row(id: usize) -> AssetBalance {
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

pub(crate) fn balance_page(
    range: std::ops::Range<usize>,
    total: i64,
    summary_total: Option<Decimal>,
) -> AssetBalanceListResponse {
    AssetBalanceListResponse {
        data: range.map(balance_row).collect(),
        total,
        page: 1,
        per_page: ASSET_BALANCE_LIST_PER_PAGE as i64,
        summary: summary_total.map(|total_purchase_amount| AssetBalanceSummary {
            total_market_value: rust_decimal_macros::dec!(0),
            total_purchase_amount,
            total_daily_change: rust_decimal_macros::dec!(0),
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
        dividends,
        RwSignal::new(AssetBalanceLookupStore::new()),
        RwSignal::new(DataOps::default()),
    )
}

pub(crate) fn csv_upload_response(inserted: usize) -> CsvUploadResponse {
    CsvUploadResponse {
        inserted,
        skipped: 0,
        errors: vec![],
    }
}

pub(crate) fn csv_preview_response(rows: Vec<serde_json::Value>) -> CsvPreviewResponse {
    CsvPreviewResponse {
        total_rows: 1,
        valid_rows: 1,
        errors: vec![],
        rows,
    }
}

pub(crate) fn csv_preview_row(code: &str) -> serde_json::Value {
    serde_json::json!({
        "security_code": code,
        "security_name": "銘柄",
        "shares": 100,
        "average_purchase_price": 2500,
        "market_value": 260000
    })
}
