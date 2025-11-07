#!/usr/bin/env python3
import re
import glob

# Pattern to match McpResponse::ToolResult { content: ..., is_error: ... }
# This handles multiline cases
pattern = r'McpResponse::ToolResult\s*\{\s*content:\s*([^}]+?),\s*is_error:\s*([^,}]+?)(?:,)?\s*\}'

def fix_file(filepath):
    with open(filepath, 'r') as f:
        content = f.read()

    # Count occurrences
    matches = list(re.finditer(pattern, content, re.DOTALL))
    if not matches:
        return 0

    print(f"Fixing {len(matches)} occurrences in {filepath}")

    # Replace from end to start to preserve positions
    for match in reversed(matches):
        content_arg = match.group(1).strip()
        is_error_arg = match.group(2).strip()
        replacement = f'McpResponse::tool_result({content_arg}, {is_error_arg})'
        content = content[:match.start()] + replacement + content[match.end():]

    with open(filepath, 'w') as f:
        f.write(content)

    return len(matches)

total = 0
for filepath in glob.glob('src/**/*.rs', recursive=True) + glob.glob('tests/**/*.rs', recursive=True):
    total += fix_file(filepath)

print(f"\nTotal fixes: {total}")
