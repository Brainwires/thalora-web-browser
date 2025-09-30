#!/bin/bash

# Test script for multi-window form submission workflow
export THALORA_ENABLE_SESSIONS=true

echo "=== Testing Multi-Window Form Submission Workflow ==="
echo ""

echo "1. Navigate to TwoLogs form page with session 'workflow_test'..."
echo '{"jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": {"name": "browser_navigate_to", "arguments": {"url": "https://www.twologs.com/en/resources/formtest.asp", "session_id": "workflow_test"}}}' | ./target/release/thalora | jq '.content[0].success'
echo ""

echo "2. Type test URL into the form field..."
echo '{"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {"name": "browser_type_text", "arguments": {"selector": "input[name=\"scriptaddress\"]", "text": "https://httpbin.org/post", "session_id": "workflow_test"}}}' | ./target/release/thalora | jq '.content[0].success'
echo ""

echo "3. Click the submit button (should create predictive session)..."
CLICK_RESULT=$(echo '{"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {"name": "browser_click_element", "arguments": {"selector": "input[name=\"continue\"]", "session_id": "workflow_test"}}}' | ./target/release/thalora)
echo "$CLICK_RESULT" | jq '.content[0].success'
echo ""

echo "4. Check for potential_new_window info in click response..."
POTENTIAL_NEW_WINDOW=$(echo "$CLICK_RESULT" | jq '.content[0].potential_new_window')
if [ "$POTENTIAL_NEW_WINDOW" != "null" ]; then
    echo "✅ Potential new window detected!"
    echo "$CLICK_RESULT" | jq '.content[0].potential_new_window'

    echo ""
    echo "5. Extract predictive session ID..."
    PREDICTIVE_SESSION=$(echo "$CLICK_RESULT" | jq -r '.content[0].potential_new_window.predictive_session_id // empty')
    if [ -n "$PREDICTIVE_SESSION" ]; then
        echo "✅ Predictive session created: $PREDICTIVE_SESSION"

        echo ""
        echo "6. Validate the predictive session..."
        echo '{"jsonrpc": "2.0", "id": 4, "method": "tools/call", "params": {"name": "browser_validate_session", "arguments": {"session_id": "'$PREDICTIVE_SESSION'"}}}' | ./target/release/thalora | jq '.content[0].session_exists'
    else
        echo "❌ No predictive session ID found in response"
    fi
else
    echo "❌ No potential_new_window info found in click response"
fi

echo ""
echo "=== Test Complete ==="