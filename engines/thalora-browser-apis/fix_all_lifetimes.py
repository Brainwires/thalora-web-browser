#!/usr/bin/env python3
"""
Fix all lifetime errors by extracting values before borrows end.

Transforms:
    if let Some(data) = this_obj.downcast_ref::<Type>() {
        Ok(JsString::from(data.field()).into())
    } else {
        Err(...)
    }

Into:
    let value = if let Some(data) = this_obj.downcast_ref::<Type>() {
        data.field()
    } else {
        return Err(...);
    };
    Ok(JsString::from(value).into())
"""

import re
import sys
from pathlib import Path

def fix_lifetime_pattern(content):
    """Fix the common lifetime pattern."""

    # Pattern 1: Simple Ok(JsString::from(data.field()).into())
    pattern1 = re.compile(
        r'([ \t]+)if let Some\((\w+)\) = this_obj\.downcast_ref::<(\w+)>\(\) \{\s*'
        r'Ok\(JsString::from\(\2\.(\w+)\(\)\)\.into\(\)\)\s*'
        r'\} else \{\s*'
        r'(Err\([^}]+\))\s*'
        r'\}',
        re.MULTILINE | re.DOTALL
    )

    def replace1(match):
        indent = match.group(1)
        var = match.group(2)
        type_name = match.group(3)
        method = match.group(4)
        err = match.group(5)

        return f'''{indent}let value = if let Some({var}) = this_obj.downcast_ref::<{type_name}>() {{
{indent}    {var}.{method}()
{indent}}} else {{
{indent}    return {err};
{indent}}};
{indent}Ok(JsString::from(value).into())'''

    content = pattern1.sub(replace1, content)

    # Pattern 2: Ok(data.field().into())
    pattern2 = re.compile(
        r'([ \t]+)if let Some\((\w+)\) = this_obj\.downcast_ref::<(\w+)>\(\) \{\s*'
        r'Ok\(\2\.(\w+)\(\)\.into\(\)\)\s*'
        r'\} else \{\s*'
        r'(Err\([^}]+\))\s*'
        r'\}',
        re.MULTILINE | re.DOTALL
    )

    def replace2(match):
        indent = match.group(1)
        var = match.group(2)
        type_name = match.group(3)
        method = match.group(4)
        err = match.group(5)

        return f'''{indent}let value = if let Some({var}) = this_obj.downcast_ref::<{type_name}>() {{
{indent}    {var}.{method}()
{indent}}} else {{
{indent}    return {err};
{indent}}};
{indent}Ok(value.into())'''

    content = pattern2.sub(replace2, content)

    # Pattern 3: Ok(JsValue::new(data.field()))
    pattern3 = re.compile(
        r'([ \t]+)if let Some\((\w+)\) = this_obj\.downcast_ref::<(\w+)>\(\) \{\s*'
        r'Ok\(JsValue::new\(\2\.(\w+)\(\)\)\)\s*'
        r'\} else \{\s*'
        r'(Err\([^}]+\))\s*'
        r'\}',
        re.MULTILINE | re.DOTALL
    )

    def replace3(match):
        indent = match.group(1)
        var = match.group(2)
        type_name = match.group(3)
        method = match.group(4)
        err = match.group(5)

        return f'''{indent}let value = if let Some({var}) = this_obj.downcast_ref::<{type_name}>() {{
{indent}    {var}.{method}()
{indent}}} else {{
{indent}    return {err};
{indent}}};
{indent}Ok(JsValue::new(value))'''

    content = pattern3.sub(replace3, content)

    # Pattern 4: Match with Some(obj) return
    pattern4 = re.compile(
        r'([ \t]+)if let Some\((\w+)\) = this_obj\.downcast_ref::<(\w+)>\(\) \{\s*'
        r'match \2\.(\w+)\(\) \{\s*'
        r'Some\((\w+)\) => Ok\(\5\.into\(\)\),\s*'
        r'None => Ok\(JsValue::null\(\)\),?\s*'
        r'\}\s*'
        r'\} else \{\s*'
        r'(Err\([^}]+\))\s*'
        r'\}',
        re.MULTILINE | re.DOTALL
    )

    def replace4(match):
        indent = match.group(1)
        var = match.group(2)
        type_name = match.group(3)
        method = match.group(4)
        result_var = match.group(5)
        err = match.group(6)

        return f'''{indent}let value = if let Some({var}) = this_obj.downcast_ref::<{type_name}>() {{
{indent}    {var}.{method}()
{indent}}} else {{
{indent}    return {err};
{indent}}};
{indent}match value {{
{indent}    Some({result_var}) => Ok({result_var}.into()),
{indent}    None => Ok(JsValue::null()),
{indent}}}'''

    content = pattern4.sub(replace4, content)

    # Pattern 5: Match with Some(val) => JsString::from(val)
    pattern5 = re.compile(
        r'([ \t]+)if let Some\((\w+)\) = this_obj\.downcast_ref::<(\w+)>\(\) \{\s*'
        r'match \2\.(\w+)\(\) \{\s*'
        r'Some\((\w+)\) => Ok\(JsString::from\(\5\)\.into\(\)\),\s*'
        r'None => Ok\(JsValue::null\(\)\),?\s*'
        r'\}\s*'
        r'\} else \{\s*'
        r'(Err\([^}]+\))\s*'
        r'\}',
        re.MULTILINE | re.DOTALL
    )

    def replace5(match):
        indent = match.group(1)
        var = match.group(2)
        type_name = match.group(3)
        method = match.group(4)
        result_var = match.group(5)
        err = match.group(6)

        return f'''{indent}let value = if let Some({var}) = this_obj.downcast_ref::<{type_name}>() {{
{indent}    {var}.{method}()
{indent}}} else {{
{indent}    return {err};
{indent}}};
{indent}match value {{
{indent}    Some({result_var}) => Ok(JsString::from({result_var}).into()),
{indent}    None => Ok(JsValue::null()),
{indent}}}'''

    content = pattern5.sub(replace5, content)

    # Pattern 6: Ok(JsValue::from(js_string!(data.method())))
    pattern6 = re.compile(
        r'([ \t]+)if let Some\((\w+)\) = this_obj\.downcast_ref::<(\w+)>\(\) \{\s*'
        r'Ok\(JsValue::from\(js_string!\(\2\.(\w+)\(\)\)\)\)\s*'
        r'\} else \{\s*'
        r'(Err\([^}]+\))\s*'
        r'\}',
        re.MULTILINE | re.DOTALL
    )

    def replace6(match):
        indent = match.group(1)
        var = match.group(2)
        type_name = match.group(3)
        method = match.group(4)
        err = match.group(5)

        return f'''{indent}let value = if let Some({var}) = this_obj.downcast_ref::<{type_name}>() {{
{indent}    {var}.{method}()
{indent}}} else {{
{indent}    return {err};
{indent}}};
{indent}Ok(JsValue::from(js_string!(value)))'''

    content = pattern6.sub(replace6, content)

    # Pattern 7: Setter with if let Some
    pattern7 = re.compile(
        r'([ \t]+)if let Some\((\w+)\) = this_obj\.downcast_ref::<(\w+)>\(\) \{\s*'
        r'\2\.([^(]+)\(([^)]+)\);?\s*'
        r'Ok\(JsValue::undefined\(\)\)\s*'
        r'\} else \{\s*'
        r'(Err\([^}]+\))\s*'
        r'\}',
        re.MULTILINE | re.DOTALL
    )

    def replace7(match):
        indent = match.group(1)
        var = match.group(2)
        type_name = match.group(3)
        method = match.group(4)
        args = match.group(5)
        err = match.group(6)

        return f'''{indent}if let Some({var}) = this_obj.downcast_ref::<{type_name}>() {{
{indent}    {var}.{method}({args});
{indent}    Ok(JsValue::undefined())
{indent}}} else {{
{indent}    {err}
{indent}}}'''

    content = pattern7.sub(replace7, content)

    # Pattern 8: Ok(JsValue::from(data.method())) without js_string
    pattern8 = re.compile(
        r'([ \t]+)if let Some\((\w+)\) = this_obj\.downcast_ref::<(\w+)>\(\) \{\s*'
        r'Ok\(JsValue::from\(\2\.(\w+)\(\)\)\)\s*'
        r'\} else \{\s*'
        r'(Err\([^}]+\))\s*'
        r'\}',
        re.MULTILINE | re.DOTALL
    )

    def replace8(match):
        indent = match.group(1)
        var = match.group(2)
        type_name = match.group(3)
        method = match.group(4)
        err = match.group(5)

        return f'''{indent}let value = if let Some({var}) = this_obj.downcast_ref::<{type_name}>() {{
{indent}    {var}.{method}()
{indent}}} else {{
{indent}    return {err};
{indent}}};
{indent}Ok(JsValue::from(value))'''

    content = pattern8.sub(replace8, content)

    # Pattern 9: Setter with context conversion - multiline
    pattern9 = re.compile(
        r'([ \t]+)if let Some\((\w+)\) = this_obj\.downcast_ref::<(\w+)>\(\) \{\s*'
        r'let (\w+) = ([^;]+\.to_string\(context\)\?);?\s*'
        r'([^}]+)\s*'
        r'Ok\(JsValue::undefined\(\)\)\s*'
        r'\} else \{\s*'
        r'(Err\([^}]+\))\s*'
        r'\}',
        re.MULTILINE | re.DOTALL
    )

    def replace9(match):
        indent = match.group(1)
        var = match.group(2)
        type_name = match.group(3)
        let_var = match.group(4)
        conversion = match.group(5)
        body = match.group(6).strip()
        err = match.group(7)

        return f'''{indent}if let Some({var}) = this_obj.downcast_ref::<{type_name}>() {{
{indent}    let {let_var} = {conversion};
{indent}    {body}
{indent}    Ok(JsValue::undefined())
{indent}}} else {{
{indent}    {err}
{indent}}}'''

    content = pattern9.sub(replace9, content)

    # Pattern 10: Getter with method call returning Option<JsObject>
    pattern10 = re.compile(
        r'([ \t]+)if let Some\((\w+)\) = this_obj\.downcast_ref::<(\w+)>\(\) \{\s*'
        r'(?://[^\n]*\n\s*)?'  # Optional comment
        r'if let Some\((\w+)\) = \2\.([^{]+) \{\s*'
        r'Ok\(\4\.into\(\)\)\s*'
        r'\} else \{\s*'
        r'Ok\(JsValue::null\(\)\)\s*'
        r'\}\s*'
        r'\} else \{\s*'
        r'(Err\([^}]+\))\s*'
        r'\}',
        re.MULTILINE | re.DOTALL
    )

    def replace10(match):
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

    content = pattern10.sub(replace10, content)

    return content

def fix_file(file_path):
    """Fix all lifetime errors in a file."""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()

        original = content
        content = fix_lifetime_pattern(content)

        if content != original:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(content)
            return True
        return False
    except Exception as e:
        print(f"Error fixing {file_path}: {e}", file=sys.stderr)
        return False

def main():
    src_dir = Path('src')
    fixed_count = 0

    for rs_file in src_dir.rglob('*.rs'):
        if fix_file(rs_file):
            fixed_count += 1
            print(f"Fixed: {rs_file}")

    print(f"\nFixed {fixed_count} files")

if __name__ == '__main__':
    main()
