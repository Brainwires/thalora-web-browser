// Core modules
pub mod core;
pub mod readability;
pub mod web_search;

// Type definitions
pub mod types;

// Utility functions
pub mod utils;

// Sub-modules
pub mod extraction;
pub mod search;

// Re-export commonly used types
pub use types::{SearchResult, SearchResults};
