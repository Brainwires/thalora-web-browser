use anyhow::Result;

pub fn run() -> Result<()> {
    println!("COMPLETE FORM AUTOMATION Demo");
    println!("==================================");
    println!("Demonstrating full browser automation with proper session persistence");
    println!();

    let env = &[("THALORA_ENABLE_SESSIONS", "true")];

    let input = concat!(
        r#"{"jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": {"name": "browser_session_management", "arguments": {"action": "create", "persistent": false}}}"#, "\n",
        r#"{"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {"name": "browser_navigate_to", "arguments": {"url": "https://www.twologs.com/en/resources/formtest.asp", "session_id": "EXTRACT_FROM_RESPONSE_1", "wait_for_load": true}}}"#, "\n",
        r#"{"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {"name": "browser_type_text", "arguments": {"selector": "input[name=\"scriptaddress\"]", "text": "https://httpbin.org/post", "session_id": "EXTRACT_FROM_RESPONSE_1", "clear_first": true}}}"#, "\n",
        r#"{"jsonrpc": "2.0", "id": 4, "method": "tools/call", "params": {"name": "browser_click_element", "arguments": {"selector": "input[name=\"replacetext\"]", "session_id": "EXTRACT_FROM_RESPONSE_1"}}}"#, "\n",
        r#"{"jsonrpc": "2.0", "id": 5, "method": "tools/call", "params": {"name": "browser_get_page_content", "arguments": {"session_id": "EXTRACT_FROM_RESPONSE_1", "include_html": false}}}"#, "\n",
        r#"{"jsonrpc": "2.0", "id": 6, "method": "tools/call", "params": {"name": "browser_session_management", "arguments": {"action": "info", "session_id": "EXTRACT_FROM_RESPONSE_1"}}}"#, "\n",
        r#"{"jsonrpc": "2.0", "id": 7, "method": "tools/call", "params": {"name": "browser_click_element", "arguments": {"selector": "input[type=\"submit\"]", "session_id": "EXTRACT_FROM_RESPONSE_1", "wait_for_navigation": true}}}"#, "\n",
        r#"{"jsonrpc": "2.0", "id": 8, "method": "tools/call", "params": {"name": "browser_session_management", "arguments": {"action": "close", "session_id": "EXTRACT_FROM_RESPONSE_1"}}}"#, "\n",
    );

    super::run_mcp_binary(input, env)
}
