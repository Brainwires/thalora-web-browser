#!/usr/bin/env python3
"""
Fix all lifetime errors in node.rs
Handles all three patterns systematically
"""

import re

def fix_pattern_1_method_call(content):
    """Fix: if let Some(var) = obj.downcast_ref::<Type>() { method(); Ok(undefined) }"""
    pattern = r'if let Some\((\w+)\) = (\w+)\.downcast_ref::<([^>]+)>\(\) \{\s+' \
              r'\1\.(\w+)\(([^}]*?)\);\s+' \
              r'Ok\(JsValue::undefined\(\)\)\s+' \
              r'\} else \{\s+' \
              r'Err\(JsNativeError::typ\(\)\s+' \
              r'\.with_message\("([^"]+)"\)\s+' \
              r'\.into\(\)\)\s+' \
              r'\}'

    def replace_1(match):
        var_name = match.group(1)
        obj_name = match.group(2)
        type_name = match.group(3)
        method_name = match.group(4)
        method_args = match.group(5).strip()
        error_msg = match.group(6)

        return f'''{{
            let {var_name} = {obj_name}.downcast_ref::<{type_name}>().ok_or_else(|| {{
                JsNativeError::typ()
                    .with_message("{error_msg}")
            }})?;
            {var_name}.{method_name}({method_args});
        }}
        Ok(JsValue::undefined())'''

    return re.sub(pattern, replace_1, content, flags=re.DOTALL)

def fix_pattern_2_value_extraction(content):
    """Fix: let value = if let Some(var) = obj.downcast_ref::<Type>() { method() } else { return Err(...) };"""
    pattern = r'let (\w+) = if let Some\((\w+)\) = (\w+)\.downcast_ref::<([^>]+)>\(\) \{\s+' \
              r'\2\.(\w+)\(([^}]*?)\)\s+' \
              r'\} else \{\s+' \
              r'return Err\(JsNativeError::typ\(\)\s+' \
              r'\.with_message\("([^"]+)"\)\s+' \
              r'\.into\(\)\)\s+' \
              r'\};'

    def replace_2(match):
        result_var = match.group(1)
        var_name = match.group(2)
        obj_name = match.group(3)
        type_name = match.group(4)
        method_name = match.group(5)
        method_args = match.group(6).strip()
        error_msg = match.group(7)

        return f'''let {result_var} = {{
            let {var_name} = {obj_name}.downcast_ref::<{type_name}>().ok_or_else(|| {{
                JsNativeError::typ()
                    .with_message("{error_msg}")
            }})?;
            {var_name}.{method_name}({method_args})
        }};'''

    return re.sub(pattern, replace_2, content, flags=re.DOTALL)

def fix_pattern_3_match(content):
    """Fix: if let Some(var) = obj.downcast_ref::<Type>() { match ... } else { Err(...) }"""
    # Match the full pattern including multi-line match blocks
    pattern = r'if let Some\((\w+)\) = (\w+)\.downcast_ref::<([^>]+)>\(\) \{\s+' \
              r'match \1\.(\w+)\(([^{]*?)\) \{(.*?)^\s+\}\s+' \
              r'\} else \{\s+' \
              r'Err\(JsNativeError::typ\(\)\s+' \
              r'\.with_message\("([^"]+)"\)\s+' \
              r'\.into\(\)\)\s+' \
              r'\}'

    def replace_3(match):
        var_name = match.group(1)
        obj_name = match.group(2)
        type_name = match.group(3)
        method_name = match.group(4)
        method_args = match.group(5).strip()
        match_body = match.group(6)
        error_msg = match.group(7)

        return f'''let {var_name} = {obj_name}.downcast_ref::<{type_name}>().ok_or_else(|| {{
            JsNativeError::typ()
                .with_message("{error_msg}")
        }})?;

        match {var_name}.{method_name}({method_args}) {{{match_body}
        }}'''

    return re.sub(pattern, replace_3, content, flags=re.MULTILINE | re.DOTALL)

def main():
    file_path = "src/dom/node/node.rs"

    print(f"Reading {file_path}...")
    with open(file_path, 'r') as f:
        content = f.read()

    original_len = len(content)

    # Apply fixes in order
    print("Applying pattern 1 (method calls)...")
    content = fix_pattern_1_method_call(content)

    print("Applying pattern 2 (value extraction)...")
    content = fix_pattern_2_value_extraction(content)

    print("Applying pattern 3 (match expressions)...")
    content = fix_pattern_3_match(content)

    if len(content) != original_len:
        print(f"Writing changes to {file_path}...")
        with open(file_path, 'w') as f:
            f.write(content)
        print("✓ Fixed node.rs")
    else:
        print("No changes made")

if __name__ == "__main__":
    main()
