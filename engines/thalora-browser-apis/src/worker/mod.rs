//! Web Workers APIs

// Native implementations using tokio/rquest
#[cfg(feature = "_native-core")]
pub mod import_scripts;
#[cfg(feature = "_native-core")]
pub mod service_worker;
#[cfg(feature = "_native-core")]
pub mod service_worker_container;
#[cfg(feature = "_native-core")]
pub mod worker;
#[cfg(feature = "_native-core")]
pub mod worker_error;
#[cfg(feature = "_native-core")]
pub mod worker_events;
#[cfg(feature = "_native-core")]
pub mod worker_global_scope;
#[cfg(feature = "_native-core")]
pub mod worker_navigator;
#[cfg(feature = "_native-core")]
pub mod worker_script_loader;
#[cfg(feature = "_native-core")]
pub mod worker_thread;

// WASM stubs - browser's native Worker API is used directly
#[cfg(feature = "wasm")]
pub mod worker_wasm;
#[cfg(feature = "wasm")]
pub mod service_worker_wasm;
#[cfg(feature = "wasm")]
pub mod service_worker_container_wasm;

// Re-exports for uniform API
#[cfg(feature = "wasm")]
pub use worker_wasm as worker;
#[cfg(feature = "wasm")]
pub use service_worker_wasm as service_worker;
#[cfg(feature = "wasm")]
pub use service_worker_container_wasm as service_worker_container;

#[cfg(all(test, feature = "native"))]
mod tests;

#[cfg(all(test, feature = "native"))]
mod worker_thread_tests;

#[cfg(all(test, feature = "native"))]
mod worker_message_tests;

#[cfg(all(test, feature = "native"))]
mod worker_api_tests;
