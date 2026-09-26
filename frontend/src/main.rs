mod api;
mod asset_balance;
mod asset_balance_domain;
mod components;
mod csv_flow;
mod dividend_info;
mod dividend_per_share;
mod dto;
mod list_search;
mod pages;
mod pagination;
mod receipts;
mod receipts_domain;
mod session;
#[cfg(test)]
mod test_support;

use pages::App;

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(|| leptos::prelude::view! { <App /> });
}
