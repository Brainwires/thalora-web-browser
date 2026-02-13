pub mod types;

// Browser core is only available for native builds (requires reqwest, tokio, etc.)
#[cfg(any(feature = "native", feature = "web-search"))]
pub mod core;

pub mod scraper;
pub mod navigation;
pub mod form_analyzer;

// Re-export shared constants
pub use thalora_constants::USER_AGENT;

// Re-export main types
pub use types::*;

// Re-export browser for native builds only
#[cfg(any(feature = "native", feature = "web-search"))]
pub use core::HeadlessWebBrowser;

pub use form_analyzer::{FormAnalyzer, FormInfo};