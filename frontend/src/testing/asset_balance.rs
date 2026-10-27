use crate::api::dto::{AssetBalance, SessionUser};

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
