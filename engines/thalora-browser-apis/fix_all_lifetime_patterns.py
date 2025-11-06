#!/usr/bin/env python3
import re
import glob

def fix_file(filepath):
    try:
        with open(filepath, 'r') as f:
            content = f.read()
    except:
        return 0
    
    original = content
    changes = 0
    
    # Pattern 1: if let Some(var) = obj.downcast_ref::<Type>() { method_call; Ok(undefined) } else { Err(...) }
    p1 = r'if let Some\((\w+)\) = (\w+)\.downcast_ref::<([^>]+)>\(\) \{\s+\1\.(\w+)\(([^;]*)\);\s+Ok\(JsValue::undefined\(\)\)\s+\} else \{\s+Err\(JsNativeError::typ\(\)\s+\.with_message\("([^"]+)"\)\s+\.into\(\)\)\s+\}'
    
    def r1(m):
        var, obj, typ, method, args, errmsg = m.groups()
        return f'''{{
            let {var} = {obj}.downcast_ref::<{typ}>().ok_or_else(|| {{
                JsNativeError::typ()
                    .with_message("{errmsg}")
            }})?;
            {var}.{method}({args});
        }}
        Ok(JsValue::undefined())'''
    
    new_content = re.sub(p1, r1, content, flags=re.MULTILINE)
    if new_content != content:
        changes += content.count('if let Some') - new_content.count('if let Some')
        content = new_content
    
    # Pattern 2: if let Some(var) = obj.downcast_ref::<Type>() { match var.method() { ... } } else { Err(...) }
    p2 = r'if let Some\((\w+)\) = (\w+)\.downcast_ref::<([^>]+)>\(\) \{\s+match \1\.(\w+)\(([^)]*)\) \{\s+Ok\(([^)]*)\) => Ok\(([^,\)]+)\),\s+Err\(err\) => Err\(JsNativeError::range\(\)\s+\.with_message\(err\)\s+\.into\(\)\),\s+\}\s+\} else \{\s+Err\(JsNativeError::typ\(\)\s+\.with_message\("([^"]+)"\)\s+\.into\(\)\)\s+\}'
    
    def r2(m):
        var, obj, typ, method, args, ok_var, ok_result, errmsg = m.groups()
        return f'''let {var} = {obj}.downcast_ref::<{typ}>().ok_or_else(|| {{
            JsNativeError::typ()
                .with_message("{errmsg}")
        }})?;

        match {var}.{method}({args}) {{
            Ok({ok_var}) => Ok({ok_result}),
            Err(err) => Err(JsNativeError::range()
                .with_message(err)
                .into()),
        }}'''
    
    new_content = re.sub(p2, r2, content, flags=re.MULTILINE)
    if new_content != content:
        changes += 1
        content = new_content
    
    if content != original:
        with open(filepath, 'w') as f:
            f.write(content)
        return changes
    return 0

# Fix all files
files = glob.glob('src/**/*.rs', recursive=True)
total = 0
for f in files:
    result = fix_file(f)
    if result:
        print(f"Fixed {result} in {f}")
        total += result

print(f"\nTotal: {total}")
