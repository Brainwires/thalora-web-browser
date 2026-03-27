use anyhow::Result;

pub fn run() -> Result<()> {
    println!("Testing Thalora MCP with clean JSON output");
    println!();

    let env = &[("THALORA_SILENT", "1")];

    println!("Test 1: List available tools");
    super::run_mcp_binary(
        r#"{"jsonrpc": "2.0", "id": 1, "method": "tools/list", "params": {}}
"#,
        env,
    )?;

    println!();
    println!("Test 2: Scrape form test page");
    super::run_mcp_binary(
        r#"{"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {"name": "snapshot_url", "arguments": {"url": "https://www.twologs.com/en/resources/formtest.asp"}}}
"#,
        env,
    )?;

    println!();
    println!("Test 3: Extract form fields using selector");
    super::run_mcp_binary(
        r#"{"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {"name": "scrape_content_by_selector", "arguments": {"url": "https://www.twologs.com/en/resources/formtest.asp", "selectors": {"form_fields": "input", "form_action": "form"}}}}
"#,
        env,
    )?;

    println!();
    println!("Clean MCP tests completed");
    Ok(())
}
