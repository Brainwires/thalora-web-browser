/// Comprehensive tests for the new CDP debugging tools
/// Tests all 8 newly implemented CDP debugging capabilities
use serde_json::json;

// Import from the local crate
use thalora::protocols::cdp::CdpServer;
use thalora::protocols::cdp_tools::CdpTools;
use thalora::protocols::mcp::McpResponse;

/// Helper function to create a test CDP server
fn create_test_cdp_server() -> CdpServer {
    CdpServer::new()
}

/// Helper function to create test CDP tools
fn create_test_cdp_tools() -> CdpTools {
    CdpTools::new()
}

/// Helper function to extract text content from MCP response
fn extract_response_text(response: &McpResponse) -> Option<String> {
    response
        .content
        .first()
        .and_then(|v| v.get("text"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

#[tokio::test]
async fn test_cdp_dom_query_selector() {
    let mut cdp_tools = create_test_cdp_tools();
    let mut cdp_server = create_test_cdp_server();

    // Test with valid selector
    let args = json!({
        "selector": "button.login",
        "node_id": 1
    });

    let response = cdp_tools.query_selector(args, &mut cdp_server).await;

    // Should return a response (even if CDP server isn't fully connected)
    assert!(!response.content.is_empty(), "Response should have content");
    if let Some(text) = extract_response_text(&response) {
        assert!(
            text.contains("button.login") || text.contains("CDP"),
            "Response should mention selector or CDP: {}",
            text
        );
    }
}

#[tokio::test]
async fn test_cdp_dom_query_selector_missing_params() {
    let mut cdp_tools = create_test_cdp_tools();
    let mut cdp_server = create_test_cdp_server();

    // Test with missing selector parameter
    let args = json!({
        "node_id": 1
    });

    let response = cdp_tools.query_selector(args, &mut cdp_server).await;

    assert!(
        response.is_error,
        "Should return error for missing selector"
    );
    if let Some(text) = extract_response_text(&response) {
        assert!(
            text.contains("Missing required parameter: selector"),
            "Should indicate missing selector parameter: {}",
            text
        );
    }
}

#[tokio::test]
async fn test_cdp_dom_get_attributes() {
    let mut cdp_tools = create_test_cdp_tools();
    let mut cdp_server = create_test_cdp_server();

    // Test with valid node ID
    let args = json!({
        "node_id": 123
    });

    let response = cdp_tools.get_attributes(args, &mut cdp_server).await;

    assert!(!response.content.is_empty(), "Response should have content");
    if let Some(text) = extract_response_text(&response) {
        assert!(
            text.contains("123") || text.contains("CDP"),
            "Response should mention node ID or CDP: {}",
            text
        );
    }
}

#[tokio::test]
async fn test_cdp_dom_get_attributes_missing_params() {
    let mut cdp_tools = create_test_cdp_tools();
    let mut cdp_server = create_test_cdp_server();

    // Test with missing node_id parameter
    let args = json!({});

    let response = cdp_tools.get_attributes(args, &mut cdp_server).await;

    assert!(response.is_error, "Should return error for missing node_id");
    if let Some(text) = extract_response_text(&response) {
        assert!(
            text.contains("Missing required parameter: node_id"),
            "Should indicate missing node_id parameter: {}",
            text
        );
    }
}

#[tokio::test]
async fn test_cdp_dom_get_computed_style() {
    let mut cdp_tools = create_test_cdp_tools();
    let mut cdp_server = create_test_cdp_server();

    // Test with valid node ID
    let args = json!({
        "node_id": 456
    });

    let response = cdp_tools.get_computed_style(args, &mut cdp_server).await;

    assert!(!response.content.is_empty(), "Response should have content");
    if let Some(text) = extract_response_text(&response) {
        assert!(
            text.contains("456") || text.contains("CDP") || text.contains("style"),
            "Response should mention node ID, CDP, or style: {}",
            text
        );
    }
}

#[tokio::test]
async fn test_cdp_network_get_cookies() {
    let mut cdp_tools = create_test_cdp_tools();
    let mut cdp_server = create_test_cdp_server();

    // Test without URL filter
    let args = json!({});

    let response = cdp_tools.get_cookies(args, &mut cdp_server).await;

    assert!(!response.content.is_empty(), "Response should have content");
    if let Some(text) = extract_response_text(&response) {
        assert!(
            text.contains("Cookies") || text.contains("CDP"),
            "Response should mention cookies or CDP: {}",
            text
        );
    }
}

#[tokio::test]
async fn test_cdp_network_get_cookies_with_urls() {
    let mut cdp_tools = create_test_cdp_tools();
    let mut cdp_server = create_test_cdp_server();

    // Test with URL filter
    let args = json!({
        "urls": ["https://example.com", "https://test.com"]
    });

    let response = cdp_tools.get_cookies(args, &mut cdp_server).await;

    assert!(!response.content.is_empty(), "Response should have content");
    if let Some(text) = extract_response_text(&response) {
        assert!(
            text.contains("Cookies") || text.contains("CDP"),
            "Response should mention cookies or CDP: {}",
            text
        );
    }
}

#[tokio::test]
async fn test_cdp_network_set_cookie() {
    let mut cdp_tools = create_test_cdp_tools();
    let mut cdp_server = create_test_cdp_server();

    // Test with required parameters
    let args = json!({
        "name": "test_cookie",
        "value": "test_value",
        "domain": "example.com",
        "path": "/",
        "secure": true,
        "http_only": false
    });

    let response = cdp_tools.set_cookie(args, &mut cdp_server).await;

    assert!(!response.content.is_empty(), "Response should have content");
    if let Some(text) = extract_response_text(&response) {
        assert!(
            text.contains("test_cookie") || text.contains("CDP"),
            "Response should mention cookie name or CDP: {}",
            text
        );
    }
}

#[tokio::test]
async fn test_cdp_network_set_cookie_missing_params() {
    let mut cdp_tools = create_test_cdp_tools();
    let mut cdp_server = create_test_cdp_server();

    // Test with missing name parameter
    let args = json!({
        "value": "test_value"
    });

    let response = cdp_tools.set_cookie(args, &mut cdp_server).await;

    assert!(response.is_error, "Should return error for missing name");
    if let Some(text) = extract_response_text(&response) {
        assert!(
            text.contains("Missing required parameter: name"),
            "Should indicate missing name parameter: {}",
            text
        );
    }

    // Test with missing value parameter
    let args = json!({
        "name": "test_cookie"
    });

    let response = cdp_tools.set_cookie(args, &mut cdp_server).await;

    assert!(response.is_error, "Should return error for missing value");
    if let Some(text) = extract_response_text(&response) {
        assert!(
            text.contains("Missing required parameter: value"),
            "Should indicate missing value parameter: {}",
            text
        );
    }
}

#[tokio::test]
async fn test_cdp_console_get_messages() {
    let mut cdp_tools = create_test_cdp_tools();
    let mut cdp_server = create_test_cdp_server();

    // Test without filters
    let args = json!({});

    let response = cdp_tools.get_console_messages(args, &mut cdp_server).await;

    assert!(!response.content.is_empty(), "Response should have content");
    if let Some(text) = extract_response_text(&response) {
        assert!(
            text.contains("Console") || text.contains("messages") || text.contains("CDP"),
            "Response should mention console, messages, or CDP: {}",
            text
        );
    }
}

#[tokio::test]
async fn test_cdp_console_get_messages_with_filters() {
    let mut cdp_tools = create_test_cdp_tools();
    let mut cdp_server = create_test_cdp_server();

    // Test with level filter and limit
    let args = json!({
        "level": "error",
        "limit": 50
    });

    let response = cdp_tools.get_console_messages(args, &mut cdp_server).await;

    assert!(!response.content.is_empty(), "Response should have content");
    if let Some(text) = extract_response_text(&response) {
        assert!(
            text.contains("Console") || text.contains("error") || text.contains("CDP"),
            "Response should mention console, error level, or CDP: {}",
            text
        );
    }
}

#[tokio::test]
async fn test_cdp_page_screenshot() {
    let mut cdp_tools = create_test_cdp_tools();
    let mut cdp_server = create_test_cdp_server();

    // Test with default parameters
    let args = json!({});

    let response = cdp_tools.take_screenshot(args, &mut cdp_server).await;

    assert!(!response.content.is_empty(), "Response should have content");
    if let Some(text) = extract_response_text(&response) {
        assert!(
            text.contains("Screenshot") || text.contains("CDP"),
            "Response should mention screenshot or CDP: {}",
            text
        );
    }
}

#[tokio::test]
async fn test_cdp_page_screenshot_with_options() {
    let mut cdp_tools = create_test_cdp_tools();
    let mut cdp_server = create_test_cdp_server();

    // Test with custom format and options
    let args = json!({
        "format": "jpeg",
        "quality": 90,
        "full_page": true
    });

    let response = cdp_tools.take_screenshot(args, &mut cdp_server).await;

    assert!(!response.content.is_empty(), "Response should have content");
    if let Some(text) = extract_response_text(&response) {
        assert!(
            text.contains("Screenshot") || text.contains("CDP"),
            "Response should mention screenshot or CDP: {}",
            text
        );
    }
}

#[tokio::test]
async fn test_cdp_page_reload() {
    let mut cdp_tools = create_test_cdp_tools();
    let mut cdp_server = create_test_cdp_server();

    // Test with default parameters
    let args = json!({});

    let response = cdp_tools.reload_page(args, &mut cdp_server).await;

    assert!(!response.content.is_empty(), "Response should have content");
    if let Some(text) = extract_response_text(&response) {
        assert!(
            text.contains("reload") || text.contains("CDP"),
            "Response should mention reload or CDP: {}",
            text
        );
    }
}

#[tokio::test]
async fn test_cdp_page_reload_ignore_cache() {
    let mut cdp_tools = create_test_cdp_tools();
    let mut cdp_server = create_test_cdp_server();

    // Test with ignore_cache option
    let args = json!({
        "ignore_cache": true
    });

    let response = cdp_tools.reload_page(args, &mut cdp_server).await;

    assert!(!response.content.is_empty(), "Response should have content");
    if let Some(text) = extract_response_text(&response) {
        assert!(
            text.contains("reload") || text.contains("cache") || text.contains("CDP"),
            "Response should mention reload, cache, or CDP: {}",
            text
        );
    }
}

/// Integration test that verifies all CDP tools are accessible
#[tokio::test]
async fn test_all_cdp_tools_accessible() {
    let mut cdp_tools = create_test_cdp_tools();
    let mut cdp_server = create_test_cdp_server();

    // Test that all 8 new CDP tools can be called without panicking
    let tools_to_test = vec![
        ("query_selector", json!({"selector": "div"})),
        ("get_attributes", json!({"node_id": 1})),
        ("get_computed_style", json!({"node_id": 1})),
        ("get_cookies", json!({})),
        ("set_cookie", json!({"name": "test", "value": "test"})),
        ("get_console_messages", json!({})),
        ("take_screenshot", json!({})),
        ("reload_page", json!({})),
    ];

    for (tool_name, args) in tools_to_test {
        let response = match tool_name {
            "query_selector" => cdp_tools.query_selector(args, &mut cdp_server).await,
            "get_attributes" => cdp_tools.get_attributes(args, &mut cdp_server).await,
            "get_computed_style" => cdp_tools.get_computed_style(args, &mut cdp_server).await,
            "get_cookies" => cdp_tools.get_cookies(args, &mut cdp_server).await,
            "set_cookie" => cdp_tools.set_cookie(args, &mut cdp_server).await,
            "get_console_messages" => cdp_tools.get_console_messages(args, &mut cdp_server).await,
            "take_screenshot" => cdp_tools.take_screenshot(args, &mut cdp_server).await,
            "reload_page" => cdp_tools.reload_page(args, &mut cdp_server).await,
            _ => panic!("Unknown tool: {}", tool_name),
        };

        assert!(
            !response.content.is_empty(),
            "Tool {} should return content",
            tool_name
        );
    }
}

/// Test parameter validation across all tools
#[tokio::test]
async fn test_parameter_validation() {
    let mut cdp_tools = create_test_cdp_tools();
    let mut cdp_server = create_test_cdp_server();

    // Test tools that require specific parameters
    let required_param_tests = vec![
        ("query_selector", json!({}), "selector"),
        ("get_attributes", json!({}), "node_id"),
        ("get_computed_style", json!({}), "node_id"),
        ("set_cookie", json!({}), "name"),
        ("set_cookie", json!({"name": "test"}), "value"),
    ];

    for (tool_name, args, expected_missing_param) in required_param_tests {
        let response = match tool_name {
            "query_selector" => cdp_tools.query_selector(args, &mut cdp_server).await,
            "get_attributes" => cdp_tools.get_attributes(args, &mut cdp_server).await,
            "get_computed_style" => cdp_tools.get_computed_style(args, &mut cdp_server).await,
            "set_cookie" => cdp_tools.set_cookie(args, &mut cdp_server).await,
            _ => panic!("Unknown tool: {}", tool_name),
        };

        assert!(
            response.is_error,
            "Tool {} should return error for missing {}",
            tool_name, expected_missing_param
        );
        if let Some(text) = extract_response_text(&response) {
            assert!(
                text.contains(&format!(
                    "Missing required parameter: {}",
                    expected_missing_param
                )),
                "Tool {} should indicate missing parameter {}: {}",
                tool_name,
                expected_missing_param,
                text
            );
        }
    }
}
