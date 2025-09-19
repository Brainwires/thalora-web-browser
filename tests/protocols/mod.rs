// MCP Testing Module - Organization for all MCP-related tests

pub mod mcp_harness;
pub mod mcp_protocol_test;
pub mod mcp_tools_test;
pub mod mcp_integration_test;
pub mod mcp_performance_test;

// New CDP debugging and session management tests
pub mod cdp_debugging_tests;
pub mod session_management_tests;
pub mod cdp_session_integration_tests;

// Re-export common testing utilities
pub use mcp_harness::{McpTestHarness, McpTestConfig, McpToolResponse, create_initialized_harness, create_release_harness};