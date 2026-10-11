//! Real OS thread-based worker execution
//!
//! This module implements true multi-threaded worker execution where each worker
//! runs in its own OS thread with a dedicated JavaScript context and event loop.

mod command_handler;
mod script_loader;
mod thread;
mod types;

// Re-export public types
pub use thread::WorkerThread;
pub use types::{WorkerCommand, WorkerConfig, WorkerEvent, WorkerStatus, WorkerType};
