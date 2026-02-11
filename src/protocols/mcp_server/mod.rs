pub mod core;
pub mod tools;
pub mod scraping;
pub mod transport;

// Re-export main types
pub use core::McpServer;
pub use scraping::{SearchResults, SearchResult};