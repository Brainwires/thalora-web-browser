#!/usr/bin/env python3
"""
Fix lifetime errors where `this_obj` doesn't live long enough.

The pattern is:
    let this_obj = this.as_object()?;
    if let Some(data) = this_obj.downcast_ref::<SomeData>() {
        Ok(data.some_field().into())  // ERROR: data borrows this_obj
    }

Should be:
    let this_obj = this.as_object()?;
    let value = if let Some(data) = this_obj.downcast_ref::<SomeData>() {
        data.some_field()  // Extract while borrow is active
    } else {
        return Err(...);
    };
    Ok(value.into())  // Use after borrow ends
"""

import re
import sys
from pathlib import Path

def fix_lifetime_in_function(content, func_start, func_end):
    """Fix lifetime errors in a single function."""
    func_content = content[func_start:func_end]

    # Pattern: if let Some(data) = this_obj.downcast_ref::<Type>() {
    #              Ok(data.field().into())
    #          }
    pattern = r'if let Some\((\w+)\) = this_obj\.downcast_ref::<(\w+)>\(\) \{\s*Ok\((\w+)\.([^)]+)\)\.into\(\)\s*\}'

    def replace_pattern(match):
        var_name = match.group(1)
        type_name = match.group(2)
        access_expr = f"{var_name}.{match.group(4)}"

        # Create the fixed version
        return f'''let value = if let Some({var_name}) = this_obj.downcast_ref::<{type_name}>() {{
        {access_expr}
    }} else {{
        return Err(JsNativeError::typ()
            .with_message("called on wrong type")
            .into());
    }};
    Ok(value.into())'''

    fixed = re.sub(pattern, replace_pattern, func_content, flags=re.MULTILINE | re.DOTALL)

    if fixed != func_content:
        return content[:func_start] + fixed + content[func_end:]
    return content

def fix_file(file_path):
    """Fix all lifetime errors in a file."""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()

        original_content = content

        # Find all functions with the pattern
        # Look for: let this_obj = this.as_object()
        #           if let Some(...) = this_obj.downcast_ref
        pattern = r'fn \w+\([^)]+\) -> JsResult<JsValue> \{[^}]*let this_obj = this\.as_object\(\)[^}]*if let Some\([^)]+\) = this_obj\.downcast_ref[^}]*\}'

        matches = list(re.finditer(pattern, content, re.MULTILINE | re.DOTALL))

        # Process in reverse to maintain positions
        for match in reversed(matches):
            content = fix_lifetime_in_function(content, match.start(), match.end())

        if content != original_content:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(content)
            return True
        return False
    except Exception as e:
        print(f"Error processing {file_path}: {e}", file=sys.stderr)
        return False

def main():
    """Fix lifetime errors in all Rust files."""
    src_dir = Path('src')

    fixed_count = 0
    for rs_file in src_dir.rglob('*.rs'):
        if fix_file(rs_file):
            fixed_count += 1
            print(f"Fixed: {rs_file}")

    print(f"\nTotal files fixed: {fixed_count}")

if __name__ == '__main__':
    main()
