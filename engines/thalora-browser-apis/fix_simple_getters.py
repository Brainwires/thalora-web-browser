#!/usr/bin/env python3
"""
Fix simple getter pattern:
    if let Some(data) = obj.downcast_ref::<Type>() {
        Ok(data.field / data.method())
    } else {
        Err(...)
    }

Transform to:
    let data = obj.downcast_ref::<Type>().ok_or_else(|| ...)?;
    Ok(data.field / data.method())
"""

import re
import sys

def fix_simple_getter_pattern(content):
    """Fix the simple getter if-let pattern"""

    # Pattern: if let Some(var) = obj.downcast_ref::<Type>() {
    #            Ok(var.something)
    #          } else { Err(...) }
    pattern = r'if let Some\((\w+)\) = (\w+)\.downcast_ref::<([^>]+)>\(\) \{\s+' \
              r'Ok\(([^\n]+)\)\s+' \
              r'\} else \{\s+' \
              r'Err\(([^\)]+\))\s*\.into\(\)\)\s+' \
              r'\}'

    def replacer(match):
        var_name = match.group(1)
        obj_name = match.group(2)
        type_name = match.group(3)
        ok_expr = match.group(4).strip()
        err_expr = match.group(5).strip()

        return f'''let {var_name} = {obj_name}.downcast_ref::<{type_name}>().ok_or_else(|| {{
        {err_expr}.into())
    }})?;
    Ok({ok_expr})'''

    return re.sub(pattern, replacer, content, flags=re.DOTALL)

def main():
    if len(sys.argv) < 2:
        print("Usage: fix_simple_getters.py <file>")
        return 1

    filepath = sys.argv[1]

    with open(filepath, 'r') as f:
        content = f.read()

    original = content
    content = fix_simple_getter_pattern(content)

    if content != original:
        with open(filepath, 'w') as f:
            f.write(content)
        print(f"✓ Fixed {filepath}")
        return 0
    else:
        print(f"No changes for {filepath}")
        return 0

if __name__ == "__main__":
    sys.exit(main())
