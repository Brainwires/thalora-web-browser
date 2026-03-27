use anyhow::Result;

pub fn run() -> Result<()> {
    println!("Testing Multi-Window Form Submission Workflow");
    println!();

    let env = &[("THALORA_ENABLE_SESSIONS", "true")];

    println!("1. Navigate to TwoLogs form page with session 'workflow_test'...");
    super::run_mcp_binary(
        r#"{"jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": {"name": "browser_navigate_to", "arguments": {"url": "https://www.twologs.com/en/resources/formtest.asp", "session_id": "workflow_test"}}}
"#,
        env,
    )?;

    println!();
    println!("2. Type test URL into the form field...");
    super::run_mcp_binary(
        r#"{"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {"name": "browser_type_text", "arguments": {"selector": "input[name=\"scriptaddress\"]", "text": "https://httpbin.org/post", "session_id": "workflow_test"}}}
"#,
        env,
    )?;

    println!();
    println!("3. Click the submit button (should create predictive session)...");
    super::run_mcp_binary(
        r#"{"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {"name": "browser_click_element", "arguments": {"selector": "input[name=\"continue\"]", "session_id": "workflow_test"}}}
"#,
        env,
    )?;

    println!();
    println!("Test Complete");
    Ok(())
}
