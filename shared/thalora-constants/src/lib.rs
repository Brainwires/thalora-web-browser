//! Shared constants for Thalora browser
//!
//! Single source of truth for browser configuration

/// Single source of truth for browser user-agent
/// Chrome 120.0 on Windows 10 - used by HTTP client, Navigator API, Fetch API, etc.
/// MUST be Chrome since we expose window.chrome object
pub const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
