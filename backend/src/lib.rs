#[cfg(not(target_arch = "wasm32"))]
pub mod config;
#[cfg(not(target_arch = "wasm32"))]
pub mod db;
#[cfg(not(target_arch = "wasm32"))]
pub mod errors;
#[cfg(not(target_arch = "wasm32"))]
pub mod extractors;
#[cfg(not(target_arch = "wasm32"))]
pub mod handlers;
#[cfg(not(target_arch = "wasm32"))]
pub mod logging;
#[cfg(not(target_arch = "wasm32"))]
pub mod middleware;
#[cfg(not(target_arch = "wasm32"))]
pub mod models;
#[cfg(not(target_arch = "wasm32"))]
pub mod openapi;
#[cfg(not(target_arch = "wasm32"))]
pub mod routes;
#[cfg(not(target_arch = "wasm32"))]
pub mod services;
#[cfg(not(target_arch = "wasm32"))]
pub mod startup;
#[cfg(not(target_arch = "wasm32"))]
pub mod state;
#[cfg(all(test, not(target_arch = "wasm32")))]
pub mod test_db;
#[cfg(all(test, not(target_arch = "wasm32")))]
pub mod test_env;
#[cfg(target_arch = "wasm32")]
mod worker_entry;

#[cfg(not(target_arch = "wasm32"))]
pub use config::Config;
#[cfg(not(target_arch = "wasm32"))]
pub use errors::ApiError;
#[cfg(not(target_arch = "wasm32"))]
pub use extractors::validated_json::ValidatedJson;
#[cfg(not(target_arch = "wasm32"))]
pub use models::stock::Stock;
#[cfg(not(target_arch = "wasm32"))]
pub use openapi::ApiDoc;
#[cfg(not(target_arch = "wasm32"))]
pub use state::AppState;
