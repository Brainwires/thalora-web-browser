//! Real OS thread-based worker execution
//!
//! This module implements true multi-threaded worker execution where each worker
//! runs in its own OS thread with a dedicated JavaScript context and event loop.

mod callback_registry;
mod command_handler;
mod event_loop;
mod script_loader;
mod thread;
mod timer_api;
mod types;

// Re-export public types
pub use event_loop::WorkerEventLoop;
pub use thread::WorkerThread;
pub use types::{WorkerCommand, WorkerConfig, WorkerEvent, WorkerStatus, WorkerType};
