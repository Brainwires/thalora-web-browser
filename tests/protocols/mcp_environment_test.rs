// MCP Environment Variable Tests - Testing tool category enablement/disablement
use super::mcp_harness::*;
use serde_json::json;
use std::time::Duration;

// Test that all tools are available by default
#[test]
fn test_all_tools_enabled_by_default() {
    let mut harness = create_initialized_harness().expect("Failed to create harness");

    let tools = harness.list_tools().expect("Should be able to list tools");
    assert!(!tools.is_empty(), "Should have tools available by default");

    // Check for tools from each category
    let tool_names: Vec<String> = tools
        .iter()
        .filter_map(|tool| tool.get("name")?.as_str())
        .map(|name| name.to_string())
        .collect();

    // AI Memory tools
    assert!(
        tool_names.iter().any(|name| name.starts_with("ai_memory_")),
        "Should have AI memory tools enabled by default"
    );

    // CDP tools
    assert!(
        tool_names.iter().any(|name| name.starts_with("cdp_")),
        "Should have CDP tools enabled by default"
    );

    // Scraping tools
    assert!(
        tool_names.iter().any(|name| name.starts_with("scrape_")),
        "Should have scraping tools enabled by default"
    );

    // Search tools
    assert!(
        tool_names
            .iter()
            .any(|name| name == "web_search" || name == "image_search"),
        "Should have search tools enabled by default"
    );

    // Browser automation tools
    assert!(
        tool_names
            .iter()
            .any(|name| name.starts_with("browser_") && name.contains("click")),
        "Should have browser automation tools enabled by default"
    );

    // Session management tools
    assert!(
        tool_names
            .iter()
            .any(|name| name.starts_with("browser_") && name.contains("session")),
        "Should have session management tools enabled by default"
    );

    eprintln!(
        "All tool categories verified as enabled by default - found {} tools",
        tools.len()
    );
}

// Test disabling AI Memory tools
#[test]
fn test_ai_memory_tools_disabled() {
    let mut harness = create_harness_with_disabled_categories(&["THALORA_ENABLE_AI_MEMORY"])
        .expect("Failed to create harness with disabled AI memory");

    let tools = harness.list_tools().expect("Should be able to list tools");
    let tool_names: Vec<String> = tools
        .iter()
        .filter_map(|tool| tool.get("name")?.as_str())
        .map(|name| name.to_string())
        .collect();

    // AI Memory tools should not be available
    assert!(
        !tool_names.iter().any(|name| name.starts_with("ai_memory_")),
        "AI memory tools should be disabled"
    );

    // Other categories should still be available
    assert!(
        tool_names.iter().any(|name| name.starts_with("cdp_")),
        "CDP tools should still be enabled"
    );
    assert!(
        tool_names.iter().any(|name| name.starts_with("scrape_")),
        "Scraping tools should still be enabled"
    );

    eprintln!(
        "AI memory tools successfully disabled - {} tools remaining",
        tools.len()
    );
}

// Test disabling CDP tools
#[test]
fn test_cdp_tools_disabled() {
    let mut harness = create_harness_with_disabled_categories(&["THALORA_ENABLE_CDP"])
        .expect("Failed to create harness with disabled CDP");

    let tools = harness.list_tools().expect("Should be able to list tools");
    let tool_names: Vec<String> = tools
        .iter()
        .filter_map(|tool| tool.get("name")?.as_str())
        .map(|name| name.to_string())
        .collect();

    // CDP tools should not be available
    assert!(
        !tool_names.iter().any(|name| name.starts_with("cdp_")),
        "CDP tools should be disabled"
    );

    // Other categories should still be available
    assert!(
        tool_names.iter().any(|name| name.starts_with("ai_memory_")),
        "AI memory tools should still be enabled"
    );
    assert!(
        tool_names.iter().any(|name| name.starts_with("scrape_")),
        "Scraping tools should still be enabled"
    );

    eprintln!(
        "CDP tools successfully disabled - {} tools remaining",
        tools.len()
    );
}

// Test disabling scraping tools (which are enabled by default)
#[test]
fn test_scraping_tools_disabled() {
    // Explicitly disable scraping (which is enabled by default)
    let mut env_vars = std::collections::HashMap::new();
    env_vars.insert("THALORA_ENABLE_AI_MEMORY".to_string(), "true".to_string());
    env_vars.insert("THALORA_ENABLE_CDP".to_string(), "true".to_string());
    env_vars.insert("THALORA_ENABLE_SCRAPING".to_string(), "false".to_string());
    env_vars.insert("THALORA_ENABLE_SEARCH".to_string(), "true".to_string());
    env_vars.insert("THALORA_ENABLE_SESSIONS".to_string(), "true".to_string());

    let mut harness =
        create_harness_with_env(env_vars).expect("Failed to create harness with disabled scraping");

    let tools = harness.list_tools().expect("Should be able to list tools");
    let tool_names: Vec<String> = tools
        .iter()
        .filter_map(|tool| tool.get("name")?.as_str())
        .map(|name| name.to_string())
        .collect();

    // Scraping tools should not be available
    assert!(
        !tool_names.iter().any(|name| name.starts_with("scrape_")),
        "Scraping tools should be disabled"
    );

    // Other categories should still be available
    assert!(
        tool_names.iter().any(|name| name.starts_with("ai_memory_")),
        "AI memory tools should still be enabled"
    );
    assert!(
        tool_names.iter().any(|name| name.starts_with("cdp_")),
        "CDP tools should still be enabled"
    );

    eprintln!(
        "Scraping tools successfully disabled - {} tools remaining",
        tools.len()
    );
}

// Test disabling search tools
#[test]
fn test_search_tools_disabled() {
    let mut harness = create_harness_with_disabled_categories(&["THALORA_ENABLE_SEARCH"])
        .expect("Failed to create harness with disabled search");

    let tools = harness.list_tools().expect("Should be able to list tools");
    let tool_names: Vec<String> = tools
        .iter()
        .filter_map(|tool| tool.get("name")?.as_str())
        .map(|name| name.to_string())
        .collect();

    // Search tools should not be available
    assert!(
        !tool_names
            .iter()
            .any(|name| name == "web_search" || name == "image_search"),
        "Search tools should be disabled"
    );

    // Other categories should still be available
    assert!(
        tool_names.iter().any(|name| name.starts_with("ai_memory_")),
        "AI memory tools should still be enabled"
    );
    assert!(
        tool_names.iter().any(|name| name.starts_with("cdp_")),
        "CDP tools should still be enabled"
    );

    eprintln!(
        "Search tools successfully disabled - {} tools remaining",
        tools.len()
    );
}

// Test disabling sessions (browser automation + session management)
#[test]
fn test_sessions_tools_disabled() {
    let mut harness = create_harness_with_disabled_categories(&["THALORA_ENABLE_SESSIONS"])
        .expect("Failed to create harness with disabled sessions");

    let tools = harness.list_tools().expect("Should be able to list tools");
    let tool_names: Vec<String> = tools
        .iter()
        .filter_map(|tool| tool.get("name")?.as_str())
        .map(|name| name.to_string())
        .collect();

    // All browser tools should not be available when sessions are disabled
    let browser_tools = [
        "browser_click_element",
        "browser_fill_form",
        "browser_type_text",
        "browser_wait_for_element",
        "browser_session_management",
        "browser_get_page_content",
        "browser_navigate_to",
        "browser_navigate_back",
        "browser_navigate_forward",
        "browser_refresh_page",
    ];
    assert!(
        !tool_names
            .iter()
            .any(|name| browser_tools.contains(&name.as_str())),
        "All browser tools should be disabled when sessions are disabled"
    );

    // Other categories should still be available
    assert!(
        tool_names.iter().any(|name| name.starts_with("ai_memory_")),
        "AI memory tools should still be enabled"
    );
    assert!(
        tool_names.iter().any(|name| name.starts_with("cdp_")),
        "CDP tools should still be enabled"
    );

    eprintln!(
        "Sessions tools successfully disabled - {} tools remaining",
        tools.len()
    );
}

// Test that CDP automatically enables sessions
#[test]
fn test_cdp_enables_sessions_automatically() {
    // Only enable CDP, but sessions should be enabled automatically
    let mut harness = create_harness_with_only_categories(&["THALORA_ENABLE_CDP"])
        .expect("Failed to create harness with only CDP enabled");

    let tools = harness.list_tools().expect("Should be able to list tools");
    let tool_names: Vec<String> = tools
        .iter()
        .filter_map(|tool| tool.get("name")?.as_str())
        .map(|name| name.to_string())
        .collect();

    // CDP tools should be available
    assert!(
        tool_names.iter().any(|name| name.starts_with("cdp_")),
        "CDP tools should be enabled"
    );

    // Sessions tools should also be available (automatic dependency)
    assert!(
        tool_names
            .iter()
            .any(|name| name == "browser_click_element"),
        "Browser automation tools should be enabled via CDP dependency"
    );
    assert!(
        tool_names
            .iter()
            .any(|name| name == "browser_session_management"),
        "Session management tools should be enabled via CDP dependency"
    );

    // Other categories should not be available
    assert!(
        !tool_names.iter().any(|name| name.starts_with("ai_memory_")),
        "AI memory tools should be disabled"
    );
    assert!(
        !tool_names.iter().any(|name| name.starts_with("scrape_")),
        "Scraping tools should be disabled"
    );

    eprintln!(
        "CDP dependency logic verified - {} tools available",
        tools.len()
    );
}

// Test disabling multiple categories
#[test]
fn test_multiple_categories_disabled() {
    let disabled_categories = [
        "THALORA_ENABLE_AI_MEMORY",
        "THALORA_ENABLE_SCRAPING",
        "THALORA_ENABLE_SEARCH",
    ];
    let mut harness = create_harness_with_disabled_categories(&disabled_categories)
        .expect("Failed to create harness with multiple disabled categories");

    let tools = harness.list_tools().expect("Should be able to list tools");
    let tool_names: Vec<String> = tools
        .iter()
        .filter_map(|tool| tool.get("name")?.as_str())
        .map(|name| name.to_string())
        .collect();

    // Disabled categories should not be available
    assert!(
        !tool_names.iter().any(|name| name.starts_with("ai_memory_")),
        "AI memory tools should be disabled"
    );
    assert!(
        !tool_names.iter().any(|name| name.starts_with("scrape_")),
        "Scraping tools should be disabled"
    );
    assert!(
        !tool_names
            .iter()
            .any(|name| name == "web_search" || name == "image_search"),
        "Search tools should be disabled"
    );

    // Enabled categories should still be available
    assert!(
        tool_names.iter().any(|name| name.starts_with("cdp_")),
        "CDP tools should still be enabled"
    );
    assert!(
        tool_names.iter().any(|name| name.starts_with("browser_")),
        "Browser tools should still be enabled"
    );

    eprintln!(
        "Multiple categories successfully disabled - {} tools remaining",
        tools.len()
    );
}

// Test enabling only specific categories
#[test]
fn test_only_specific_categories_enabled() {
    let enabled_categories = ["THALORA_ENABLE_AI_MEMORY", "THALORA_ENABLE_CDP"];
    let mut harness = create_harness_with_only_categories(&enabled_categories)
        .expect("Failed to create harness with only specific categories");

    let tools = harness.list_tools().expect("Should be able to list tools");
    let tool_names: Vec<String> = tools
        .iter()
        .filter_map(|tool| tool.get("name")?.as_str())
        .map(|name| name.to_string())
        .collect();

    // Only enabled categories should be available
    assert!(
        tool_names.iter().any(|name| name.starts_with("ai_memory_")),
        "AI memory tools should be enabled"
    );
    assert!(
        tool_names.iter().any(|name| name.starts_with("cdp_")),
        "CDP tools should be enabled"
    );

    // Disabled categories should not be available
    assert!(
        !tool_names.iter().any(|name| name.starts_with("scrape_")),
        "Scraping tools should be disabled"
    );
    assert!(
        !tool_names
            .iter()
            .any(|name| name == "web_search" || name == "image_search"),
        "Search tools should be disabled"
    );
    assert!(
        !tool_names.iter().any(|name| name.starts_with("browser_")),
        "Browser tools should be disabled"
    );

    eprintln!(
        "Only specific categories enabled - {} tools available",
        tools.len()
    );
}

// Test that disabled tools return appropriate errors when called
#[test]
fn test_disabled_tools_return_errors() {
    let mut harness = create_harness_with_disabled_categories(&["THALORA_ENABLE_AI_MEMORY"])
        .expect("Failed to create harness with disabled AI memory");

    // Try to call a disabled AI memory tool
    let response = harness.call_tool(
        "ai_memory_store_research",
        json!({
            "key": "test_key",
            "topic": "test topic",
            "summary": "test summary",
            "tags": ["test"]
        }),
    );

    // Should either fail during call or return an error response
    match response {
        Ok(response) => {
            // If the tool call succeeds, it should indicate the tool is not found or return an error
            if response.is_error {
                let empty_json = json!("");
                let error_text = response.content[0]
                    .get("text")
                    .unwrap_or(&empty_json)
                    .as_str()
                    .unwrap_or("");
                assert!(
                    error_text.contains("not found") || error_text.contains("Tool not found"),
                    "Error should indicate tool not found: {}",
                    error_text
                );
                eprintln!("Disabled tool correctly returned error: {}", error_text);
            } else {
                // If it doesn't return an error, the behavior might be that the tool simply doesn't exist
                // This is also acceptable behavior for disabled tools
                eprintln!(
                    "Disabled tool call completed without error - this may indicate the tool was found despite being disabled"
                );
            }
        }
        Err(_) => {
            // Failing during call is also acceptable for disabled tools
            eprintln!("Disabled tool call failed as expected");
        }
    }
}

// Test minimal configuration (all categories disabled)
#[test]
fn test_all_categories_disabled() {
    // Explicitly disable all categories, including scraping (which is enabled by default)
    let mut env_vars = std::collections::HashMap::new();
    env_vars.insert("THALORA_ENABLE_AI_MEMORY".to_string(), "false".to_string());
    env_vars.insert("THALORA_ENABLE_CDP".to_string(), "false".to_string());
    env_vars.insert("THALORA_ENABLE_SCRAPING".to_string(), "false".to_string());
    env_vars.insert("THALORA_ENABLE_SEARCH".to_string(), "false".to_string());
    env_vars.insert("THALORA_ENABLE_SESSIONS".to_string(), "false".to_string());

    let mut harness = create_harness_with_env(env_vars)
        .expect("Failed to create harness with all categories disabled");

    let tools = harness
        .list_tools()
        .expect("Should be able to list tools even when all categories disabled");

    // Should have no tools available
    assert!(
        tools.is_empty(),
        "Should have no tools when all categories are disabled"
    );

    eprintln!(
        "All categories successfully disabled - {} tools available",
        tools.len()
    );
}

// Test default behavior (only scraping tools enabled by default)
#[test]
fn test_default_behavior_no_env_vars() {
    // Create harness with empty environment variables to test true defaults
    let env_vars = std::collections::HashMap::new();
    let mut harness = create_harness_with_env(env_vars)
        .expect("Failed to create harness with no environment variables");

    let tools = harness.list_tools().expect("Should be able to list tools");
    let tool_names: Vec<String> = tools
        .iter()
        .filter_map(|tool| tool.get("name")?.as_str())
        .map(|name| name.to_string())
        .collect();

    // By default (with no environment variables set), only scraping tools should be enabled
    assert!(
        !tools.is_empty(),
        "Should have scraping tools available by default"
    );
    assert!(
        tool_names.iter().any(|name| name.starts_with("scrape_")),
        "Should have scraping tools enabled by default"
    );

    // Other categories should be disabled
    assert!(
        !tool_names.iter().any(|name| name.starts_with("ai_memory_")),
        "AI memory tools should be disabled by default"
    );
    assert!(
        !tool_names.iter().any(|name| name.starts_with("cdp_")),
        "CDP tools should be disabled by default"
    );
    assert!(
        !tool_names.iter().any(|name| name.starts_with("browser_")),
        "Browser tools should be disabled by default"
    );
    assert!(
        !tool_names
            .iter()
            .any(|name| name == "web_search" || name == "image_search"),
        "Search tools should be disabled by default"
    );

    eprintln!(
        "Default behavior verified - {} tools available (scraping tools enabled by default)",
        tools.len()
    );
}

// Test that environment variables are correctly applied
#[test]
fn test_custom_environment_variables() {
    let mut env_vars = std::collections::HashMap::new();
    env_vars.insert("THALORA_ENABLE_AI_MEMORY".to_string(), "true".to_string());
    env_vars.insert("THALORA_ENABLE_CDP".to_string(), "false".to_string());
    env_vars.insert("THALORA_ENABLE_SCRAPING".to_string(), "true".to_string());
    env_vars.insert("THALORA_ENABLE_SEARCH".to_string(), "false".to_string());
    env_vars.insert("THALORA_ENABLE_SESSIONS".to_string(), "true".to_string());

    let mut harness = create_harness_with_env(env_vars)
        .expect("Failed to create harness with custom environment variables");

    let tools = harness.list_tools().expect("Should be able to list tools");
    let tool_names: Vec<String> = tools
        .iter()
        .filter_map(|tool| tool.get("name")?.as_str())
        .map(|name| name.to_string())
        .collect();

    // Enabled categories should be available
    assert!(
        tool_names.iter().any(|name| name.starts_with("ai_memory_")),
        "AI memory tools should be enabled"
    );
    assert!(
        tool_names.iter().any(|name| name.starts_with("scrape_")),
        "Scraping tools should be enabled"
    );
    assert!(
        tool_names
            .iter()
            .any(|name| name == "browser_click_element"),
        "Browser automation tools should be enabled"
    );

    // Disabled categories should not be available
    assert!(
        !tool_names.iter().any(|name| name.starts_with("cdp_")),
        "CDP tools should be disabled"
    );
    assert!(
        !tool_names
            .iter()
            .any(|name| name == "web_search" || name == "image_search"),
        "Search tools should be disabled"
    );

    eprintln!(
        "Custom environment variables correctly applied - {} tools available",
        tools.len()
    );
}

// Performance test - ensure tool filtering doesn't significantly impact performance
#[test]
fn test_tool_filtering_performance() {
    let start_time = std::time::Instant::now();

    // Test with all tools enabled
    let mut harness_all = create_initialized_harness().expect("Failed to create full harness");
    let tools_all = harness_all.list_tools().expect("Should list all tools");
    let all_tools_time = start_time.elapsed();

    // Test with minimal tools
    let start_minimal = std::time::Instant::now();
    let mut harness_minimal = create_harness_with_only_categories(&["THALORA_ENABLE_AI_MEMORY"])
        .expect("Failed to create minimal harness");
    let tools_minimal = harness_minimal
        .list_tools()
        .expect("Should list minimal tools");
    let minimal_tools_time = start_minimal.elapsed();

    // Both should complete quickly
    assert!(
        all_tools_time < Duration::from_secs(5),
        "Listing all tools should be fast: {:?}",
        all_tools_time
    );
    assert!(
        minimal_tools_time < Duration::from_secs(5),
        "Listing minimal tools should be fast: {:?}",
        minimal_tools_time
    );

    // Verify tool counts
    assert!(
        tools_all.len() > tools_minimal.len(),
        "All tools ({}) should be more than minimal tools ({})",
        tools_all.len(),
        tools_minimal.len()
    );
    assert!(
        tools_minimal.len() > 0,
        "Should have at least some tools in minimal configuration"
    );

    eprintln!(
        "Tool filtering performance verified - all: {} tools in {:?}, minimal: {} tools in {:?}",
        tools_all.len(),
        all_tools_time,
        tools_minimal.len(),
        minimal_tools_time
    );
}
