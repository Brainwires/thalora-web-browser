pub mod core;
pub mod polyfills;
pub mod dom_events;
pub mod custom_events;
pub mod types;

// Re-export main types
pub use core::EventManager;
pub use types::*;