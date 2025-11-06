#!/usr/bin/env python3
"""
Extract inline #[cfg(test)] modules from implementation files and move them to tests/ directory.
"""

import os
import re
from pathlib import Path

def extract_test_module(file_path):
    """Extract inline test module from a file and return (modified_content, test_content)"""
    with open(file_path, 'r') as f:
        content = f.read()

    # Find all #[cfg(test)] mod tests { ... } blocks
    # This regex handles nested braces
    test_pattern = r'(#\[cfg\(test\)\]\s*mod\s+tests\s*\{)'

    matches = list(re.finditer(test_pattern, content))
    if not matches:
        return None, None

    # For each match, find the matching closing brace
    test_blocks = []
    for match in matches:
        start = match.start()
        brace_start = content.index('{', start)

        # Count braces to find matching closing brace
        brace_count = 1
        pos = brace_start + 1
        while pos < len(content) and brace_count > 0:
            if content[pos] == '{':
                brace_count += 1
            elif content[pos] == '}':
                brace_count -= 1
            pos += 1

        if brace_count == 0:
            # Extract the entire test block including #[cfg(test)]
            test_block = content[start:pos]
            test_blocks.append((start, pos, test_block))

    if not test_blocks:
        return None, None

    # Remove test blocks from content (in reverse order to maintain positions)
    modified_content = content
    for start, end, _ in reversed(test_blocks):
        modified_content = modified_content[:start] + modified_content[end:]

    # Clean up extra blank lines
    modified_content = re.sub(r'\n\n\n+', '\n\n', modified_content)
    modified_content = modified_content.rstrip() + '\n'

    # Combine all test blocks
    test_content = '\n\n'.join(block for _, _, block in test_blocks)

    return modified_content, test_content

def create_test_file(original_file, test_content):
    """Create a test file in tests/ directory"""
    # Get relative path from src/
    rel_path = original_file.relative_to('src')

    # Determine test file path
    if rel_path.name == 'mod.rs':
        # For mod.rs files, use parent directory name
        test_path = Path('tests') / rel_path.parent / 'tests.rs'
    else:
        # For regular files, replace .rs with _tests.rs
        test_path = Path('tests') / rel_path.parent / (rel_path.stem + '_tests.rs')

    # Create directory
    test_path.parent.mkdir(parents=True, exist_ok=True)

    # Get module path for imports
    module_parts = list(rel_path.parent.parts) + [rel_path.stem] if rel_path.stem != 'mod' else list(rel_path.parent.parts)
    module_path = '::'.join(module_parts)

    # Create test file content with proper imports
    header = f"""//! Tests for {module_path}

use thalora_browser_apis::boa_engine::{{Context, JsValue, JsResult}};
use thalora_browser_apis::{module_path}::*;

"""

    # Extract the test module content (without the #[cfg(test)] mod tests wrapper)
    # Remove outer wrapper
    test_inner = test_content
    test_inner = re.sub(r'^#\[cfg\(test\)\]\s*mod\s+tests\s*\{\s*', '', test_inner)
    test_inner = re.sub(r'\s*\}\s*$', '', test_inner)

    full_test_content = header + test_inner + '\n'

    # Write test file
    with open(test_path, 'w') as f:
        f.write(full_test_content)

    print(f"Created test file: {test_path}")
    return test_path

def main():
    os.chdir('/home/nightness/dev/brainwires-studio/rust/thalora-web-browser/engines/thalora-browser-apis')

    # Find all .rs files with inline tests
    files_with_tests = []
    for root, dirs, files in os.walk('src'):
        for file in files:
            if file.endswith('.rs'):
                file_path = Path(root) / file
                with open(file_path, 'r') as f:
                    if '#[cfg(test)]' in f.read():
                        files_with_tests.append(file_path)

    print(f"Found {len(files_with_tests)} files with inline tests")

    # Process each file
    for file_path in files_with_tests:
        print(f"\nProcessing: {file_path}")
        modified_content, test_content = extract_test_module(file_path)

        if test_content:
            # Create test file
            create_test_file(file_path, test_content)

            # Update original file
            with open(file_path, 'w') as f:
                f.write(modified_content)
            print(f"Updated: {file_path}")
        else:
            print(f"No test module found (might be mod tests declaration only)")

if __name__ == '__main__':
    main()
