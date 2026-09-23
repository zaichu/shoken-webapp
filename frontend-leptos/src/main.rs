mod api;
mod asset_balance_page;
mod auth;
mod dto_gen;
mod dto_manual;
mod home_page;
mod login_page;
mod not_found_page;
mod page;
mod receipts;
mod receipts_page;
mod search;

use page::App;

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(|| leptos::prelude::view! { <App /> });
}
