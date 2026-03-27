use anyhow::Result;

pub fn run() -> Result<()> {
    println!("Testing Event API isolation on complex form page");

    let env = &[("THALORA_ENABLE_SESSIONS", "true")];

    // Step 1: create session and navigate
    println!("Step 1: Creating session and navigating to form...");
    super::run_mcp_binary(
        concat!(
            r#"{"jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": {"name": "browser_session_management", "arguments": {"action": "create", "persistent": false}}}"#, "\n",
            r#"{"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {"name": "browser_navigate_to", "arguments": {"url": "https://www.twologs.com/en/resources/formtest.asp", "session_id": "EXTRACT_FROM_RESPONSE_1", "wait_for_load": true}}}"#, "\n",
        ),
        env,
    )?;

    println!();
    println!("Testing individual DOM APIs");

    println!("Test 1: Testing typeof Event constructor");
    super::run_mcp_binary(
        r#"{"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {"name": "browser_type_text", "arguments": {"selector": "h1", "text": "dummy", "session_id": "EXTRACT_FROM_RESPONSE_1", "clear_first": true}}}
"#,
        env,
    )?;

    println!();
    println!("Test 2: Testing Event constructor directly");
    super::run_mcp_binary(
        r#"{"jsonrpc": "2.0", "id": 4, "method": "tools/call", "params": {"name": "browser_type_text", "arguments": {"selector": "h2", "text": "test", "session_id": "EXTRACT_FROM_RESPONSE_1", "clear_first": true}}}
"#,
        env,
    )?;

    println!();
    println!("Testing known-failing form input element:");
    super::run_mcp_binary(
        r#"{"jsonrpc": "2.0", "id": 5, "method": "tools/call", "params": {"name": "browser_type_text", "arguments": {"selector": "input[name=\"scriptaddress\"]", "text": "test", "session_id": "EXTRACT_FROM_RESPONSE_1", "clear_first": true}}}
"#,
        env,
    )
}
