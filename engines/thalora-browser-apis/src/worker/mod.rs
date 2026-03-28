//! Web Workers APIs

// Native implementations using tokio/reqwest
#[cfg(feature = "native")]
pub mod import_scripts;
#[cfg(feature = "native")]
pub mod service_worker;
#[cfg(feature = "native")]
pub mod service_worker_container;
#[cfg(feature = "native")]
pub mod worker;
#[cfg(feature = "native")]
pub mod worker_error;
#[cfg(feature = "native")]
pub mod worker_events;
#[cfg(feature = "native")]
pub mod worker_global_scope;
#[cfg(feature = "native")]
pub mod worker_navigator;
#[cfg(feature = "native")]
pub mod worker_script_loader;
#[cfg(feature = "native")]
pub mod worker_thread;

// WASM stubs - browser's native Worker API is used directly
#[cfg(feature = "wasm")]
pub mod service_worker_container_wasm;
#[cfg(feature = "wasm")]
pub mod service_worker_wasm;
#[cfg(feature = "wasm")]
pub mod worker_wasm;

// Re-exports for uniform API
#[cfg(feature = "wasm")]
pub use service_worker_container_wasm as service_worker_container;
#[cfg(feature = "wasm")]
pub use service_worker_wasm as service_worker;
#[cfg(feature = "wasm")]
pub use worker_wasm as worker;

#[cfg(all(test, feature = "native"))]
mod tests;

#[cfg(all(test, feature = "native"))]
mod worker_thread_tests;

#[cfg(all(test, feature = "native"))]
mod worker_message_tests;

#[cfg(all(test, feature = "native"))]
mod worker_api_tests;
