mod rate_limit;
mod security;
#[cfg(not(target_arch = "wasm32"))]
mod tracing;

#[cfg(target_arch = "wasm32")]
pub use rate_limit::binding_rate_limit;
#[cfg(not(target_arch = "wasm32"))]
pub use rate_limit::{build_keyed_rate_limiter, keyed_rate_limit};
pub use security::{add_security_headers, validate_origin};
#[cfg(not(target_arch = "wasm32"))]
pub use tracing::PathOnlyMakeSpan;
