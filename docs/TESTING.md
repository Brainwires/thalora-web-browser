# Thalora MCP Testing Guide

This document describes the comprehensive testing system for the Thalora MCP (Model Context Protocol) server interface.

## Overview

The MCP testing system provides complete validation of the stdio-based interface that AI models use to interact with the Thalora headless browser. The tests verify protocol compliance, tool functionality, integration workflows, and performance characteristics.

## Test Architecture

### Test Organization

```
tests/
├── mcp_tests.rs                    # Main test entry point and documentation
├── protocols/
│   ├── mcp_harness.rs             # Test harness for subprocess communication
│   ├── mcp_protocol_test.rs       # JSON-RPC protocol compliance tests
│   ├── mcp_tools_test.rs          # Individual tool functionality tests
│   ├── mcp_integration_test.rs    # Complex workflow integration tests
│   ├── mcp_performance_test.rs    # Performance benchmarking and stress tests
│   └── mod.rs                     # Module organization
└── run_mcp_tests.sh               # Test runner script
```

### Test Categories

#### 1. Protocol Compliance Tests (`mcp_protocol_test.rs`)
- **Initialization**: JSON-RPC handshake and capability negotiation
- **Message Format**: Request/response structure validation
- **Error Handling**: Malformed input and recovery testing
- **Server Stability**: Continuous operation under various conditions

#### 2. Tool Functionality Tests (`mcp_tools_test.rs`)
- **AI Memory Tools**: Store, retrieve, and search research data
- **Web Scraping**: URL content extraction and Google search
- **Browser Automation**: Element interaction and form handling
- **CDP Tools**: JavaScript evaluation and DOM inspection
- **Parameter Validation**: Required fields and type checking
- **Edge Cases**: Unicode data, large payloads, error conditions

#### 3. Integration Tests (`mcp_integration_test.rs`)
- **Research Workflow**: Search → Scrape → Store → Retrieve
- **Browser Automation**: JavaScript → DOM → Interaction → Storage
- **Data Persistence**: Multi-tool data flow validation
- **Error Recovery**: Graceful handling of tool failures
- **AI Simulation**: Complete AI agent interaction patterns

#### 4. Performance Tests (`mcp_performance_test.rs`)
- **Response Time Benchmarks**: Individual tool performance metrics
- **Throughput Testing**: Rapid sequential operation handling
- **Memory Usage**: Large data handling and cleanup
- **Stress Testing**: 100+ rapid requests with stability monitoring
- **Mixed Workload**: Realistic AI usage pattern simulation

## Running Tests

### Quick Start

```bash
# Run all tests
./run_mcp_tests.sh

# Quick smoke tests only
./run_mcp_tests.sh --quick

# Performance tests only
./run_mcp_tests.sh --perf

# Help and options
./run_mcp_tests.sh --help
```

### Manual Test Execution

```bash
# Run all MCP tests
cargo test --test mcp_tests

# Run specific test categories
cargo test --test mcp_tests mcp_protocol_test
cargo test --test mcp_tests mcp_tools_test
cargo test --test mcp_tests mcp_integration_test
cargo test --test mcp_tests mcp_performance_test

# Run with debug output
RUST_LOG=debug cargo test --test mcp_tests -- --nocapture

# Performance tests with release build
cargo test --release --test mcp_tests performance
```

### Individual Test Execution

```bash
# Test specific functionality
cargo test test_ai_memory_store_and_retrieve -- --nocapture
cargo test test_research_workflow -- --nocapture
cargo test test_javascript_evaluation_performance -- --nocapture

# Test MCP protocol compliance
cargo test test_mcp_initialization -- --nocapture
cargo test test_tools_list_structure -- --nocapture

# Test error handling
cargo test test_error_recovery_workflow -- --nocapture
```

## Test Harness Architecture

### McpTestHarness

The `McpTestHarness` provides a complete testing environment:

```rust
// Create a test harness
let mut harness = McpTestHarness::new()?;

// Initialize MCP connection
harness.initialize()?;

// List available tools
let tools = harness.list_tools()?;

// Call a tool
let response = harness.call_tool("scrape_url", json!({
    "url": "https://example.com",
    "wait_for_js": false
}))?;
```

### Features

- **Subprocess Management**: Spawns and manages MCP server process
- **JSON-RPC Communication**: Handles stdin/stdout protocol communication
- **Timeout Handling**: Configurable timeouts for network operations
- **Error Recovery**: Graceful handling of server failures
- **Performance Metrics**: Built-in timing and success rate tracking

## Test Environment

### Requirements

- **No External Dependencies**: Tests use built-in capabilities only
- **Reliable Test Endpoints**: httpbin.org for web scraping tests
- **Deterministic Data**: Timestamp-based keys for unique test data
- **Cleanup**: Automatic process cleanup on test completion

### Configuration

```rust
// Default configuration
let config = McpTestConfig::default();

// Custom configuration
let config = McpTestConfig {
    timeout: Duration::from_secs(60),
    use_release_build: true,
    debug_output: true,
};

let harness = McpTestHarness::with_config(config)?;
```

## Expected Tool Coverage

The test suite validates all 17+ MCP tools:

### AI Memory Tools
- `ai_memory_store_research` - Store research data with tags
- `ai_memory_get_research` - Retrieve stored research by key
- `ai_memory_search_research` - Search research by query and tags

### Web Scraping Tools
- `scrape_url` - Extract content from web pages
- `google_search` - Search Google and return results

### Browser Automation Tools
- `browser_click_element` - Click on page elements
- Additional automation tools as implemented

### Chrome DevTools Protocol Tools
- `cdp_runtime_evaluate` - Execute JavaScript in browser context
- `cdp_dom_get_document` - Retrieve DOM document structure
- Additional CDP tools as implemented

## Performance Expectations

### Response Time Targets
- **Memory Operations**: < 2 seconds average
- **JavaScript Evaluation**: < 1 second for simple expressions
- **Web Scraping**: < 30 seconds including network time
- **Tool Listing**: < 1 second

### Throughput Targets
- **Sequential Operations**: 100+ requests in < 60 seconds
- **Success Rate**: > 90% under normal conditions
- **Error Recovery**: Server remains responsive after failures

### Memory Usage
- **Baseline**: ~10MB for server process
- **Large Data**: Graceful handling of 50KB+ payloads
- **Cleanup**: No memory leaks over extended test runs

## Debugging Test Failures

### Common Issues

1. **Build Failures**
   ```bash
   cargo build --quiet
   cargo check --tests
   ```

2. **Server Startup Issues**
   ```bash
   # Test manual server startup
   cargo run
   # Should show: "🧠 Thalora v0.1.0 - Pure Rust headless browser for AI models"
   ```

3. **Network-Dependent Test Failures**
   ```bash
   # Test with increased timeouts
   cargo test test_scrape_url -- --nocapture
   ```

4. **Permission Issues**
   ```bash
   # Ensure script is executable
   chmod +x run_mcp_tests.sh
   ```

### Debug Output

```bash
# Enable debug logging
RUST_LOG=debug cargo test test_name -- --nocapture

# Test with verbose server output
RUST_LOG=debug ./run_mcp_tests.sh --verbose
```

### Manual MCP Testing

```bash
# Build and test manually
cargo build --release

# Test tool listing
echo '{"jsonrpc": "2.0", "id": 1, "method": "tools/list", "params": {}}' | ./target/release/thalora

# Test a simple tool call
echo '{"jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": {"name": "ai_memory_search_research", "arguments": {"query": "test", "limit": 1}}}' | ./target/release/thalora
```

## Continuous Integration

### CI Test Strategy

1. **Quick Tests**: Run on every commit
   ```bash
   ./run_mcp_tests.sh --quick
   ```

2. **Full Suite**: Run on pull requests
   ```bash
   ./run_mcp_tests.sh
   ```

3. **Performance Tests**: Run nightly with release builds
   ```bash
   ./run_mcp_tests.sh --perf
   ```

### Test Reliability

- **Flaky Test Handling**: Network-dependent tests have multiple retry attempts
- **Timeout Management**: Conservative timeouts for CI environments
- **Resource Cleanup**: Automatic process termination prevents resource leaks

## Extending the Test Suite

### Adding New Tool Tests

1. **Add Tool Definition Test**: Verify tool appears in tools list
2. **Add Functionality Test**: Test normal operation with valid parameters
3. **Add Error Handling Test**: Test with invalid parameters
4. **Add Integration Test**: Include in workflow scenarios

Example:
```rust
#[test]
fn test_new_tool() {
    let mut harness = create_initialized_harness().expect("Failed to create harness");

    let response = harness.call_tool("new_tool_name", json!({
        "parameter": "value"
    })).expect("Tool should succeed");

    assert!(!response.is_error, "Tool should not return error");
    validate_tool_response(&response, "expected_content_type").expect("Valid response");
}
```

### Adding Performance Benchmarks

```rust
#[test]
fn test_new_tool_performance() {
    let mut harness = create_initialized_harness().expect("Failed to create harness");

    let metrics = run_performance_test("New Tool", 20, || {
        let start = Instant::now();
        let response = harness.call_tool("new_tool_name", json!({}))?;
        if response.is_error { anyhow::bail!("Tool returned error"); }
        Ok(start.elapsed())
    });

    assert!(metrics.avg_duration < Duration::from_secs(5), "Tool should be reasonably fast");
}
```

## Test Data Management

### Unique Test Keys
```rust
let test_key = format!("test_{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default());
```

### Test Data Cleanup
- Tests use timestamped keys to avoid conflicts
- AI memory searches are scoped to test-specific tags
- No persistent state between test runs

### Mock Data Patterns
```rust
let test_data = json!({
    "test_purpose": "specific test description",
    "timestamp": chrono::Utc::now().to_rfc3339(),
    "data": "test-specific content"
});
```

This comprehensive testing system ensures the Thalora MCP server provides reliable, performant, and standards-compliant service for AI model integration.