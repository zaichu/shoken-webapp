mod api;
mod asset_balance_domain;
mod asset_balance_lookup;
mod asset_balance_page;
mod asset_balance_portfolio;
mod asset_balance_search;
mod confirm_modal;
mod csv_flow;
mod csv_rail;
mod dividend_info;
mod dividend_per_share;
mod dto;
mod home_page;
mod idle;
mod login_page;
mod not_found_page;
mod page;
mod receipts;
mod receipts_csv;
mod receipts_domain;
mod receipts_filter;
mod receipts_page;
mod receipts_pagination;
mod receipts_search;
mod receipts_search_group_key;
mod receipts_search_support;
mod search;
mod security_link;
mod session;
mod ui;

use page::App;

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(|| leptos::prelude::view! { <App /> });
}
