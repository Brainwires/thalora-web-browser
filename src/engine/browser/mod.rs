pub mod types;
pub mod core;
pub mod scraper;
pub mod navigation;
pub mod form_analyzer;

// Re-export main types
pub use types::*;
pub use core::HeadlessWebBrowser;
pub use form_analyzer::{FormAnalyzer, FormInfo};