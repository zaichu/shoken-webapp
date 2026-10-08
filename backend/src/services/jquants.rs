mod response;

#[cfg(not(target_arch = "wasm32"))]
mod client_native;
#[cfg(target_arch = "wasm32")]
mod client_worker;

pub use response::{FinSummaryData, FinSummaryResponse};

#[cfg(not(target_arch = "wasm32"))]
pub use client_native::JQuantsClient;
#[cfg(target_arch = "wasm32")]
pub use client_worker::JQuantsClient;
