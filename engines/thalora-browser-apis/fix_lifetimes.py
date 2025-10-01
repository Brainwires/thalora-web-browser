#!/usr/bin/env python3
import re
import sys

def fix_file(filepath):
    try:
        with open(filepath, 'r') as f:
            content = f.read()
    except:
        return False
    
    original = content
    changes = 0
    
    # Pattern: let value = if let Some(var) = obj.downcast_ref::<Type>() { var.method() } else { return Err(...) };
    pattern = r'let (\w+) = if let Some\((\w+)\) = (\w+)\.downcast_ref::<([^>]+)>\(\) \{\s*([^\}]+)\s*\} else \{\s*return Err\(JsNativeError::typ\(\)\s*\.with_message\("([^"]+)"\)\s*\.into\(\)\);\s*\};'
    
    matches = list(re.finditer(pattern, content, re.MULTILINE | re.DOTALL))
    for match in reversed(matches):
        result_var = match.group(1)
        var = match.group(2)
        obj = match.group(3)
        typ = match.group(4)
        method_call = match.group(5).strip()
        errmsg = match.group(6)
        
        replacement = f'''let {result_var} = {{
            let {var} = {obj}.downcast_ref::<{typ}>().ok_or_else(|| {{
                JsNativeError::typ()
                    .with_message("{errmsg}")
            }})?;
            {method_call}
        }};'''
        
        content = content[:match.start()] + replacement + content[match.end():]
        changes += 1
    
    if content != original:
        with open(filepath, 'w') as f:
            f.write(content)
        return changes
    return 0

if __name__ == '__main__':
    files = [
        'src/dom/node/node.rs',
        'src/dom/document.rs',
        'src/dom/range.rs',
        'src/dom/element.rs',
        'src/dom/text.rs',
        'src/dom/nodelist/mod.rs',
        'src/dom/document_fragment.rs',
        'src/dom/domtokenlist/mod.rs',
        'src/browser/history.rs',
        'src/browser/window.rs',
        'src/browser/selection.rs',
    ]
    
    total = 0
    for f in files:
        result = fix_file(f)
        if result:
            print(f"Fixed {result} patterns in {f}")
            total += result
    
    print(f"\nTotal fixes: {total}")
