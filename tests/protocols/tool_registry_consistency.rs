// Every tool advertised by `tools/list` must have a handler in routing.rs.
//
// Calls each listed tool with empty arguments: parameter-validation errors are
// fine, but "Tool not found" means the tool is advertised without a route.

use super::mcp_harness::{McpTestHarness, create_harness_with_env, create_harness_with_raw_env};
use std::collections::HashMap;

fn env(pairs: &[(&str, &str)]) -> HashMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

fn assert_all_listed_tools_route(harness: &mut McpTestHarness, preset: &str) {
    let tools = harness.list_tools().expect("tools/list should succeed");
    assert!(!tools.is_empty(), "[{preset}] no tools listed");

    for tool in tools {
        let name = tool["name"].as_str().expect("tool has a name").to_string();
        let resp = harness
            .call_tool(&name, serde_json::json!({}))
            .unwrap_or_else(|e| panic!("[{preset}] call to {name} failed at transport level: {e}"));

        assert!(
            !resp.content.is_empty(),
            "[{preset}] {name} returned empty content"
        );
        let text = resp.content[0]["text"].as_str().unwrap_or_default();
        assert!(
            !text.contains("Tool not found"),
            "[{preset}] {name} is listed but has no handler: {text}"
        );
    }
}

#[test]
fn minimal_preset_tools_all_route() {
    let mut harness =
        create_harness_with_raw_env(HashMap::new()).expect("Failed to create harness");
    assert_all_listed_tools_route(&mut harness, "minimal");
}

#[test]
fn full_mode_tools_all_route() {
    // Default harness config: full mode with every category and experimental CDP enabled
    let mut harness = create_harness_with_env(HashMap::new()).expect("Failed to create harness");
    assert_all_listed_tools_route(&mut harness, "full");
}

#[test]
fn brainclaw_preset_tools_all_route() {
    let mut harness = create_harness_with_raw_env(env(&[("THALORA_PRESET", "brainclaw")]))
        .expect("Failed to create harness");
    assert_all_listed_tools_route(&mut harness, "brainclaw");
}

#[test]
fn unimplemented_tools_hidden_by_default() {
    let mut harness = create_harness_with_raw_env(env(&[("THALORA_PRESET", "brainclaw")]))
        .expect("Failed to create harness");
    let names: Vec<String> = harness
        .list_tools()
        .expect("tools/list should succeed")
        .iter()
        .filter_map(|t| t["name"].as_str().map(String::from))
        .collect();

    for hidden in [
        "extract_pdf",
        "download_file",
        "intercept_requests",
        "get_intercepted_requests",
        "page_to_pdf",
        "cdp_dom_get_document",
        "cdp_page_screenshot",
        "browser_screenshot",
    ] {
        assert!(
            !names.iter().any(|n| n == hidden),
            "{hidden} should not be listed by default"
        );
    }
    assert!(names.iter().any(|n| n == "cdp_runtime_evaluate"));
}
