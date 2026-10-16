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

// Re-exports for uniform API

#[cfg(all(test, feature = "native"))]
mod tests;

#[cfg(all(test, feature = "native"))]
mod worker_thread_tests;

#[cfg(all(test, feature = "native"))]
mod worker_event_loop_tests;

#[cfg(all(test, feature = "native"))]
mod worker_message_tests;

#[cfg(all(test, feature = "native"))]
mod worker_api_tests;
