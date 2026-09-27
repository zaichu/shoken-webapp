mod api;
mod app;
mod features;
mod session;
mod support;
#[cfg(test)]
mod testing;
mod ui;

use app::App;

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(|| leptos::prelude::view! { <App /> });
}
