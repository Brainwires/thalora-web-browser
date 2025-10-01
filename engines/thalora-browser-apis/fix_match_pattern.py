#!/usr/bin/env python3
"""
Fix Pattern 3: if let Some(var) = obj.downcast_ref::<Type>() { match ... } else { Err(...) }
This is the most common remaining pattern causing E0597 errors
"""

import re
import glob

def fix_if_let_match_pattern(content):
    """
    Transform:
        if let Some(var) = obj.downcast_ref::<Type>() {
            match var.method() { ... }
        } else {
            Err(...)
        }

    To:
        let var = obj.downcast_ref::<Type>().ok_or_else(|| ...)?;
        match var.method() { ... }
    """

    # Pattern to match if-let with match inside and else with Err
    # This is a multi-line regex that needs to be very careful
    pattern = r'if let Some\((\w+)\) = (\w+)\.downcast_ref::<([^>]+)>\(\) \{\s+' \
              r'match ([^\{]+) \{(.*?)^\s+\}\s+' \
              r'\} else \{\s+' \
              r'Err\((.*?)\)\s+' \
              r'\}'

    def replacer(match):
        var_name = match.group(1)
        obj_name = match.group(2)
        type_name = match.group(3)
        match_expr = match.group(4).strip()
        match_body = match.group(5)
        err_expr = match.group(6).strip()

        # Extract error message from the Err expression
        # Handle both .into() and without
        err_expr_clean = err_expr.replace('.into()', '').strip()

        return f'''let {var_name} = {obj_name}.downcast_ref::<{type_name}>().ok_or_else(|| {{
            {err_expr_clean}
        }})?;

        match {match_expr} {{{match_body}
        }}'''

    return re.sub(pattern, replacer, content, flags=re.MULTILINE | re.DOTALL)

def process_file(filepath):
    """Process a single file"""
    try:
        with open(filepath, 'r') as f:
            content = f.read()

        original = content
        content = fix_if_let_match_pattern(content)

        if content != original:
            with open(filepath, 'w') as f:
                f.write(content)
            return True
        return False
    except Exception as e:
        print(f"Error processing {filepath}: {e}")
        return False

def main():
    # Get all Rust files
    files = glob.glob('src/**/*.rs', recursive=True)

    fixed_count = 0
    for filepath in files:
        if process_file(filepath):
            print(f"✓ Fixed {filepath}")
            fixed_count += 1

    print(f"\nFixed {fixed_count} files")

if __name__ == "__main__":
    main()
