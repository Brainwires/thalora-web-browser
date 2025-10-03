//! Web Workers APIs

pub mod service_worker;
pub mod service_worker_container;
pub mod worker;
pub mod worker_error;
pub mod worker_events;
pub mod worker_global_scope;
pub mod worker_navigator;
pub mod worker_script_loader;

#[cfg(test)]
mod tests;
