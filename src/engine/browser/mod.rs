pub mod types;

// Browser core is only available for native builds (requires reqwest, tokio, etc.)
#[cfg(feature = "core")]
pub mod core;

pub mod form_analyzer;
pub mod navigation;
pub mod scraper;

// Re-export shared constants
pub use thalora_constants::USER_AGENT;

// Re-export main types
pub use types::*;

// Re-export browser for native builds only
#[cfg(feature = "core")]
pub use core::HeadlessWebBrowser;

pub use form_analyzer::{FormAnalyzer, FormInfo};
