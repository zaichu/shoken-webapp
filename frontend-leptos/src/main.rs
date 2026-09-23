mod api;
mod asset_balance_page;
mod dto;
mod home_page;
mod login_page;
mod not_found_page;
mod page;
mod receipts;
mod receipts_domain;
mod receipts_page;
mod search;
mod session;

use page::App;

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(|| leptos::prelude::view! { <App /> });
}
