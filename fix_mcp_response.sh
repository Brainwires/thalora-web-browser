#!/bin/bash

# Fix all McpResponse::ToolResult { content: ..., is_error: ... } to McpResponse::tool_result(..., ...)

find src tests -name "*.rs" -type f | while read file; do
    # Use perl for multiline regex replacement
    perl -i -0pe 's/McpResponse::ToolResult\s*\{\s*content:\s*([^}]+?),\s*is_error:\s*([^}]+?),?\s*\}/McpResponse::tool_result($1, $2)/gs' "$file"
done

echo "Fixed all McpResponse::ToolResult usages"
