// MCP Test Harness - Utilities for testing the stdio-based MCP server
use std::process::{Command, Stdio, Child};
use std::io::{BufRead, BufReader, Write};
use std::time::{Duration, Instant};
use serde_json::{Value, json};
use anyhow::{Result, Context, bail};

/// Test harness for the MCP server subprocess
pub struct McpTestHarness {
    pub process: Child,
    next_id: u64,
}

/// Configuration for MCP test harness
#[derive(Debug, Clone)]
pub struct McpTestConfig {
    pub timeout: Duration,
    pub use_release_build: bool,
    pub debug_output: bool,
    pub env_vars: std::collections::HashMap<String, String>,
}

impl Default for McpTestConfig {
    fn default() -> Self {
        let mut env_vars = std::collections::HashMap::new();
        // Enable all tool categories by default for testing (since they default to disabled in production)
        env_vars.insert("THALORA_ENABLE_AI_MEMORY".to_string(), "true".to_string());
        env_vars.insert("THALORA_ENABLE_CDP".to_string(), "true".to_string());
        env_vars.insert("THALORA_ENABLE_SCRAPING".to_string(), "true".to_string());
        env_vars.insert("THALORA_ENABLE_SEARCH".to_string(), "true".to_string());
        env_vars.insert("THALORA_ENABLE_BROWSER_AUTOMATION".to_string(), "true".to_string());
        env_vars.insert("THALORA_ENABLE_SESSION_MANAGEMENT".to_string(), "true".to_string());

        Self {
            timeout: Duration::from_secs(30),
            use_release_build: true,  // Use release builds by default for faster startup
            debug_output: false,
            env_vars,
        }
    }
}

/// Response from an MCP tool call
#[derive(Debug, Clone)]
pub struct McpToolResponse {
    pub id: Value,
    pub content: Vec<Value>,
    pub is_error: bool,
    pub duration: Duration,
}

impl McpTestHarness {
    /// Create a new MCP test harness with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(McpTestConfig::default())
    }

    /// Create a new MCP test harness with custom configuration
    pub fn with_config(config: McpTestConfig) -> Result<Self> {
        let binary_path = if config.use_release_build {
            "./target/release/thalora"
        } else {
            // For debug builds, we'll use cargo run
            "cargo"
        };

        let mut cmd = if config.use_release_build {
            Command::new(binary_path)
        } else {
            let mut cmd = Command::new("cargo");
            cmd.args(&["run", "--quiet", "--bin", "thalora"]);
            cmd
        };

        cmd.env("THALORA_SILENT", "1");  // Suppress debug output for clean JSON responses

        // Apply custom environment variables
        for (key, value) in &config.env_vars {
            cmd.env(key, value);
        }

        let mut process = cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(if config.debug_output { Stdio::inherit() } else { Stdio::piped() })
            .spawn()
            .context("Failed to spawn MCP server process")?;

        // Give the process a moment to start up
        std::thread::sleep(Duration::from_millis(100));

        // Check if process is still running
        if let Ok(Some(exit_status)) = process.try_wait() {
            bail!("MCP server process exited immediately with status: {}", exit_status);
        }

        Ok(Self {
            process,
            next_id: 1,
        })
    }

    /// Initialize the MCP connection
    pub fn initialize(&mut self) -> Result<Value> {
        let request = json!({
            "jsonrpc": "2.0",
            "id": self.next_id(),
            "method": "initialize",
            "params": {}
        });

        self.send_request_raw(request)
    }

    /// Get the list of available tools
    pub fn list_tools(&mut self) -> Result<Vec<Value>> {
        let request = json!({
            "jsonrpc": "2.0",
            "id": self.next_id(),
            "method": "tools/list",
            "params": {}
        });

        let response = self.send_request_raw(request)?;

        // Extract tools from response - handle different possible formats
        // Try direct tools field first: {"tools": [...]}
        if let Some(tools) = response.get("tools") {
            if let Some(tools_array) = tools.as_array() {
                Ok(tools_array.clone())
            } else {
                bail!("Tools field is not an array: {:?}", tools);
            }
        }
        // Try nested content format: {"content":[{"tools":[...]}],"isError":false}
        else if let Some(content) = response.get("content") {
            if let Some(content_array) = content.as_array() {
                if let Some(first_content) = content_array.first() {
                    if let Some(tools) = first_content.get("tools") {
                        if let Some(tools_array) = tools.as_array() {
                            Ok(tools_array.clone())
                        } else {
                            bail!("Tools field is not an array: {:?}", tools);
                        }
                    } else {
                        bail!("No tools field in content: {:?}", first_content);
                    }
                } else {
                    bail!("Content array is empty: {:?}", content_array);
                }
            } else {
                bail!("Content is not an array: {:?}", content);
            }
        } else {
            bail!("No tools or content field in response: {:?}", response);
        }
    }

    /// Call a tool with the given name and arguments
    pub fn call_tool(&mut self, name: &str, arguments: Value) -> Result<McpToolResponse> {
        let start_time = Instant::now();
        let id = self.next_id();

        let request = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "tools/call",
            "params": {
                "name": name,
                "arguments": arguments
            }
        });

        let response = self.send_request_raw(request)?;
        let duration = start_time.elapsed();

        // Parse the tool response
        let content = response.get("content")
            .and_then(|c| c.as_array())
            .unwrap_or(&vec![])
            .clone();

        let is_error = response.get("isError")
            .and_then(|e| e.as_bool())
            .unwrap_or(false);

        Ok(McpToolResponse {
            id: json!(id),
            content,
            is_error,
            duration,
        })
    }

    /// Send a raw JSON-RPC request and return the response
    pub fn send_request_raw(&mut self, request: Value) -> Result<Value> {
        let request_str = serde_json::to_string(&request)
            .context("Failed to serialize request")?;

        // Send request
        let stdin = self.process.stdin.as_mut()
            .context("Failed to get stdin handle")?;

        writeln!(stdin, "{}", request_str)
            .context("Failed to write request to stdin")?;

        stdin.flush()
            .context("Failed to flush stdin")?;

        // Read response
        let stdout = self.process.stdout.as_mut()
            .context("Failed to get stdout handle")?;

        let mut reader = BufReader::new(stdout);
        let mut response_line = String::new();

        reader.read_line(&mut response_line)
            .context("Failed to read response from stdout")?;

        if response_line.trim().is_empty() {
            bail!("Received empty response from MCP server");
        }

        let response: Value = serde_json::from_str(response_line.trim())
            .context("Failed to parse JSON response")?;

        // Check for JSON-RPC error
        if let Some(error) = response.get("error") {
            bail!("MCP server returned error: {:?}", error);
        }

        // Extract result field if present
        if let Some(result) = response.get("result") {
            Ok(result.clone())
        } else {
            Ok(response)
        }
    }

    /// Get next request ID
    fn next_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Check if the MCP server process is still running
    pub fn is_running(&mut self) -> bool {
        match self.process.try_wait() {
            Ok(None) => true, // Still running
            Ok(Some(_)) => false, // Exited
            Err(_) => false, // Error checking status
        }
    }

    /// Wait for the process to exit with timeout
    pub fn wait_for_exit(&mut self, timeout: Duration) -> Result<std::process::ExitStatus> {
        let start = Instant::now();

        loop {
            if let Ok(Some(status)) = self.process.try_wait() {
                return Ok(status);
            }

            if start.elapsed() > timeout {
                bail!("Process did not exit within timeout");
            }

            std::thread::sleep(Duration::from_millis(10));
        }
    }
}

impl Drop for McpTestHarness {
    fn drop(&mut self) {
        // Try to terminate gracefully first
        if let Err(_) = self.process.kill() {
            // If kill fails, process might already be dead
        }

        // Wait for process to exit
        let _ = self.process.wait();
    }
}

// Helper functions for common test patterns

/// Create a test harness and initialize it
pub fn create_initialized_harness() -> Result<McpTestHarness> {
    let mut harness = McpTestHarness::new()?;
    harness.initialize()?;
    Ok(harness)
}

/// Create a test harness with release build for performance testing
#[allow(dead_code)]
pub fn create_release_harness() -> Result<McpTestHarness> {
    let config = McpTestConfig {
        use_release_build: true,
        ..Default::default()
    };
    let mut harness = McpTestHarness::with_config(config)?;
    harness.initialize()?;
    Ok(harness)
}

/// Create a test harness with custom environment variables
pub fn create_harness_with_env(env_vars: std::collections::HashMap<String, String>) -> Result<McpTestHarness> {
    let config = McpTestConfig {
        env_vars,
        ..Default::default()
    };
    let mut harness = McpTestHarness::with_config(config)?;
    harness.initialize()?;
    Ok(harness)
}

/// Create a test harness with specific tool categories disabled
pub fn create_harness_with_disabled_categories(disabled_categories: &[&str]) -> Result<McpTestHarness> {
    let mut env_vars = std::collections::HashMap::new();

    // Start with all categories enabled
    let all_categories = [
        "THALORA_ENABLE_AI_MEMORY",
        "THALORA_ENABLE_CDP",
        "THALORA_ENABLE_SCRAPING",
        "THALORA_ENABLE_SEARCH",
        "THALORA_ENABLE_BROWSER_AUTOMATION",
        "THALORA_ENABLE_SESSION_MANAGEMENT"
    ];

    for category in all_categories {
        if disabled_categories.contains(&category) {
            env_vars.insert(category.to_string(), "false".to_string());
        } else {
            env_vars.insert(category.to_string(), "true".to_string());
        }
    }

    create_harness_with_env(env_vars)
}

/// Create a test harness with only specific tool categories enabled
pub fn create_harness_with_only_categories(enabled_categories: &[&str]) -> Result<McpTestHarness> {
    let mut env_vars = std::collections::HashMap::new();

    let all_categories = [
        "THALORA_ENABLE_AI_MEMORY",
        "THALORA_ENABLE_CDP",
        "THALORA_ENABLE_SCRAPING",
        "THALORA_ENABLE_SEARCH",
        "THALORA_ENABLE_BROWSER_AUTOMATION",
        "THALORA_ENABLE_SESSION_MANAGEMENT"
    ];

    for category in all_categories {
        if enabled_categories.contains(&category) {
            env_vars.insert(category.to_string(), "true".to_string());
        } else {
            env_vars.insert(category.to_string(), "false".to_string());
        }
    }

    create_harness_with_env(env_vars)
}

/// Validate that a tool response contains expected fields
pub fn validate_tool_response(response: &McpToolResponse, expected_content_type: &str) -> Result<()> {
    if response.is_error {
        bail!("Tool call returned error: {:?}", response.content);
    }

    if response.content.is_empty() {
        bail!("Tool response contains no content");
    }

    // Check if the first content item has the expected structure
    if let Some(first_content) = response.content.first() {
        if let Some(text) = first_content.get("text") {
            if !text.is_string() {
                bail!("Tool response text field is not a string: {:?}", text);
            }
        } else if expected_content_type == "text" {
            bail!("Tool response missing expected text field: {:?}", first_content);
        }
    }

    Ok(())
}

/// Assert that a tool call completes successfully within a time limit
pub fn assert_tool_success(response: &McpToolResponse, max_duration: Duration) -> Result<()> {
    if response.is_error {
        bail!("Tool call failed: {:?}", response.content);
    }

    if response.duration > max_duration {
        bail!("Tool call took too long: {:?} > {:?}", response.duration, max_duration);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_harness_creation() {
        let harness = McpTestHarness::new();
        assert!(harness.is_ok(), "Should be able to create MCP test harness");
    }

    #[test]
    fn test_harness_initialization() {
        let mut harness = McpTestHarness::new().expect("Failed to create harness");
        assert!(harness.is_running(), "MCP server should be running");

        let init_response = harness.initialize();
        assert!(init_response.is_ok(), "Initialization should succeed: {:?}", init_response.err());
    }

    #[test]
    fn test_tools_list() {
        let mut harness = create_initialized_harness().expect("Failed to create harness");

        let tools = harness.list_tools();
        assert!(tools.is_ok(), "Should be able to list tools: {:?}", tools.err());

        let tools = tools.unwrap();
        assert!(!tools.is_empty(), "Should have at least one tool");

        // Verify tool structure
        for tool in &tools {
            assert!(tool.get("name").is_some(), "Tool should have a name: {:?}", tool);
            assert!(tool.get("description").is_some(), "Tool should have a description: {:?}", tool);
            assert!(tool.get("inputSchema").is_some(), "Tool should have an input schema: {:?}", tool);
        }
    }
}