//! Shared constants for Thalora browser
//!
//! Single source of truth for browser configuration

/// Chrome major version - update this to match current Chrome releases
pub const CHROME_VERSION: u32 = 131;

/// Single source of truth for browser user-agent
/// Chrome 131.0 on Windows 10 - used by HTTP client, Navigator API, Fetch API, etc.
/// MUST be Chrome since we expose window.chrome object
pub const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";

/// Chrome client hints header value
pub const SEC_CH_UA: &str = r#""Google Chrome";v="131", "Chromium";v="131", "Not_A Brand";v="24""#;
