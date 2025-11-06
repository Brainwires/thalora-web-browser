#!/usr/bin/env python3
"""
Comprehensive lifetime error fixer.
Handles complex patterns by extracting all necessary values before the downcast_ref borrow.
"""

import re
import sys
from pathlib import Path

def fix_setter_with_context_conversion(content):
    """Fix setters that convert args before using downcast_ref."""
    pattern = re.compile(
        r'([ \t]+)if let Some\((\w+)\) = this_obj\.downcast_ref::<(\w+)>\(\) \{\s*'
        r'let (\w+) = ([^\n]+);?\s*'
        r'([^\}]+?)\s*'
        r'((?:\w+)\.(?:set_\w+|append_\w+|insert_\w+|delete_\w+|replace_\w+)\([^)]+\);?)\s*'
        r'Ok\(JsValue::undefined\(\)\)\s*'
        r'\} else \{\s*'
        r'(Err\([^}]+\))\s*'
        r'\}',
        re.MULTILINE | re.DOTALL
    )

    def replacer(match):
        indent = match.group(1)
        var = match.group(2)
        type_name = match.group(3)
        let_var = match.group(4)
        conversion = match.group(5)
        extra_lines = match.group(6).strip()
        method_call = match.group(7)
        err = match.group(8)

        # Move ALL conversions before the downcast
        return f'''{indent}let {let_var} = {conversion};
{indent}if let Some({var}) = this_obj.downcast_ref::<{type_name}>() {{
{indent}    {extra_lines}
{indent}    {method_call}
{indent}    Ok(JsValue::undefined())
{indent}}} else {{
{indent}    {err}
{indent}}}'''

    return pattern.sub(replacer, content)

def fix_simple_setter(content):
    """Fix simple setters without extra conversions."""
    pattern = re.compile(
        r'([ \t]+)let (\w+) = ([^\n]+\.to_string\(context\)\?);?\s*'
        r'let (\w+) = ([^\n]+);?\s*\n'
        r'([ \t]+)if let Some\((\w+)\) = this_obj\.downcast_ref::<(\w+)>\(\) \{',
        re.MULTILINE
    )

    # This pattern is already good - just ensure it matches
    return content

def fix_type_check_only(content):
    """Fix functions that only check the type but don't use the data."""
    pattern = re.compile(
        r'if let Some\(_(\w+)\) = this_obj\.downcast_ref::<(\w+)>\(\) \{\s*'
        r'Ok\(JsValue::new\((true|false)\)\)\s*'
        r'\}',
        re.MULTILINE
    )

    def replacer(match):
        var = match.group(1)
        type_name = match.group(2)
        value = match.group(3)

        return f'''if this_obj.downcast_ref::<{type_name}>().is_some() {{
            Ok(JsValue::new({value}))
        }}'''

    return pattern.sub(replacer, content)

def fix_nested_if_let(content):
    """Fix patterns with nested if let for Option results."""
    pattern = re.compile(
        r'([ \t]+)if let Some\((\w+)\) = this_obj\.downcast_ref::<(\w+)>\(\) \{\s*'
        r'(?://[^\n]*\n\s*)?'  # Optional comment
        r'if let Some\((\w+)\) = \2\.([^\n]+) \{\s*'
        r'Ok\(\4\.into\(\)\)\s*'
        r'\} else \{\s*'
        r'Ok\(JsValue::null\(\)\)\s*'
        r'\}\s*'
        r'\} else \{\s*'
        r'(Err\([^}]+\))\s*'
        r'\}',
        re.MULTILINE | re.DOTALL
    )

    def replacer(match):
        indent = match.group(1)
        var = match.group(2)
        type_name = match.group(3)
        result_var = match.group(4)
        method_call = match.group(5).strip()
        err = match.group(6)

        return f'''{indent}let opt_value = if let Some({var}) = this_obj.downcast_ref::<{type_name}>() {{
{indent}    {var}.{method_call}
{indent}}} else {{
{indent}    return {err};
{indent}}};
{indent}if let Some({result_var}) = opt_value {{
{indent}    Ok({result_var}.into())
{indent}}} else {{
{indent}    Ok(JsValue::null())
{indent}}}'''

    return pattern.sub(replacer, content)

def fix_match_in_if_let(content):
    """Fix patterns with match statements inside if let."""
    pattern = re.compile(
        r'([ \t]+)if let Some\((\w+)\) = this_obj\.downcast_ref::<(\w+)>\(\) \{\s*'
        r'match \2\.([^\{]+) \{\s*'
        r'([^\}]+)\s*'
        r'\}\s*'
        r'\} else \{\s*'
        r'(Err\([^}]+\))\s*'
        r'\}',
        re.MULTILINE | re.DOTALL
    )

    def replacer(match):
        indent = match.group(1)
        var = match.group(2)
        type_name = match.group(3)
        method_call = match.group(4).strip()
        match_body = match.group(5).strip()
        err = match.group(6)

        return f'''{indent}let value = if let Some({var}) = this_obj.downcast_ref::<{type_name}>() {{
{indent}    {var}.{method_call}
{indent}}} else {{
{indent}    return {err};
{indent}}};
{indent}match value {{
{indent}    {match_body}
{indent}}}'''

    return pattern.sub(replacer, content)

def main():
    src_dir = Path('src')
    fixed_count = 0

    for rs_file in src_dir.rglob('*.rs'):
        try:
            with open(rs_file, 'r', encoding='utf-8') as f:
                content = f.read()

            original = content

            # Apply all fix patterns
            content = fix_setter_with_context_conversion(content)
            content = fix_simple_setter(content)
            content = fix_type_check_only(content)
            content = fix_nested_if_let(content)
            content = fix_match_in_if_let(content)

            if content != original:
                with open(rs_file, 'w', encoding='utf-8') as f:
                    f.write(content)
                fixed_count += 1
                print(f"Fixed: {rs_file}")

        except Exception as e:
            print(f"Error fixing {rs_file}: {e}", file=sys.stderr)

    print(f"\nFixed {fixed_count} files")

if __name__ == '__main__':
    main()
