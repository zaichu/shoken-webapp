pub mod config;
pub mod db;
pub mod errors;
pub mod extractors;
pub mod handlers;
pub mod middleware;
pub mod models;
#[cfg(not(target_arch = "wasm32"))]
pub mod openapi;
pub mod services;
pub mod state;
#[cfg(not(target_arch = "wasm32"))]
pub mod test_constants;
#[cfg(all(test, not(target_arch = "wasm32")))]
pub mod test_db;
#[cfg(target_arch = "wasm32")]
mod worker_entry;

#[cfg(not(target_arch = "wasm32"))]
pub use openapi::ApiDoc;
pub use state::AppState;
