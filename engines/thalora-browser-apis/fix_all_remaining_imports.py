#!/usr/bin/env python3
"""
Comprehensive import fixer - handles all remaining import issues
"""

import re
import os
from pathlib import Path

def fix_file_imports(file_path):
    """Fix all import issues in a single file"""
    with open(file_path, 'r') as f:
        content = f.read()

    original = content

    # Fix any remaining crate::BuiltIn* references in use statements
    content = re.sub(r'use crate::\{([^}]*BuiltInConstructor[^}]*)\}',
                     r'use boa_engine::builtins::{\1}', content)
    content = re.sub(r'use crate::\{([^}]*BuiltInObject[^}]*)\}',
                     r'use boa_engine::builtins::{\1}', content)
    content = re.sub(r'use crate::\{([^}]*IntrinsicObject[^}]*)\}',
                     r'use boa_engine::builtins::{\1}', content)

    # Fix individual trait imports
    content = re.sub(r'\bcrate::BuiltInConstructor\b', 'boa_engine::builtins::BuiltInConstructor', content)
    content = re.sub(r'\bcrate::BuiltInObject\b', 'boa_engine::builtins::BuiltInObject', content)
    content = re.sub(r'\bcrate::IntrinsicObject\b', 'boa_engine::builtins::IntrinsicObject', content)

    # Fix message_event references
    content = re.sub(r'use super::message_event', 'use crate::events::message_event', content)
    content = re.sub(r'\bsuper::message_event::', 'crate::events::message_event::', content)

    # Fix file_system function references (they're in the file_system module of our crate)
    content = re.sub(r'crate::file_system::(show_\w+_file_picker)',
                     r'crate::file::file_system::\1', content)

    # Fix StandardConstructor imports
    content = re.sub(r'use crate::context::intrinsics::StandardConstructor',
                     'use boa_engine::context::intrinsics::StandardConstructor', content)

    if content != original:
        with open(file_path, 'w') as f:
            f.write(content)
        return True
    return False

def main():
    os.chdir('/home/nightness/dev/brainwires-studio/rust/thalora-web-browser/engines/thalora-browser-apis')

    modified = 0
    for rs_file in Path('src').rglob('*.rs'):
        if fix_file_imports(rs_file):
            print(f"Fixed: {rs_file}")
            modified += 1

    print(f"\nModified {modified} files")

if __name__ == '__main__':
    main()
