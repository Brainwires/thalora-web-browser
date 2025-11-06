#!/usr/bin/env python3
"""
Fix ALL remaining lifetime errors in all files
Comprehensive fix for all E0597 patterns
"""

import re
import subprocess
import sys

def get_files_with_lifetime_errors():
    """Get list of files that have E0597 errors"""
    result = subprocess.run(
        ['cargo', 'check', '--message-format=json'],
        capture_output=True,
        text=True
    )

    files = set()
    for line in result.stderr.split('\n'):
        if 'E0597' in line and '-->src' in line or '--> src' in line:
            # Extract file path
            parts = line.split('-->')
            if len(parts) > 1:
                filepath = parts[1].split(':')[0].strip()
                if filepath.startswith('src/'):
                    files.add(filepath)

    return sorted(files)

def fix_pattern_1_method_call(content):
    """Fix: if let Some(var) = obj.downcast_ref::<Type>() { method(); Ok(undefined) }"""
    # More flexible pattern to catch variations
    pattern = r'if let Some\((\w+)\) = (\w+)\.downcast_ref::<([^>]+)>\(\) \{\s+' \
              r'\1\.(\w+)\(([^;]*)\);\s+' \
              r'Ok\(JsValue::undefined\(\)\)\s+' \
              r'\} else \{\s+' \
              r'Err\(JsNativeError::typ\(\)[^\}]*with_message\("([^"]+)"\)[^\}]*\)\s+' \
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
    # Pattern with return Err
    pattern1 = r'let (\w+) = if let Some\((\w+)\) = (\w+)\.downcast_ref::<([^>]+)>\(\) \{\s+' \
               r'\2\.(\w+)\(([^\}]*)\)\s+' \
               r'\} else \{\s+' \
               r'return Err\(JsNativeError::typ\(\)[^\}]*with_message\("([^"]+)"\)[^\}]*\)\s+' \
               r'\};'

    def replace_1(match):
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

    content = re.sub(pattern1, replace_1, content, flags=re.DOTALL)

    # Pattern without return
    pattern2 = r'let (\w+) = if let Some\((\w+)\) = (\w+)\.downcast_ref::<([^>]+)>\(\) \{\s+' \
               r'\2\.(\w+)\(([^\}]*)\)\s+' \
               r'\} else \{\s+' \
               r'Err\(JsNativeError::typ\(\)[^\}]*with_message\("([^"]+)"\)[^\}]*\)\s+' \
               r'\};'

    content = re.sub(pattern2, replace_1, content, flags=re.DOTALL)

    return content

def fix_pattern_3_direct_if_let(content):
    """Fix: if let Some(var) = obj.downcast_ref() { ... } without else"""
    # This is for cases where the borrow is held too long in an if-let
    # We need to be careful not to break working code
    return content

def main():
    print("Finding files with E0597 lifetime errors...")
    files = get_files_with_lifetime_errors()

    if not files:
        print("✓ No lifetime errors found!")
        return 0

    print(f"Found {len(files)} files with lifetime errors")

    for filepath in files:
        print(f"Processing {filepath}...")

        try:
            with open(filepath, 'r') as f:
                content = f.read()

            original_len = len(content)

            # Apply fixes
            content = fix_pattern_1_method_call(content)
            content = fix_pattern_2_value_extraction(content)

            if len(content) != original_len:
                with open(filepath, 'w') as f:
                    f.write(content)
                print(f"  ✓ Fixed {filepath}")
            else:
                print(f"  - No automatic fixes for {filepath}")

        except Exception as e:
            print(f"  ✗ Error processing {filepath}: {e}")

    print("\nRechecking error count...")
    result = subprocess.run(
        ['bash', '-c', 'cargo check 2>&1 | grep "error\\[E0597\\]" | wc -l'],
        capture_output=True,
        text=True
    )
    remaining = result.stdout.strip()
    print(f"Remaining E0597 errors: {remaining}")

    return 0

if __name__ == "__main__":
    sys.exit(main())
