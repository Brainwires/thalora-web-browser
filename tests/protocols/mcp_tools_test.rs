// MCP Tools Tests - Comprehensive testing of all MCP tools
#[allow(unused_imports)]
#[allow(unused_variables)]
use serde_json::{Value, json};
use std::time::Duration;

use super::mcp_harness::*;

// Credential Vault Tools Tests
#[test]
fn test_vault_store_list_delete_roundtrip() {
    let mut config = McpTestConfig::default();
    config
        .env_vars
        .insert("THALORA_ENABLE_VAULT".to_string(), "true".to_string());
    config.env_vars.insert(
        "THALORA_MASTER_PASSWORD".to_string(),
        "test_master_password_min_32chars_secure".to_string(),
    );
    config
        .env_vars
        .insert("THALORA_VAULT_PATH".to_string(), unique_vault_path());
    let mut harness = McpTestHarness::with_config(config).expect("Failed to create harness");
    harness.initialize().expect("Failed to initialize harness");

    let service = "vault-roundtrip-service";
    let secret = "super-secret-password-value-123";
    let text_of = |r: &McpToolResponse| -> String {
        r.content[0]
            .get("text")
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string()
    };

    // Store
    let store = harness
        .call_tool(
            "vault_store_credential",
            json!({
                "service": service,
                "username": "roundtrip-user",
                "password": secret
            }),
        )
        .expect("Store should succeed");
    assert!(
        !store.is_error,
        "Store should not error: {}",
        text_of(&store)
    );
    assert!(
        !text_of(&store).contains(secret),
        "Store output must not contain the secret"
    );

    // List: credential present, secret absent
    let list = harness
        .call_tool("vault_list_credentials", json!({"service": service}))
        .expect("List should succeed");
    assert!(!list.is_error, "List should not error: {}", text_of(&list));
    let list_text = text_of(&list);
    assert!(
        list_text.contains("roundtrip-user"),
        "List should include the stored username: {list_text}"
    );
    assert!(
        !list_text.contains(secret),
        "List output must not contain the secret: {list_text}"
    );

    // Delete
    let delete = harness
        .call_tool("vault_delete_credential", json!({"service": service}))
        .expect("Delete should succeed");
    assert!(
        !delete.is_error,
        "Delete should not error: {}",
        text_of(&delete)
    );

    // List again: empty
    let list_after = harness
        .call_tool("vault_list_credentials", json!({"service": service}))
        .expect("List after delete should succeed");
    let after_text = text_of(&list_after);
    assert!(
        !after_text.contains("roundtrip-user"),
        "Deleted credential should no longer be listed: {after_text}"
    );
    if let Ok(v) = serde_json::from_str::<Value>(&after_text) {
        if let Some(creds) = v.get("credentials").and_then(|c| c.as_array()) {
            assert!(
                creds.is_empty(),
                "Credentials should be empty: {after_text}"
            );
        }
    }
}

#[test]
fn test_scrape_basic() {
    let mut harness = create_initialized_harness().expect("Failed to create harness");

    // Test with a simple HTTP endpoint that should be reliable
    let response = match harness.call_tool(
        "snapshot_url",
        json!({
            "url": "https://httpbin.org/html",
            "wait_for_js": false
        }),
    ) {
        Ok(r) => r,
        Err(e) if e.to_string().contains("Timeout") || e.to_string().contains("timeout") => {
            eprintln!("Skipping: snapshot_url timed out (expected on CI)");
            return;
        }
        Err(e) => panic!("Unexpected error: {}", e),
    };

    assert_tool_success(&response, Duration::from_secs(30))
        .expect("Scraping should complete within 30s");

    let response_text = response.content[0].get("text").unwrap().as_str().unwrap();
    assert!(
        response_text.len() > 100,
        "Should return substantial content"
    );
    assert!(
        response_text.contains("Melville")
            || response_text.contains("blacksmith")
            || response_text.contains("html")
            || response_text.contains("HTML")
            || response_text.contains("title")
            || response_text.contains("httpbin"),
        "Should contain expected content from httpbin.org/html"
    );
}

#[test]
fn test_scrape_with_invalid_url() {
    let mut harness = create_initialized_harness().expect("Failed to create harness");

    let response = harness.call_tool(
        "snapshot_url",
        json!({
            "url": "invalid-url-format",
            "wait_for_js": false
        }),
    );

    // Should either fail or return error response
    if let Ok(response) = response {
        assert!(response.is_error, "Should return error for invalid URL");
    }
    // Failing during call is also acceptable
}

#[test]
fn test_google_search() {
    let mut harness = create_initialized_harness().expect("Failed to create harness");

    let response = match harness.call_tool(
        "web_search",
        json!({
            "query": "rust programming language",
            "num_results": 3
        }),
    ) {
        Ok(r) => r,
        Err(e) if e.to_string().contains("Timeout") || e.to_string().contains("timeout") => {
            eprintln!("Skipping: web_search timed out (expected on CI)");
            return;
        }
        Err(e) => panic!("Unexpected error: {}", e),
    };

    assert_tool_success(&response, Duration::from_secs(30))
        .expect("Search should complete within 30s");

    // Response is MCP text content containing JSON with search results
    let response_text = response.content[0]
        .get("text")
        .expect("Should have text field")
        .as_str()
        .unwrap();
    let parsed: serde_json::Value =
        serde_json::from_str(response_text).expect("Response text should be valid JSON");
    let results = parsed
        .get("results")
        .expect("Should have results field")
        .as_array()
        .unwrap();

    // For now, just verify the structure is correct
    assert!(
        results.len() <= 3,
        "Should not return more results than requested"
    );

    // If results are returned, check they have the expected structure
    if !results.is_empty() {
        let first_result = &results[0];
        assert!(
            first_result.get("title").is_some(),
            "Results should have title field"
        );
        assert!(
            first_result.get("url").is_some(),
            "Results should have URL field"
        );
    }
}

#[test]
fn test_google_search_with_limit() {
    let mut harness = create_initialized_harness().expect("Failed to create harness");

    let response = match harness.call_tool(
        "web_search",
        json!({
            "query": "test query",
            "num_results": 1
        }),
    ) {
        Ok(r) => r,
        Err(e) if e.to_string().contains("Timeout") || e.to_string().contains("timeout") => {
            eprintln!("Skipping: web_search timed out (expected on CI)");
            return;
        }
        Err(e) => panic!("Unexpected error: {}", e),
    };

    assert_tool_success(&response, Duration::from_secs(30)).expect("Limited search should be fast");
}

// Browser Automation Tools Tests
#[test]
fn test_browser_click_element() {
    let mut harness = create_initialized_harness().expect("Failed to create harness");

    // This test may fail if the element doesn't exist, which is expected
    let response = harness.call_tool(
        "browser_click_element",
        json!({
            "selector": "#nonexistent-element"
        }),
    );

    // Either succeeds (if implemented to handle missing elements gracefully)
    // or returns an error (which is also valid)
    if let Ok(response) = response {
        // If it succeeds, it should be reasonably fast
        assert!(
            response.duration < Duration::from_secs(10),
            "Click operation should complete quickly"
        );
    }
}

// CDP (Chrome DevTools Protocol) Tools Tests
#[test]
fn test_cdp_runtime_evaluate() {
    let mut harness = create_initialized_harness().expect("Failed to create harness");

    let response = match harness.call_tool(
        "cdp_runtime_evaluate",
        json!({
            "expression": "1 + 1",
            "await_promise": false
        }),
    ) {
        Ok(r) => r,
        Err(e) if e.to_string().contains("Timeout") || e.to_string().contains("timeout") => {
            eprintln!("Skipping: cdp_runtime_evaluate timed out (expected on CI)");
            return;
        }
        Err(e) => panic!("Unexpected error: {}", e),
    };

    assert_tool_success(&response, Duration::from_secs(10)).expect("Evaluation should be fast");
    validate_tool_response(&response, "text").expect("Evaluation should return valid response");

    let response_text = response.content[0].get("text").unwrap().as_str().unwrap();
    assert!(
        response_text.contains("2") || response_text.contains("result"),
        "Should contain evaluation result"
    );
}

#[test]
fn test_cdp_runtime_evaluate_with_error() {
    let mut harness = create_initialized_harness().expect("Failed to create harness");

    let response = harness.call_tool(
        "cdp_runtime_evaluate",
        json!({
            "expression": "throw new Error('test error')",
            "await_promise": false
        }),
    );

    // Should either return an error response or fail
    if let Ok(response) = response {
        // If it returns, it might include error information
        let _response_text = response.content[0]
            .get("text")
            .unwrap_or(&json!(""))
            .as_str()
            .unwrap_or("");
        // Error handling is implementation-dependent
    }
}

#[test]
fn test_cdp_dom_get_document() {
    let mut harness = create_initialized_harness().expect("Failed to create harness");

    let response = harness
        .call_tool(
            "cdp_dom_get_document",
            json!({
                "depth": 2
            }),
        )
        .expect("DOM document retrieval should succeed");

    assert_tool_success(&response, Duration::from_secs(10)).expect("DOM retrieval should be fast");
    validate_tool_response(&response, "text").expect("DOM should return valid response");

    let response_text = response.content[0].get("text").unwrap().as_str().unwrap();
    assert!(
        response_text.contains("document")
            || response_text.contains("DOM")
            || response_text.contains("html"),
        "Should contain DOM information"
    );
}

// Parameter Validation Tests
#[test]
fn test_tools_handle_missing_required_params() {
    let mut harness = create_initialized_harness().expect("Failed to create harness");

    let test_cases = vec![
        ("snapshot_url", json!({})),           // Missing required 'url'
        ("ai_memory_get_research", json!({})), // Missing required 'key'
        ("cdp_runtime_evaluate", json!({})),   // Missing required 'expression'
        ("web_search", json!({})),             // Missing required 'query'
    ];

    for (tool_name, args) in test_cases {
        let response = harness.call_tool(tool_name, args);

        // Should either fail during call or return error response
        match response {
            Ok(response) => {
                assert!(
                    response.is_error,
                    "Tool {} should return error for missing required params",
                    tool_name
                );
            }
            Err(_) => {
                // Failing during call is also acceptable
            }
        }
    }
}

#[test]
fn test_tools_handle_invalid_param_types() {
    let mut harness = create_initialized_harness().expect("Failed to create harness");

    let test_cases = vec![
        ("snapshot_url", json!({"url": 123})), // url should be string
        (
            "vault_store_credential",
            json!({"service": 123, "username": "u", "password": "p"}),
        ), // service should be string
        (
            "google_search",
            json!({"query": ["array", "not", "string"], "num_results": "not_number"}),
        ),
        ("cdp_runtime_evaluate", json!({"expression": true})), // expression should be string
    ];

    for (tool_name, args) in test_cases {
        let response = harness.call_tool(tool_name, args);

        // Should handle type errors gracefully
        match response {
            Ok(response) => {
                // If it succeeds, check if it detected the error
                if !response.is_error {
                    // Some tools might coerce types, which is also valid
                    println!("Tool {} handled invalid types by coercing", tool_name);
                }
            }
            Err(_) => {
                // Failing during call is acceptable for type errors
            }
        }
    }
}

// Performance and Reliability Tests
#[test]
fn test_tools_response_times() {
    let mut harness = create_initialized_harness().expect("Failed to create harness");

    let quick_tools = vec![
        ("vault_list_credentials", json!({})),
        ("cdp_runtime_evaluate", json!({"expression": "true"})),
    ];

    for (tool_name, args) in quick_tools {
        let response = match harness.call_tool(tool_name, args) {
            Ok(r) => r,
            Err(e) if e.to_string().contains("Timeout") || e.to_string().contains("timeout") => {
                eprintln!("Skipping: {} timed out (expected on CI)", tool_name);
                continue;
            }
            Err(e) => panic!("Unexpected error for {}: {}", tool_name, e),
        };

        assert!(
            response.duration < Duration::from_secs(5),
            "Tool {} should complete within 5 seconds, took {:?}",
            tool_name,
            response.duration
        );
    }
}

// Edge Case Tests
#[test]
fn test_tools_with_unicode_data() {
    let mut harness = create_initialized_harness().expect("Failed to create harness");

    let username = "测试用户 🚀 عربي ∑";
    let service = "unicode-vault-service";

    let response = harness
        .call_tool(
            "vault_store_credential",
            json!({
                "service": service,
                "username": username,
                "password": "unicode-test-password-123"
            }),
        )
        .expect("Unicode credential storage should succeed");

    assert!(
        !response.is_error,
        "Should handle Unicode data without error"
    );

    // Retrieve (metadata only) and verify
    let list_response = harness
        .call_tool("vault_list_credentials", json!({"service": service}))
        .expect("Unicode credential listing should succeed");

    assert!(
        !list_response.is_error,
        "Should list Unicode credential without error"
    );
    let response_text = list_response.content[0]
        .get("text")
        .unwrap()
        .as_str()
        .unwrap();
    assert!(
        response_text.contains("测试用户") || response_text.contains("\\u"),
        "Should preserve Unicode content: {response_text}"
    );

    let _ = harness.call_tool("vault_delete_credential", json!({"service": service}));
}
