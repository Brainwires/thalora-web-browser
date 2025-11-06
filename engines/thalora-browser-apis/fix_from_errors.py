#!/usr/bin/env python3
"""
Fix lifetime errors by parsing cargo check output and applying targeted fixes.
"""

import re
import subprocess
from pathlib import Path
from collections import defaultdict

def get_error_locations():
    """Get all E0597 error locations from cargo check."""
    result = subprocess.run(
        ['cargo', 'check'],
        capture_output=True,
        text=True,
        cwd='.'
    )

    errors = []
    pattern = re.compile(r'error\[E0597\]:[^\n]*\n\s+--> ([^:]+):(\d+):(\d+)')

    for match in pattern.finditer(result.stderr):
        file_path = match.group(1)
        line_num = int(match.group(2))
        col_num = int(match.group(3))
        errors.append((file_path, line_num, col_num))

    return errors

def read_function_block(lines, start_line):
    """Read the entire function block starting from a line."""
    # Find the function start
    fn_start = start_line
    while fn_start > 0 and 'fn ' not in lines[fn_start]:
        fn_start -= 1

    # Find the function end (matching braces)
    brace_count = 0
    started = False
    fn_end = fn_start

    for i in range(fn_start, len(lines)):
        for char in lines[i]:
            if char == '{':
                brace_count += 1
                started = True
            elif char == '}':
                brace_count -= 1
                if started and brace_count == 0:
                    fn_end = i
                    return fn_start, fn_end, lines[fn_start:fn_end+1]

    return fn_start, fn_end, lines[fn_start:fn_end+1]

def fix_function(func_lines):
    """Apply smart fixes to a function with lifetime errors."""
    func_text = '\n'.join(func_lines)

    # Pattern 1: Move string conversions before downcast_ref
    # if let Some(x) = this_obj.downcast_ref() {
    #     let y = args.get().to_string(context)?;
    #     x.set(y.to_std_string());
    # }
    #
    # becomes:
    #
    # let y = args.get().to_string(context)?;
    # let y_str = y.to_std_string();
    # if let Some(x) = this_obj.downcast_ref() {
    #     x.set(y_str);
    # }

    pattern1 = re.compile(
        r'(\s+)if let Some\((\w+)\) = this_obj\.downcast_ref::<([^>]+)>\(\) \{\s*\n'
        r'(\s+)let (\w+) = ([^;]+);?\s*\n'
        r'((?:[^\}])*?)'
        r'\2\.(\w+)\([^)]*\2[^)]*\);?\s*\n',
        re.MULTILINE | re.DOTALL
    )

    def replace1(match):
        indent1 = match.group(1)
        var = match.group(2)
        type_name = match.group(3)
        indent2 = match.group(4)
        let_var = match.group(5)
        conversion = match.group(6)
        middle = match.group(7)
        method = match.group(8)

        # Extract the actual value being passed
        # This is a simplified approach - move conversion outside
        return f'''{indent1}let {let_var} = {conversion};
{indent1}// Converted value extracted before borrow
{indent1}if let Some({var}) = this_obj.downcast_ref::<{type_name}>() {{
{middle}{indent2}{var}.{method}(&{let_var}.to_std_string_escaped());
{indent1}}}'''

    func_text = pattern1.sub(replace1, func_text)

    # Pattern 2: Replace if let Some(_unused) with .is_some()
    pattern2 = re.compile(
        r'if let Some\(_\w+\) = this_obj\.downcast_ref::<([^>]+)>\(\) \{',
        re.MULTILINE
    )

    func_text = pattern2.sub(
        r'if this_obj.downcast_ref::<\1>().is_some() {',
        func_text
    )

    return func_text.split('\n')

def fix_file(file_path, error_lines):
    """Fix all lifetime errors in a file."""
    with open(file_path, 'r', encoding='utf-8') as f:
        lines = f.readlines()

    # Strip newlines for processing
    lines = [line.rstrip('\n') for line in lines]

    # Group errors by function
    functions_to_fix = set()
    for line_num in error_lines:
        fn_start, fn_end, func_lines = read_function_block(lines, line_num - 1)
        functions_to_fix.add((fn_start, fn_end))

    # Fix each function
    fixed_lines = lines.copy()
    for fn_start, fn_end in sorted(functions_to_fix, reverse=True):
        func_lines = fixed_lines[fn_start:fn_end+1]
        fixed_func = fix_function(func_lines)
        fixed_lines[fn_start:fn_end+1] = fixed_func

    # Write back
    with open(file_path, 'w', encoding='utf-8') as f:
        f.write('\n'.join(fixed_lines) + '\n')

    return len(functions_to_fix)

def main():
    print("Getting error locations from cargo check...")
    errors = get_error_locations()

    if not errors:
        print("No E0597 errors found!")
        return

    print(f"Found {len(errors)} lifetime errors")

    # Group by file
    errors_by_file = defaultdict(list)
    for file_path, line_num, col_num in errors:
        errors_by_file[file_path].append(line_num)

    print(f"Errors in {len(errors_by_file)} files")

    total_fixed = 0
    for file_path, error_lines in errors_by_file.items():
        try:
            fixed_count = fix_file(file_path, error_lines)
            total_fixed += fixed_count
            print(f"Fixed {fixed_count} functions in {file_path}")
        except Exception as e:
            print(f"Error fixing {file_path}: {e}")

    print(f"\nTotal functions fixed: {total_fixed}")
    print("Run 'cargo check' again to verify fixes")

if __name__ == '__main__':
    main()
