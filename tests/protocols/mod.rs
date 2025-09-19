// MCP Testing Module - Organization for all MCP-related tests

pub mod mcp_harness;
pub mod mcp_protocol_test;
pub mod mcp_tools_test;
pub mod mcp_integration_test;
pub mod mcp_performance_test;

// Re-export common testing utilities
pub use mcp_harness::{McpTestHarness, McpTestConfig, McpToolResponse, create_initialized_harness, create_release_harness};