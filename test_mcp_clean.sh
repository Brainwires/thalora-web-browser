#!/bin/bash

# Test Thalora MCP functionality with clean output by redirecting stderr
echo "🧪 Testing Thalora MCP with clean JSON output"
echo

echo "📋 Test 1: List available tools"
echo '{"jsonrpc": "2.0", "id": 1, "method": "tools/list", "params": {}}' | env THALORA_SILENT=1 ./target/release/thalora 2>/dev/null

echo
echo "🎯 Test 2: Scrape form test page"
echo '{"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {"name": "scrape", "arguments": {"url": "https://www.twologs.com/en/resources/formtest.asp"}}}' | env THALORA_SILENT=1 ./target/release/thalora 2>/dev/null | jq -r '.content[0].title // "No title found"'

echo
echo "🎯 Test 3: Extract form fields using selector"
echo '{"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {"name": "scrape_content_by_selector", "arguments": {"url": "https://www.twologs.com/en/resources/formtest.asp", "selectors": {"form_fields": "input", "form_action": "form"}}}}' | env THALORA_SILENT=1 ./target/release/thalora 2>/dev/null | jq '.content[0].extracted_data // empty'

echo
echo "✅ Clean MCP tests completed"