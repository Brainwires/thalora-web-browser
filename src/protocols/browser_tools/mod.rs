pub mod core;
pub mod session;
pub mod handlers;

// Re-export main types
pub use core::BrowserTools;
pub use session::BrowserSession;