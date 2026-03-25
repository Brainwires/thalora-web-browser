// MCP Testing Module - Organization for all MCP-related tests

pub mod mcp_environment_test;
pub mod mcp_harness;
pub mod mcp_integration_test;
pub mod mcp_performance_test;
pub mod mcp_protocol_test;
pub mod mcp_tools_test;

// New CDP debugging and session management tests
pub mod cdp_debugging_tests;
pub mod cdp_session_integration_tests;
pub mod session_management_tests;

// WASM debug tools tests (requires wasm-debug feature)
#[cfg(feature = "wasm-debug")]
pub mod wasm_debug_tools_test;

// Re-export common testing utilities
#[allow(unused_imports)]
pub use mcp_harness::{
    McpTestConfig, McpTestHarness, McpToolResponse, create_harness_with_disabled_categories,
    create_harness_with_env, create_harness_with_only_categories, create_initialized_harness,
    create_release_harness,
};
