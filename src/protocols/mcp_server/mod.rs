pub mod core;
pub mod tools;
pub mod scraping;

// Re-export main types
pub use core::McpServer;
pub use scraping::{SearchResults, SearchResult};