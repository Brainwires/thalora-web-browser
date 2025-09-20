#!/bin/bash
set -e

echo "Debugging Bing Search HTML Structure..."

# Scrape Bing search and save full HTML to analyze selectors
echo '{"jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": {"name": "scrape_url", "arguments": {"url": "https://www.bing.com/search?q=rust+programming&count=3", "wait_for_js": true}}}' | ./target/release/thalora > bing_search_full.json

# Extract just the content part for analysis
cat bing_search_full.json | jq -r '.content[0].content' > bing_search_content.html

echo "Bing search HTML saved to bing_search_content.html"
echo "File size: $(wc -c < bing_search_content.html) bytes"
echo "Number of lines: $(wc -l < bing_search_content.html)"

# Look for common result patterns
echo ""
echo "=== ANALYZING BING RESULT SELECTORS ==="
echo "Looking for .b_algo class:"
grep -o ".b_algo" bing_search_content.html | wc -l

echo "Looking for result-related classes:"
grep -oE 'class="[^"]*[Rr]esult[^"]*"' bing_search_content.html | head -5

echo "Looking for title-related classes:"
grep -oE 'class="[^"]*[Tt]itle[^"]*"' bing_search_content.html | head -5

echo "Looking for h2/h3 tags:"
grep -oE '<h[23][^>]*>' bing_search_content.html | head -5

echo "Looking for search result URLs:"
grep -oE 'href="[^"]*"' bing_search_content.html | head -5

echo ""
echo "=== SAMPLE HTML STRUCTURE ==="
# Show first result-like structure
grep -A 10 -B 2 'class.*result' bing_search_content.html | head -20