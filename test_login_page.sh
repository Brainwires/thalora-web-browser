#!/bin/bash

# Test script specifically for login page DOM traversal
echo "Testing Login Page DOM Traversal..."

{
    # Initialize
    echo '{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2024-11-05", "capabilities": {}}}'

    # Create session
    echo '{"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {"name": "browser_session_management", "arguments": {"action": "create", "session_id": "login_test", "persistent": false}}}'

    # Navigate to login page
    echo '{"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {"name": "scrape_url", "arguments": {"url": "https://dev.brainwires.net/login?redirect=chat", "session_id": "login_test", "wait_for_js": true}}}'

    # Test if we can find form elements in the browser session
    echo '{"jsonrpc": "2.0", "id": 4, "method": "tools/call", "params": {"name": "browser_get_page_content", "arguments": {"session_id": "login_test"}}}'

    # Try to find the email input field using CSS selector (this would require browser_* tools that support CSS selectors)
    # For now, let's use the stateless JavaScript execution to test DOM queries
    echo '{"jsonrpc": "2.0", "id": 5, "method": "tools/call", "params": {"name": "cdp_runtime_evaluate", "arguments": {"expression": "document.querySelectorAll(\"input[type=email]\").length"}}}'

    # Test finding the login form
    echo '{"jsonrpc": "2.0", "id": 6, "method": "tools/call", "params": {"name": "cdp_runtime_evaluate", "arguments": {"expression": "document.querySelectorAll(\"form\").length"}}}'

    # Test finding buttons
    echo '{"jsonrpc": "2.0", "id": 7, "method": "tools/call", "params": {"name": "cdp_runtime_evaluate", "arguments": {"expression": "document.querySelectorAll(\"button\").length"}}}'

    # Test finding Google login button specifically
    echo '{"jsonrpc": "2.0", "id": 8, "method": "tools/call", "params": {"name": "cdp_runtime_evaluate", "arguments": {"expression": "document.querySelectorAll(\"*\").length"}}}'

    # Clean up
    echo '{"jsonrpc": "2.0", "id": 9, "method": "tools/call", "params": {"name": "browser_session_management", "arguments": {"action": "close", "session_id": "login_test"}}}'

} | ./target/release/thalora | jq -c '.content[0] // .content // .result // .' | while read -r response; do
    echo "Response: $response"
done

echo "Login page test completed!"