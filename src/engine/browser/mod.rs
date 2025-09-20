pub mod types;
pub mod core;
pub mod scraper;
pub mod navigation;

// Re-export main types
pub use types::*;
pub use core::HeadlessWebBrowser;