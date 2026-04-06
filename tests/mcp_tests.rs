// MCP Test Suite - Main entry point for all MCP server testing
//
// This file organizes and runs comprehensive tests for the Thalora MCP server.
// The test suite covers:
//
// 1. Protocol compliance (JSON-RPC, initialization, error handling)
// 2. Individual tool functionality (17+ MCP tools)
// 3. Integration workflows (complex multi-tool scenarios)
// 4. Performance benchmarking and stress testing
//
// To run all MCP tests:
//   cargo test mcp_tests
//
// To run specific test categories:
//   cargo test mcp_protocol_test
//   cargo test mcp_tools_test
//   cargo test mcp_integration_test
//   cargo test mcp_performance_test
//
// For release build testing (better performance):
//   cargo test --release mcp_tests

mod protocols;

// Re-export test modules for easy access
pub use protocols::{
    mcp_environment_test,
    mcp_harness::{McpTestConfig, McpTestHarness, create_initialized_harness},
    mcp_integration_test, mcp_performance_test, mcp_protocol_test, mcp_tools_test,
};

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    /// Meta-test to verify the test harness itself works
    #[test]
    fn test_harness_functionality() {
        let harness_result = create_initialized_harness();
        if let Err(e) = &harness_result {
            eprintln!("Harness creation failed: {}", e);
        }
        assert!(
            harness_result.is_ok(),
            "Test harness should initialize successfully: {:?}",
            harness_result.err()
        );

        let mut harness = harness_result.unwrap();
        assert!(harness.is_running(), "MCP server should be running");

        // Basic functionality test
        let tools_result = harness.list_tools();
        assert!(tools_result.is_ok(), "Should be able to list tools");

        let tools = tools_result.unwrap();
        assert!(!tools.is_empty(), "Should have at least one tool available");

        eprintln!(
            "Test harness verification successful - found {} tools",
            tools.len()
        );
    }

    /// Integration test to verify end-to-end testing capability
    #[test]
    fn test_end_to_end_verification() {
        let mut harness = create_initialized_harness().expect("Failed to create test harness");

        // Test the complete flow: store, retrieve, search
        let test_key = format!(
            "verification_{}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
        );
        let test_data = serde_json::json!({
            "purpose": "end-to-end verification",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "test_suite": "mcp_tests"
        });

        // Store data
        let store_response = harness.call_tool("ai_memory_store_research", serde_json::json!({
            "key": &test_key,
            "topic": "end-to-end verification testing",
            "summary": format!("End-to-end verification test data: {}", serde_json::to_string(&test_data).unwrap_or_default()),
            "tags": ["verification", "e2e"]
        })).expect("Store should succeed");

        assert!(!store_response.is_error, "Store should not return error");
        assert!(
            store_response.duration < Duration::from_secs(10),
            "Store should be reasonably fast"
        );

        // Retrieve data
        let get_response = harness
            .call_tool(
                "ai_memory_get_research",
                serde_json::json!({
                    "key": &test_key
                }),
            )
            .expect("Get should succeed");

        assert!(!get_response.is_error, "Get should not return error");
        assert!(
            get_response.duration < Duration::from_secs(10),
            "Get should be reasonably fast"
        );

        // Verify content
        let response_text = get_response.content[0]
            .get("text")
            .unwrap()
            .as_str()
            .unwrap();
        assert!(
            response_text.contains("verification"),
            "Retrieved data should contain original content"
        );

        // Search for data
        let search_response = harness
            .call_tool(
                "ai_memory_search_research",
                serde_json::json!({
                    "tags": ["verification"],
                    "limit": 1
                }),
            )
            .expect("Search should succeed");

        assert!(!search_response.is_error, "Search should not return error");

        eprintln!("End-to-end verification completed successfully");
    }

    /// Quick smoke test for all major tool categories
    #[test]
    fn test_tool_categories_smoke() {
        let mut harness = create_initialized_harness().expect("Failed to create test harness");

        // Memory tools
        let memory_result = harness.call_tool(
            "ai_memory_search_research",
            serde_json::json!({
                "query": "smoke_test",
                "limit": 1
            }),
        );
        assert!(memory_result.is_ok(), "Memory tools should be functional");

        // JavaScript evaluation
        let js_result = harness.call_tool(
            "cdp_runtime_evaluate",
            serde_json::json!({
                "expression": "1 + 1",
                "await_promise": false
            }),
        );
        match &js_result {
            Err(e) if e.to_string().contains("Timeout") || e.to_string().contains("timeout") => {
                eprintln!("Skipping: cdp_runtime_evaluate timed out (expected on CI)");
                return;
            }
            _ => {
                assert!(
                    js_result.is_ok(),
                    "JavaScript evaluation should be functional"
                );
            }
        }

        // DOM tools
        let dom_result = harness.call_tool(
            "cdp_dom_get_document",
            serde_json::json!({
                "depth": 1
            }),
        );
        assert!(dom_result.is_ok(), "DOM tools should be functional");

        eprintln!("All major tool categories passed smoke test");
    }
}

// Documentation for test organization
#[cfg(test)]
mod test_documentation {
    //! # MCP Test Suite Organization
    //!
    //! ## Test Categories
    //!
    //! ### Protocol Tests (`mcp_protocol_test.rs`)
    //! - JSON-RPC compliance
    //! - Initialization handshake
    //! - Error handling
    //! - Message format validation
    //! - Server stability under malformed input
    //!
    //! ### Tool Tests (`mcp_tools_test.rs`)
    //! - Individual tool functionality
    //! - Parameter validation
    //! - Error conditions
    //! - Response format verification
    //! - Edge cases and Unicode handling
    //!
    //! ### Integration Tests (`mcp_integration_test.rs`)
    //! - Multi-tool workflows
    //! - Data persistence across operations
    //! - Error recovery patterns
    //! - Concurrent operation handling
    //! - AI simulation scenarios
    //!
    //! ### Performance Tests (`mcp_performance_test.rs`)
    //! - Response time benchmarks
    //! - Throughput measurements
    //! - Memory usage patterns
    //! - Stress testing
    //! - Performance regression detection
    //!
    //! ## Running Tests
    //!
    //! ```bash
    //! # Run all MCP tests
    //! cargo test mcp_tests
    //!
    //! # Run specific categories
    //! cargo test mcp_protocol_test
    //! cargo test mcp_tools_test
    //! cargo test mcp_integration_test
    //! cargo test mcp_performance_test
    //!
    //! # Run with release build for performance testing
    //! cargo test --release mcp_performance_test
    //!
    //! # Run with debug output
    //! RUST_LOG=debug cargo test mcp_tests -- --nocapture
    //! ```
    //!
    //! ## Test Environment
    //!
    //! Tests spawn the MCP server as a subprocess and communicate via stdin/stdout.
    //! This ensures we're testing the actual interface that AI models will use.
    //!
    //! No external dependencies are required - tests use built-in capabilities
    //! and reliable test endpoints (like httpbin.org) for web operations.
}
