/// Integration tests for CDP debugging tools with session management
/// Tests how CDP tools and browser sessions work together for complete debugging workflows
use serde_json::json;

// Import from the local crate
use thalora::protocols::browser_tools::BrowserTools;
use thalora::protocols::cdp::CdpServer;
use thalora::protocols::cdp_tools::CdpTools;
use thalora::protocols::mcp::McpResponse;

/// Helper function to create test components
fn create_test_components() -> (CdpTools, CdpServer, BrowserTools) {
    (CdpTools::new(), CdpServer::new(), BrowserTools::new())
}

/// Helper function to extract text content from MCP response
fn extract_response_text(response: &McpResponse) -> Option<String> {
    match response {
        McpResponse::ToolResult { content, .. } => {
            if let Some(first_content) = content.first() {
                first_content
                    .get("text")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Test that verifies CDP debugging tools work within browser sessions
#[tokio::test]
async fn test_cdp_debugging_within_session() {
    let (mut cdp_tools, mut cdp_server, browser_tools) = create_test_components();

    // Step 1: Create a debugging session
    let create_session_args = json!({
        "action": "create",
        "session_id": "debug_session",
        "persistent": true
    });

    let session_response = browser_tools
        .handle_session_management(create_session_args)
        .await;
    match &session_response {
        McpResponse::ToolResult { is_error, .. } => {
            assert!(!is_error, "Debug session creation should succeed");
        }
        _ => panic!("Expected ToolResult for session creation"),
    }

    // Step 2: Use CDP tools to debug the session

    // Get DOM document structure
    let dom_args = json!({
        "depth": 3
    });
    let dom_response = cdp_tools.get_document(dom_args, &mut cdp_server).await;

    match &dom_response {
        McpResponse::ToolResult { content, .. } => {
            assert!(
                !content.is_empty(),
                "DOM document response should have content"
            );
        }
        _ => panic!("Expected ToolResult for DOM document"),
    }

    // Query for specific elements
    let query_args = json!({
        "selector": "body",
        "node_id": 1
    });
    let query_response = cdp_tools.query_selector(query_args, &mut cdp_server).await;

    match &query_response {
        McpResponse::ToolResult { content, .. } => {
            assert!(
                !content.is_empty(),
                "Query selector response should have content"
            );
        }
        _ => panic!("Expected ToolResult for query selector"),
    }

    // Get cookies for debugging authentication
    let cookies_args = json!({});
    let cookies_response = cdp_tools.get_cookies(cookies_args, &mut cdp_server).await;

    match &cookies_response {
        McpResponse::ToolResult { content, .. } => {
            assert!(!content.is_empty(), "Cookies response should have content");
        }
        _ => panic!("Expected ToolResult for cookies"),
    }

    // Step 3: Get page content through session
    let content_args = json!({
        "session_id": "debug_session"
    });
    let content_response = browser_tools.handle_get_page_content(content_args).await;

    match &content_response {
        McpResponse::ToolResult { content, .. } => {
            assert!(
                !content.is_empty(),
                "Page content response should have content"
            );
        }
        _ => panic!("Expected ToolResult for page content"),
    }

    // Step 4: Close the debugging session
    let close_args = json!({
        "action": "close",
        "session_id": "debug_session"
    });
    let close_response = browser_tools.handle_session_management(close_args).await;

    match &close_response {
        McpResponse::ToolResult { is_error, .. } => {
            assert!(!is_error, "Session closing should succeed");
        }
        _ => panic!("Expected ToolResult for session closing"),
    }
}

/// Test debugging workflow: navigation, inspection, and manipulation
#[tokio::test]
async fn test_complete_debugging_workflow() {
    let (mut cdp_tools, mut cdp_server, browser_tools) = create_test_components();

    let session_id = "workflow_session";

    // Create session for debugging workflow
    let create_args = json!({
        "action": "create",
        "session_id": session_id,
        "persistent": false
    });
    let _session_response = browser_tools.handle_session_management(create_args).await;

    // Simulate debugging workflow steps:

    // 1. Take initial screenshot
    let screenshot_args = json!({
        "format": "png",
        "full_page": false
    });
    let screenshot_response = cdp_tools
        .take_screenshot(screenshot_args, &mut cdp_server)
        .await;

    match &screenshot_response {
        McpResponse::ToolResult { content, .. } => {
            assert!(
                !content.is_empty(),
                "Screenshot response should have content"
            );
        }
        _ => panic!("Expected ToolResult for screenshot"),
    }

    // 2. Inspect page elements
    let inspect_args = json!({
        "selector": "input[type='text']",
        "node_id": 1
    });
    let inspect_response = cdp_tools
        .query_selector(inspect_args, &mut cdp_server)
        .await;

    match &inspect_response {
        McpResponse::ToolResult { content, .. } => {
            assert!(
                !content.is_empty(),
                "Element inspection should have content"
            );
        }
        _ => panic!("Expected ToolResult for element inspection"),
    }

    // 3. Get element attributes (simulating debugging form elements)
    let attr_args = json!({
        "node_id": 2
    });
    let attr_response = cdp_tools.get_attributes(attr_args, &mut cdp_server).await;

    match &attr_response {
        McpResponse::ToolResult { content, .. } => {
            assert!(
                !content.is_empty(),
                "Attributes response should have content"
            );
        }
        _ => panic!("Expected ToolResult for attributes"),
    }

    // 4. Check console for JavaScript errors
    let console_args = json!({
        "level": "error",
        "limit": 10
    });
    let console_response = cdp_tools
        .get_console_messages(console_args, &mut cdp_server)
        .await;

    match &console_response {
        McpResponse::ToolResult { content, .. } => {
            assert!(!content.is_empty(), "Console messages should have content");
        }
        _ => panic!("Expected ToolResult for console messages"),
    }

    // 5. Set debugging cookie
    let cookie_args = json!({
        "name": "debug_mode",
        "value": "enabled",
        "path": "/",
        "secure": false
    });
    let cookie_response = cdp_tools.set_cookie(cookie_args, &mut cdp_server).await;

    match &cookie_response {
        McpResponse::ToolResult { content, .. } => {
            assert!(!content.is_empty(), "Set cookie should have content");
        }
        _ => panic!("Expected ToolResult for set cookie"),
    }

    // 6. Reload page to test with debug cookie
    let reload_args = json!({
        "ignore_cache": true
    });
    let reload_response = cdp_tools.reload_page(reload_args, &mut cdp_server).await;

    match &reload_response {
        McpResponse::ToolResult { content, .. } => {
            assert!(!content.is_empty(), "Page reload should have content");
        }
        _ => panic!("Expected ToolResult for page reload"),
    }

    // 7. Navigate in browser history
    let back_args = json!({
        "session_id": session_id
    });
    let back_response = browser_tools.handle_navigate_back(back_args).await;

    match &back_response {
        McpResponse::ToolResult { content, .. } => {
            assert!(!content.is_empty(), "Navigate back should have content");
        }
        _ => panic!("Expected ToolResult for navigate back"),
    }

    // 8. Get final page state
    let final_content_args = json!({
        "session_id": session_id
    });
    let final_content_response = browser_tools
        .handle_get_page_content(final_content_args)
        .await;

    match &final_content_response {
        McpResponse::ToolResult { content, .. } => {
            assert!(
                !content.is_empty(),
                "Final page content should have content"
            );
        }
        _ => panic!("Expected ToolResult for final page content"),
    }
}

/// Test error debugging scenario: finding and fixing issues
#[tokio::test]
async fn test_error_debugging_scenario() {
    let (mut cdp_tools, mut cdp_server, browser_tools) = create_test_components();

    let session_id = "error_debug_session";

    // Create debugging session
    let create_args = json!({
        "action": "create",
        "session_id": session_id,
        "persistent": false
    });
    let _session_response = browser_tools.handle_session_management(create_args).await;

    // Debugging scenario: form submission not working

    // 1. Check console for JavaScript errors
    let console_args = json!({
        "level": "error"
    });
    let console_response = cdp_tools
        .get_console_messages(console_args, &mut cdp_server)
        .await;

    match &console_response {
        McpResponse::ToolResult { content, .. } => {
            assert!(
                !content.is_empty(),
                "Console errors check should have content"
            );
            if let Some(text) = extract_response_text(&console_response) {
                assert!(
                    text.contains("Console") || text.contains("messages") || text.contains("CDP"),
                    "Console response should mention console or messages: {}",
                    text
                );
            }
        }
        _ => panic!("Expected ToolResult for console errors"),
    }

    // 2. Inspect form elements
    let form_query_args = json!({
        "selector": "form",
        "node_id": 1
    });
    let form_response = cdp_tools
        .query_selector(form_query_args, &mut cdp_server)
        .await;

    match form_response {
        McpResponse::ToolResult { content, .. } => {
            assert!(!content.is_empty(), "Form query should have content");
        }
        _ => panic!("Expected ToolResult for form query"),
    }

    // 3. Check form element attributes
    let form_attr_args = json!({
        "node_id": 5
    });
    let form_attr_response = cdp_tools
        .get_attributes(form_attr_args, &mut cdp_server)
        .await;

    match form_attr_response {
        McpResponse::ToolResult { content, .. } => {
            assert!(!content.is_empty(), "Form attributes should have content");
        }
        _ => panic!("Expected ToolResult for form attributes"),
    }

    // 4. Check computed styles (maybe CSS is hiding the form)
    let styles_args = json!({
        "node_id": 5
    });
    let styles_response = cdp_tools
        .get_computed_style(styles_args, &mut cdp_server)
        .await;

    match styles_response {
        McpResponse::ToolResult { content, .. } => {
            assert!(!content.is_empty(), "Computed styles should have content");
        }
        _ => panic!("Expected ToolResult for computed styles"),
    }

    // 5. Check authentication cookies
    let auth_cookies_args = json!({
        "urls": ["https://example.com"]
    });
    let auth_cookies_response = cdp_tools
        .get_cookies(auth_cookies_args, &mut cdp_server)
        .await;

    match auth_cookies_response {
        McpResponse::ToolResult { content, .. } => {
            assert!(
                !content.is_empty(),
                "Auth cookies check should have content"
            );
        }
        _ => panic!("Expected ToolResult for auth cookies"),
    }

    // 6. Take screenshot to see visual state
    let debug_screenshot_args = json!({
        "format": "jpeg",
        "quality": 95
    });
    let debug_screenshot_response = cdp_tools
        .take_screenshot(debug_screenshot_args, &mut cdp_server)
        .await;

    match debug_screenshot_response {
        McpResponse::ToolResult { content, .. } => {
            assert!(!content.is_empty(), "Debug screenshot should have content");
        }
        _ => panic!("Expected ToolResult for debug screenshot"),
    }
}

/// Test session persistence during debugging operations
#[tokio::test]
async fn test_session_persistence_during_debugging() {
    let (mut cdp_tools, mut cdp_server, browser_tools) = create_test_components();

    // Create persistent debugging session
    let create_args = json!({
        "action": "create",
        "session_id": "persistent_debug",
        "persistent": true
    });
    let create_response = browser_tools.handle_session_management(create_args).await;

    match create_response {
        McpResponse::ToolResult { is_error, .. } => {
            assert!(!is_error, "Persistent session creation should succeed");
        }
        _ => panic!("Expected ToolResult for session creation"),
    }

    // Perform multiple debugging operations
    let debug_operations = vec![
        ("screenshot", json!({"format": "png"})),
        ("query_selector", json!({"selector": "div", "node_id": 1})),
        ("get_cookies", json!({})),
        ("console_messages", json!({"level": "warn"})),
    ];

    for (operation, args) in debug_operations {
        let response = match operation {
            "screenshot" => cdp_tools.take_screenshot(args, &mut cdp_server).await,
            "query_selector" => cdp_tools.query_selector(args, &mut cdp_server).await,
            "get_cookies" => cdp_tools.get_cookies(args, &mut cdp_server).await,
            "console_messages" => cdp_tools.get_console_messages(args, &mut cdp_server).await,
            _ => panic!("Unknown operation: {}", operation),
        };

        match response {
            McpResponse::ToolResult { content, .. } => {
                assert!(
                    !content.is_empty(),
                    "Operation {} should have content",
                    operation
                );
            }
            _ => panic!("Expected ToolResult for operation {}", operation),
        }
    }

    // Check session info after operations
    let info_args = json!({
        "action": "info",
        "session_id": "persistent_debug"
    });
    let info_response = browser_tools.handle_session_management(info_args).await;

    match info_response {
        McpResponse::ToolResult { content, .. } => {
            assert!(!content.is_empty(), "Session info should have content");
        }
        _ => panic!("Expected ToolResult for session info"),
    }

    // Get page content to verify session state
    let content_args = json!({
        "session_id": "persistent_debug"
    });
    let content_response = browser_tools.handle_get_page_content(content_args).await;

    match &content_response {
        McpResponse::ToolResult { content, .. } => {
            assert!(!content.is_empty(), "Page content should have content");
        }
        _ => panic!("Expected ToolResult for page content"),
    }

    // Clean up persistent session
    let close_args = json!({
        "action": "close",
        "session_id": "persistent_debug"
    });
    let close_response = browser_tools.handle_session_management(close_args).await;

    match &close_response {
        McpResponse::ToolResult { is_error, .. } => {
            assert!(!is_error, "Session closing should succeed");
        }
        _ => panic!("Expected ToolResult for session closing"),
    }
}

/// Test concurrent debugging sessions
#[tokio::test]
async fn test_concurrent_debugging_sessions() {
    let (mut cdp_tools, mut cdp_server, browser_tools) = create_test_components();

    // Create multiple debugging sessions
    let sessions = vec!["debug_session_1", "debug_session_2", "debug_session_3"];

    for session_id in &sessions {
        let create_args = json!({
            "action": "create",
            "session_id": session_id,
            "persistent": false
        });
        let create_response = browser_tools.handle_session_management(create_args).await;

        match create_response {
            McpResponse::ToolResult { is_error, .. } => {
                assert!(!is_error, "Session {} creation should succeed", session_id);
            }
            _ => panic!("Expected ToolResult for session {} creation", session_id),
        }
    }

    // List all sessions
    let list_args = json!({
        "action": "list"
    });
    let list_response = browser_tools.handle_session_management(list_args).await;

    match list_response {
        McpResponse::ToolResult { content, is_error } => {
            assert!(!is_error, "Session listing should succeed");
            assert!(!content.is_empty(), "Session list should have content");
        }
        _ => panic!("Expected ToolResult for session listing"),
    }

    // Perform debugging operations on different sessions
    for (i, session_id) in sessions.iter().enumerate() {
        // Different debugging operations for each session
        match i {
            0 => {
                // Session 1: DOM inspection
                let query_args = json!({
                    "selector": format!("div.session-{}", i),
                    "node_id": 1
                });
                let _query_response = cdp_tools.query_selector(query_args, &mut cdp_server).await;
            }
            1 => {
                // Session 2: Cookie debugging
                let cookie_args = json!({
                    "name": format!("session_{}", i),
                    "value": format!("test_{}", i)
                });
                let _cookie_response = cdp_tools.set_cookie(cookie_args, &mut cdp_server).await;
            }
            2 => {
                // Session 3: Console monitoring
                let console_args = json!({
                    "level": "log"
                });
                let _console_response = cdp_tools
                    .get_console_messages(console_args, &mut cdp_server)
                    .await;
            }
            _ => {}
        }

        // Get page content for each session
        let content_args = json!({
            "session_id": session_id
        });
        let content_response = browser_tools.handle_get_page_content(content_args).await;

        match &content_response {
            McpResponse::ToolResult { content, .. } => {
                assert!(
                    !content.is_empty(),
                    "Session {} should have page content",
                    session_id
                );
            }
            _ => panic!(
                "Expected ToolResult for session {} page content",
                session_id
            ),
        }
    }

    // Clean up all sessions
    for session_id in &sessions {
        let close_args = json!({
            "action": "close",
            "session_id": session_id
        });
        let _close_response = browser_tools.handle_session_management(close_args).await;
    }
}
