pub mod core;
pub mod scraping;
pub mod tools;

#[cfg(feature = "http-transport")]
pub mod service;

#[cfg(feature = "http-transport")]
pub mod http_transport;

// Re-export main types
pub use core::McpServer;
pub use scraping::{SearchResult, SearchResults};
