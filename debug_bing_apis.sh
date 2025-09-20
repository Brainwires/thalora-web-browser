#!/bin/bash
set -e

echo "Debugging missing JavaScript APIs for Bing search..."

# First, let's scrape a Bing search page with JavaScript execution to trigger the debug polyfill
echo "1. Scraping Bing search page to trigger API detection..."
echo '{"jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": {"name": "scrape_url", "arguments": {"url": "https://www.bing.com/search?q=rust+programming&count=3", "wait_for_js": true}}}' | ./target/release/thalora > bing_debug_result.json

echo "2. Extracting debug information..."
cat bing_debug_result.json | jq -r '.content[0].content' > bing_debug_content.html

echo "Content length: $(wc -c < bing_debug_content.html) bytes"

echo ""
echo "=== LOOKING FOR DEBUG OUTPUT ==="

# Look for our debug messages in the HTML content
if grep -q "_BING_DEBUG" bing_debug_content.html; then
    echo "✅ Debug polyfill was loaded"
else
    echo "❌ Debug polyfill was NOT loaded"
fi

# Check if XMLHttpRequest was detected as missing
if grep -q "XMLHttpRequest missing" bing_debug_content.html; then
    echo "✅ XMLHttpRequest was detected as missing"
else
    echo "ℹ️  XMLHttpRequest may already be available"
fi

# Look for any debug console output
echo ""
echo "=== ANALYZING BING SEARCH PAGE STRUCTURE ==="

# Check if search results are present in static HTML
if grep -q "b_algo\|b_title\|result" bing_debug_content.html; then
    echo "✅ Search results found in HTML structure"
    echo "Result count: $(grep -o 'class="b_algo"' bing_debug_content.html | wc -l)"
else
    echo "❌ No search results found in HTML structure"
fi

# Check for JavaScript patterns
echo ""
echo "=== JAVASCRIPT ANALYSIS ==="

if grep -q "fetch(" bing_debug_content.html; then
    echo "✅ fetch() calls found in page JavaScript"
else
    echo "ℹ️  No fetch() calls found"
fi

if grep -q "XMLHttpRequest" bing_debug_content.html; then
    echo "✅ XMLHttpRequest usage found in page JavaScript"
else
    echo "ℹ️  No XMLHttpRequest usage found"
fi

if grep -q "MutationObserver" bing_debug_content.html; then
    echo "✅ MutationObserver usage found in page JavaScript"
else
    echo "ℹ️  No MutationObserver usage found"
fi

# Check for specific Bing JavaScript patterns
echo ""
echo "=== BING-SPECIFIC PATTERNS ==="

if grep -q "BM.trigger\|sj_evt\|_G\.EF" bing_debug_content.html; then
    echo "✅ Bing-specific JavaScript frameworks detected"
else
    echo "ℹ️  No Bing-specific JavaScript frameworks found"
fi

echo ""
echo "Debug files saved:"
echo "- bing_debug_result.json: Full MCP response"
echo "- bing_debug_content.html: Extracted HTML content"
echo ""
echo "Next: Run web_search to see if our debug polyfill helps..."