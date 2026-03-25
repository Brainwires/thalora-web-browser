pub mod core;
pub mod scraping;
pub mod tools;

// Re-export main types
pub use core::McpServer;
pub use scraping::{SearchResult, SearchResults};
