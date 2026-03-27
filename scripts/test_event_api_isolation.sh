#!/bin/bash
cd "$(dirname "$0")/.."

# Test script to isolate which Event API is failing on complex forms
export THALORA_ENABLE_SESSIONS=true

echo "🔍 Testing Event API isolation on complex form page"
echo "Testing which specific API calls are causing 'not a callable function' error"

cat << 'TEST_SCRIPT' | ./target/release/thalora
{"jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": {"name": "browser_session_management", "arguments": {"action": "create", "persistent": false}}}
{"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {"name": "browser_navigate_to", "arguments": {"url": "https://www.twologs.com/en/resources/formtest.asp", "session_id": "EXTRACT_FROM_RESPONSE_1", "wait_for_load": true}}}
TEST_SCRIPT

echo ""
echo "=== Testing individual DOM APIs ==="

# Test 1: typeof Event constructor
echo "Test 1: Testing typeof Event constructor"
cat << 'TEST_SCRIPT' | ./target/release/thalora
{"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {"name": "browser_type_text", "arguments": {"selector": "h1", "text": "dummy", "session_id": "EXTRACT_FROM_RESPONSE_1", "clear_first": true}}}
TEST_SCRIPT

echo ""
echo "Test 2: Testing Event constructor directly"
echo '{"jsonrpc": "2.0", "id": 4, "method": "tools/call", "params": {"name": "browser_type_text", "arguments": {"selector": "h2", "text": "test", "session_id": "EXTRACT_FROM_RESPONSE_1", "clear_first": true}}}' | THALORA_ENABLE_SESSIONS=true ./target/release/thalora

echo ""
echo "Now testing a form input element that we know fails:"

echo '{"jsonrpc": "2.0", "id": 5, "method": "tools/call", "params": {"name": "browser_type_text", "arguments": {"selector": "input[name=\"scriptaddress\"]", "text": "test", "session_id": "EXTRACT_FROM_RESPONSE_1", "clear_first": true}}}' | THALORA_ENABLE_SESSIONS=true ./target/release/thalora