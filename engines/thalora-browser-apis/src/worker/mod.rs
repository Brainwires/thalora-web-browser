//! Web Workers APIs

pub mod import_scripts;
pub mod service_worker;
pub mod service_worker_container;
pub mod worker;
pub mod worker_error;
pub mod worker_events;
pub mod worker_global_scope;
pub mod worker_navigator;
pub mod worker_script_loader;
pub mod worker_thread;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod worker_thread_tests;

#[cfg(test)]
mod worker_message_tests;

#[cfg(test)]
mod worker_api_tests;
