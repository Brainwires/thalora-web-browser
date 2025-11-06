#!/usr/bin/env python3
"""
Remove only #[cfg(test)] mod tests; declarations (not other code)
"""

import re
from pathlib import Path

def remove_test_declarations(file_path):
    """Remove #[cfg(test)] mod tests; declarations from a file"""
    with open(file_path, 'r') as f:
        content = f.read()

    original = content

    # Pattern 1: #[cfg(test)] on one line, mod tests; on next line
    content = re.sub(r'#\[cfg\(test\)\]\s*\nmod tests;\s*\n', '', content)

    # Pattern 2: #[cfg(test)] mod tests; on same line
    content = re.sub(r'#\[cfg\(test\)\]\s*mod tests;\s*\n', '', content)

    # Pattern 3: Standalone mod tests; after removing #[cfg(test)]
    # But only if it's truly standalone (not part of pub mod tests)
    content = re.sub(r'^mod tests;\s*\n', '', content, flags=re.MULTILINE)

    if content != original:
        with open(file_path, 'w') as f:
            f.write(content)
        return True
    return False

def main():
    src_dir = Path('src')
    modified_count = 0

    for rs_file in src_dir.rglob('*.rs'):
        if remove_test_declarations(rs_file):
            print(f"Removed test declarations from: {rs_file}")
            modified_count += 1

    print(f"\nModified {modified_count} files")

if __name__ == '__main__':
    main()
