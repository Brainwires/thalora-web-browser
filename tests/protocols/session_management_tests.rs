/// Comprehensive tests for session management tools
/// Tests browser session management, navigation, and page control functionality
use serde_json::{Value, json};

// Import from the local crate
use thalora::protocols::browser_tools::BrowserTools;
use thalora::protocols::mcp::McpResponse;

/// Helper function to create test browser tools
fn create_test_browser_tools() -> BrowserTools {
    BrowserTools::new()
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

/// Helper function to extract JSON content from MCP response
#[allow(dead_code)]
fn extract_response_json(response: &McpResponse) -> Option<Value> {
    response
        .content
        .first()
        .and_then(|v| v.get("text"))
        .and_then(|v| v.as_str())
        .and_then(|s| serde_json::from_str(s).ok())
}

#[tokio::test]
async fn test_session_management_create() {
    let browser_tools = create_test_browser_tools();

    // Test creating a new session
    let args = json!({
        "action": "create",
        "session_id": "test_session_1",
        "persistent": false
    });

    let response = browser_tools.handle_session_management(args).await;

    assert!(!response.is_error, "Session creation should not error");
    assert!(!response.content.is_empty(), "Response should have content");

    if let Some(text) = extract_response_text(&response) {
        assert!(
            text.contains("test_session_1") || text.contains("created"),
            "Response should mention session ID or creation: {}",
            text
        );
    }
}

#[tokio::test]
async fn test_session_management_create_persistent() {
    let browser_tools = create_test_browser_tools();

    // Test creating a persistent session
    let args = json!({
        "action": "create",
        "session_id": "persistent_session",
        "persistent": true
    });

    let response = browser_tools.handle_session_management(args).await;

    assert!(!response.is_error, "Persistent session creation should not error");
    assert!(!response.content.is_empty(), "Response should have content");

    if let Some(text) = extract_response_text(&response) {
        assert!(
            text.contains("persistent_session") && text.contains("persistent"),
            "Response should mention session ID and persistence: {}",
            text
        );
    }
}

#[tokio::test]
async fn test_session_management_list() {
    let browser_tools = create_test_browser_tools();

    // First create a session
    let create_args = json!({
        "action": "create",
        "session_id": "list_test_session",
        "persistent": false
    });
    let _create_response = browser_tools.handle_session_management(create_args).await;

    // Then list sessions
    let list_args = json!({
        "action": "list"
    });

    let response = browser_tools.handle_session_management(list_args).await;

    assert!(!response.is_error, "Session listing should not error");
    assert!(!response.content.is_empty(), "Response should have content");

    if let Some(text) = extract_response_text(&response) {
        assert!(
            text.contains("sessions") || text.contains("list_test_session"),
            "Response should mention sessions or the created session: {}",
            text
        );
    }
}

#[tokio::test]
async fn test_session_management_info() {
    let browser_tools = create_test_browser_tools();

    // First create a session
    let create_args = json!({
        "action": "create",
        "session_id": "info_test_session",
        "persistent": false
    });
    let _create_response = browser_tools.handle_session_management(create_args).await;

    // Then get info about the session
    let info_args = json!({
        "action": "info",
        "session_id": "info_test_session"
    });

    let response = browser_tools.handle_session_management(info_args).await;

    // Note: This might error if session doesn't exist in the implementation
    assert!(!response.content.is_empty(), "Response should have content");

    if let Some(text) = extract_response_text(&response) {
        assert!(
            text.contains("info_test_session") || text.contains("Session not found"),
            "Response should mention session ID or not found: {}",
            text
        );
    }
}

#[tokio::test]
async fn test_session_management_close() {
    let browser_tools = create_test_browser_tools();

    // First create a session
    let create_args = json!({
        "action": "create",
        "session_id": "close_test_session",
        "persistent": false
    });
    let _create_response = browser_tools.handle_session_management(create_args).await;

    // Then close the session
    let close_args = json!({
        "action": "close",
        "session_id": "close_test_session"
    });

    let response = browser_tools.handle_session_management(close_args).await;

    assert!(!response.is_error, "Session closing should not error");
    assert!(!response.content.is_empty(), "Response should have content");

    if let Some(text) = extract_response_text(&response) {
        assert!(
            text.contains("close_test_session")
                && (text.contains("closed") || text.contains("true")),
            "Response should mention session ID and closure: {}",
            text
        );
    }
}

#[tokio::test]
async fn test_session_management_cleanup() {
    let browser_tools = create_test_browser_tools();

    // Test session cleanup
    let cleanup_args = json!({
        "action": "cleanup",
        "max_age_seconds": 3600
    });

    let response = browser_tools.handle_session_management(cleanup_args).await;

    assert!(!response.is_error, "Session cleanup should not error");
    assert!(!response.content.is_empty(), "Response should have content");

    if let Some(text) = extract_response_text(&response) {
        assert!(
            text.contains("cleaned_up") || text.contains("cleanup"),
            "Response should mention cleanup: {}",
            text
        );
    }
}

#[tokio::test]
async fn test_session_management_invalid_action() {
    let browser_tools = create_test_browser_tools();

    // Test with invalid action
    let args = json!({
        "action": "invalid_action"
    });

    let response = browser_tools.handle_session_management(args).await;

    // Invalid action should return an error
    assert!(response.is_error, "Expected error response for invalid action");
    if let Some(text) = extract_response_text(&response) {
        assert!(
            text.contains("Unknown action") || text.contains("invalid_action"),
            "Error should mention unknown action: {}",
            text
        );
        eprintln!("INFO: Invalid action handled correctly: {}", text);
    }
}

#[tokio::test]
async fn test_browser_get_page_content() {
    let browser_tools = create_test_browser_tools();

    // Test getting page content with default session
    let args = json!({});

    let response = browser_tools.handle_get_page_content(args).await;

    assert!(!response.content.is_empty(), "Response should have content");

    if let Some(text) = extract_response_text(&response) {
        assert!(
            text.contains("content") || text.contains("url") || text.contains("session_id"),
            "Response should mention content, URL, or session ID: {}",
            text
        );
    }
}

#[tokio::test]
async fn test_browser_get_page_content_with_session() {
    let browser_tools = create_test_browser_tools();

    // Test getting page content with specific session
    let args = json!({
        "session_id": "content_test_session"
    });

    let response = browser_tools.handle_get_page_content(args).await;

    assert!(!response.content.is_empty(), "Response should have content");

    if let Some(text) = extract_response_text(&response) {
        assert!(
            text.contains("content_test_session")
                || text.contains("content")
                || text.contains("url"),
            "Response should mention session ID, content, or URL: {}",
            text
        );
    }
}

#[tokio::test]
async fn test_browser_navigate_back() {
    let browser_tools = create_test_browser_tools();

    // Test navigation back with default session
    let args = json!({});

    let response = browser_tools.handle_navigate_back(args).await;

    assert!(!response.content.is_empty(), "Response should have content");

    if let Some(text) = extract_response_text(&response) {
        assert!(
            text.contains("back")
                || text.contains("Cannot go back")
                || text.contains("success"),
            "Response should mention navigation or inability to go back: {}",
            text
        );
    }
}

#[tokio::test]
async fn test_browser_navigate_back_with_session() {
    let browser_tools = create_test_browser_tools();

    // Test navigation back with specific session
    let args = json!({
        "session_id": "back_test_session"
    });

    let response = browser_tools.handle_navigate_back(args).await;

    assert!(!response.content.is_empty(), "Response should have content");

    if let Some(text) = extract_response_text(&response) {
        assert!(
            text.contains("back")
                || text.contains("Cannot go back")
                || text.contains("success"),
            "Response should mention navigation: {}",
            text
        );
    }
}

#[tokio::test]
async fn test_browser_navigate_forward() {
    let browser_tools = create_test_browser_tools();

    // Test navigation forward with default session
    let args = json!({});

    let response = browser_tools.handle_navigate_forward(args).await;

    assert!(!response.content.is_empty(), "Response should have content");

    if let Some(text) = extract_response_text(&response) {
        assert!(
            text.contains("forward")
                || text.contains("Cannot go forward")
                || text.contains("success"),
            "Response should mention navigation or inability to go forward: {}",
            text
        );
    }
}

#[tokio::test]
async fn test_browser_navigate_forward_with_session() {
    let browser_tools = create_test_browser_tools();

    // Test navigation forward with specific session
    let args = json!({
        "session_id": "forward_test_session"
    });

    let response = browser_tools.handle_navigate_forward(args).await;

    assert!(!response.content.is_empty(), "Response should have content");

    if let Some(text) = extract_response_text(&response) {
        assert!(
            text.contains("forward")
                || text.contains("Cannot go forward")
                || text.contains("success"),
            "Response should mention navigation: {}",
            text
        );
    }
}

/// Integration test that verifies session tools work with browser automation
#[tokio::test]
async fn test_session_integration_with_browser_tools() {
    let browser_tools = create_test_browser_tools();

    // Create a session
    let create_args = json!({
        "action": "create",
        "session_id": "integration_session",
        "persistent": false
    });
    let create_response = browser_tools.handle_session_management(create_args).await;
    assert!(!create_response.is_error, "Session creation should succeed");

    // Test click element with the session
    let click_args = json!({
        "selector": "button.test",
        "session_id": "integration_session"
    });
    let click_response = browser_tools.handle_click_element(click_args).await;

    if click_response.is_error {
        // Expected - clicking on non-existent elements should return an error
        if let Some(text) = extract_response_text(&click_response) {
            assert!(
                text.contains("Failed to click element")
                    || text.contains("element")
                    || text.contains("selector"),
                "Error should mention click failure: {}",
                text
            );
            eprintln!("INFO: Click element test handled expected error: {}", text);
        }
    } else {
        assert!(!click_response.content.is_empty(), "Click response should have content");
    }

    // Test form filling with the session
    let form_args = json!({
        "form_data": {"username": "test", "password": "test123"},
        "session_id": "integration_session"
    });
    let form_response = browser_tools.handle_fill_form(form_args).await;

    if form_response.is_error {
        // Expected - filling forms on pages with no forms should return an error
        if let Some(text) = extract_response_text(&form_response) {
            assert!(
                text.contains("form") || text.contains("Form") || text.contains("fill"),
                "Error should mention form filling failure: {}",
                text
            );
            eprintln!("INFO: Form filling test handled expected error: {}", text);
        }
    } else {
        assert!(!form_response.content.is_empty(), "Form response should have content");
    }

    // Get page content for the session
    let content_args = json!({
        "session_id": "integration_session"
    });
    let content_response = browser_tools.handle_get_page_content(content_args).await;
    assert!(!content_response.content.is_empty(), "Content response should have content");

    // Close the session
    let close_args = json!({
        "action": "close",
        "session_id": "integration_session"
    });
    let close_response = browser_tools.handle_session_management(close_args).await;
    assert!(!close_response.is_error, "Session closing should succeed");
}

/// Test session persistence and cleanup functionality
#[tokio::test]
async fn test_session_persistence_workflow() {
    let browser_tools = create_test_browser_tools();

    // Create multiple sessions with different persistence settings
    let sessions = vec![
        ("temp_session_1", false),
        ("persistent_session_1", true),
        ("temp_session_2", false),
        ("persistent_session_2", true),
    ];

    // Create all sessions
    for (session_id, persistent) in &sessions {
        let create_args = json!({
            "action": "create",
            "session_id": session_id,
            "persistent": persistent
        });
        let response = browser_tools.handle_session_management(create_args).await;
        assert!(
            !response.is_error,
            "Session creation should succeed for {}",
            session_id
        );
    }

    // List all sessions
    let list_args = json!({
        "action": "list"
    });
    let list_response = browser_tools.handle_session_management(list_args).await;
    assert!(!list_response.is_error, "Session listing should succeed");
    assert!(!list_response.content.is_empty(), "List response should have content");

    // Test cleanup (should remove non-persistent sessions older than threshold)
    let cleanup_args = json!({
        "action": "cleanup",
        "max_age_seconds": 0  // Clean up all non-persistent sessions
    });
    let cleanup_response = browser_tools.handle_session_management(cleanup_args).await;
    assert!(!cleanup_response.is_error, "Session cleanup should succeed");

    // Close remaining persistent sessions
    for (session_id, persistent) in &sessions {
        if *persistent {
            let close_args = json!({
                "action": "close",
                "session_id": session_id
            });
            let _close_response = browser_tools.handle_session_management(close_args).await;
        }
    }
}

/// Test error handling for session management
#[tokio::test]
async fn test_session_error_handling() {
    let browser_tools = create_test_browser_tools();

    // Test missing action parameter
    let args = json!({
        "session_id": "test"
    });
    let response = browser_tools.handle_session_management(args).await;

    // Missing action parameter should return an error
    assert!(response.is_error, "Expected error for missing action parameter");
    if let Some(text) = extract_response_text(&response) {
        assert!(
            text.contains("Unknown action") || text.contains("action"),
            "Error should mention missing/unknown action: {}",
            text
        );
        eprintln!("INFO: Missing action parameter handled correctly: {}", text);
    }

    // Test info for non-existent session
    let info_args = json!({
        "action": "info",
        "session_id": "non_existent_session"
    });
    let info_response = browser_tools.handle_session_management(info_args).await;

    // Non-existent session info should return an error
    assert!(info_response.is_error, "Expected error for non-existent session info");
    if let Some(text) = extract_response_text(&info_response) {
        assert!(
            text.contains("Session not found") || text.contains("not found"),
            "Error should mention session not found: {}",
            text
        );
        eprintln!("INFO: Non-existent session info handled correctly: {}", text);
    }

    // Test close for non-existent session
    let close_args = json!({
        "action": "close",
        "session_id": "non_existent_session"
    });
    let close_response = browser_tools.handle_session_management(close_args).await;

    assert!(!close_response.content.is_empty(), "Response should have content");
    if let Some(text) = extract_response_text(&close_response) {
        assert!(
            text.contains("closed") || text.contains("false"),
            "Response should indicate session was not found or closed: {}",
            text
        );
    }
}

#[tokio::test]
async fn test_browser_refresh_page() {
    let browser_tools = create_test_browser_tools();

    // Test refresh with default session (should fail with no current URL)
    let args = json!({});

    let response = browser_tools.handle_refresh_page(args).await;

    assert!(!response.content.is_empty(), "Response should have content");
    if response.is_error {
        if let Some(text) = extract_response_text(&response) {
            assert!(
                text.contains("No current URL") || text.contains("Failed to refresh"),
                "Error should mention no current URL: {}",
                text
            );
        }
    }
}

#[tokio::test]
async fn test_browser_navigation_workflow() {
    let browser_tools = create_test_browser_tools();
    let session_id = "navigation_workflow_session";

    // Step 1: Navigate to a page
    let navigate_args = json!({
        "url": "https://httpbin.org/html",
        "session_id": session_id,
        "wait_for_load": false
    });

    let navigate_response = browser_tools.handle_navigate_to(navigate_args).await;

    if navigate_response.is_error {
        // Network errors are acceptable in test environment
        if let Some(text) = extract_response_text(&navigate_response) {
            eprintln!("INFO: Navigation test handled network error: {}", text);
        }
        return; // Skip rest of test if we can't navigate
    }
    assert!(!navigate_response.content.is_empty(), "Navigate response should have content");

    // Step 2: Get page content to verify session state
    let content_args = json!({
        "session_id": session_id
    });

    let content_response = browser_tools.handle_get_page_content(content_args).await;
    assert!(!content_response.is_error, "Get page content should not error");
    assert!(!content_response.content.is_empty(), "Content response should have content");

    // Step 3: Test refresh functionality - should work now that we have a current URL
    let refresh_args = json!({
        "session_id": session_id
    });

    let refresh_response = browser_tools.handle_refresh_page(refresh_args).await;
    assert!(!refresh_response.content.is_empty(), "Refresh response should have content");
    if refresh_response.is_error {
        if let Some(text) = extract_response_text(&refresh_response) {
            eprintln!("INFO: Refresh test handled error: {}", text);
        }
    }

    // Step 4: Test back navigation (should report cannot go back since only one page)
    let back_args = json!({
        "session_id": session_id
    });

    let back_response = browser_tools.handle_navigate_back(back_args).await;
    assert!(!back_response.content.is_empty(), "Back response should have content");
}
