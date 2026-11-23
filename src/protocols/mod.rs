// Protocol implementations
pub mod browser_tools;
pub mod credential_tools;
pub mod display_server;
pub mod mcp;
pub mod mcp_server;
pub mod rate_limiter;
pub mod security;
pub mod session_manager;
#[cfg(feature = "wasm-debug")]
pub mod wasm_debug_tools;

// Re-exports for clean API
pub use browser_tools::BrowserTools;
pub use display_server::{DisplayCommand, DisplayMessage, DisplayServer};
pub use mcp::McpResponse;
pub use mcp_server::McpServer;
pub use rate_limiter::RateLimiter;
pub use session_manager::{BrowserCommand, BrowserResponse, SessionInfo, SessionManager};
