#!/bin/bash

# MCP Test Script for Thalora
# This script properly tests the MCP server with persistent sessions

echo "Starting Thalora MCP Test Script..."

# Start the MCP server in the background
./target/release/thalora &
SERVER_PID=$!

# Function to send MCP request and get response
send_mcp_request() {
    local request="$1"
    local description="$2"

    echo "=== $description ==="
    echo "Sending: $request"
    echo "$request"
    echo "Response:"
    # Add a small delay to ensure server is ready
    sleep 0.1
}

# Function to cleanup
cleanup() {
    echo "Stopping MCP server (PID: $SERVER_PID)..."
    kill $SERVER_PID 2>/dev/null
    wait $SERVER_PID 2>/dev/null
    echo "Test completed."
}

# Trap to ensure cleanup on exit
trap cleanup EXIT

# Wait for server to start
sleep 1

# Test sequence - all using the same persistent connection
{
    # 1. Initialize MCP protocol
    send_mcp_request '{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2024-11-05", "capabilities": {}}}' "Initialize MCP Protocol"

    # 2. List available tools
    send_mcp_request '{"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}}' "List Available Tools"

    # 3. Create a browser session
    send_mcp_request '{"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {"name": "browser_session_management", "arguments": {"action": "create", "session_id": "login_test", "persistent": false}}}' "Create Browser Session"

    # 4. Navigate to login page using the session
    send_mcp_request '{"jsonrpc": "2.0", "id": 4, "method": "tools/call", "params": {"name": "scrape_url", "arguments": {"url": "https://dev.brainwires.net/login?redirect=chat", "session_id": "login_test", "wait_for_js": true}}}' "Navigate to Login Page"

    # 5. Get page content from the session
    send_mcp_request '{"jsonrpc": "2.0", "id": 5, "method": "tools/call", "params": {"name": "browser_get_page_content", "arguments": {"session_id": "login_test"}}}' "Get Page Content"

    # 6. Test stateless JavaScript execution (this should work independently)
    send_mcp_request '{"jsonrpc": "2.0", "id": 6, "method": "tools/call", "params": {"name": "cdp_runtime_evaluate", "arguments": {"expression": "2 + 2"}}}' "Test Stateless JavaScript"

    # 7. Test DOM sync by setting innerHTML and querying
    send_mcp_request '{"jsonrpc": "2.0", "id": 7, "method": "tools/call", "params": {"name": "cdp_runtime_evaluate", "arguments": {"expression": "document.body.innerHTML = \"<input type=email id=test-email>\"; document.querySelector(\"#test-email\")"}}' "Test DOM Sync"

    # 8. Session info
    send_mcp_request '{"jsonrpc": "2.0", "id": 8, "method": "tools/call", "params": {"name": "browser_session_management", "arguments": {"action": "info", "session_id": "login_test"}}}' "Get Session Info"

    # 9. List all sessions
    send_mcp_request '{"jsonrpc": "2.0", "id": 9, "method": "tools/call", "params": {"name": "browser_session_management", "arguments": {"action": "list"}}}' "List All Sessions"

    # 10. Close the session
    send_mcp_request '{"jsonrpc": "2.0", "id": 10, "method": "tools/call", "params": {"name": "browser_session_management", "arguments": {"action": "close", "session_id": "login_test"}}}' "Close Session"

} | ./target/release/thalora

echo "All tests completed!"