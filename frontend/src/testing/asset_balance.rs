use crate::api::dto::{
    AssetBalance, AssetBalanceListResponse, AssetBalanceSummary, CsvPreviewResponse,
    CsvUploadResponse, SessionUser,
};
use crate::features::asset_balance::AssetBalanceCsvRow;
use crate::features::asset_balance::{
    AssetBalanceCsvStore, AssetBalanceLookupStore, BalanceSlot, DataOps,
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
        picture_url: None,
    }
}

pub(crate) fn balance_row(id: usize) -> AssetBalance {
    AssetBalance {
        id: format!("id-{id}").into(),
        security_code: format!("{id:04}").parse().unwrap(),
        security_name: "銘柄".to_string(),
        shares: rust_decimal_macros::dec!(100),
        executing_shares: rust_decimal_macros::dec!(0),
        average_purchase_price: rust_decimal_macros::dec!(2500),
        total_purchase_amount: rust_decimal_macros::dec!(250000),
        current_price: rust_decimal_macros::dec!(2600),
        daily_change: rust_decimal_macros::dec!(50),
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

pub(crate) fn csv_preview_response(
    rows: Vec<AssetBalanceCsvRow>,
) -> CsvPreviewResponse<AssetBalanceCsvRow> {
    CsvPreviewResponse {
        total_rows: 1,
        valid_rows: 1,
        errors: vec![],
        rows,
    }
}

pub(crate) fn csv_preview_row(code: &str) -> AssetBalanceCsvRow {
    AssetBalanceCsvRow {
        security_code: code.to_string(),
        security_name: "銘柄".to_string(),
        shares: rust_decimal_macros::dec!(100),
        average_purchase_price: rust_decimal_macros::dec!(2500),
        ..Default::default()
    }
}
