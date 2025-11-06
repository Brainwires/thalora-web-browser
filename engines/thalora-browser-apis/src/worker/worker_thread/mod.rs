//! Real OS thread-based worker execution
//!
//! This module implements true multi-threaded worker execution where each worker
//! runs in its own OS thread with a dedicated JavaScript context and event loop.

mod types;
mod script_loader;
mod command_handler;
mod event_loop;
mod callback_registry;
mod timer_api;
mod thread;

// Re-export public types
pub use types::{WorkerCommand, WorkerEvent, WorkerStatus, WorkerType, WorkerConfig};
pub use thread::WorkerThread;
pub use event_loop::WorkerEventLoop;
