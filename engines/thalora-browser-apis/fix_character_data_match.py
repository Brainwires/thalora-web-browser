import re

with open('src/dom/character_data.rs', 'r') as f:
    content = f.read()

# Pattern: if let Some(char_data) = ... { match char_data.method(...) { Ok(...) => ..., Err(...) => ... } } else { Err(...) }
pattern = r'if let Some\(char_data\) = this_obj\.downcast_ref::<CharacterDataData>\(\) \{\s+match char_data\.(\w+)\(([^)]*)\) \{\s+Ok\(([^)]*)\) => Ok\(([^)]+)\),\s+Err\(err\) => Err\(JsNativeError::range\(\)\s+\.with_message\(err\)\s+\.into\(\)\),\s+\}\s+\} else \{\s+Err\(JsNativeError::typ\(\)\s+\.with_message\("([^"]+)"\)\s+\.into\(\)\)\s+\}'

def replace(m):
    method = m.group(1)
    args = m.group(2)
    ok_var = m.group(3)
    ok_result = m.group(4)
    errmsg = m.group(5)
    
    return f'''let char_data = this_obj.downcast_ref::<CharacterDataData>().ok_or_else(|| {{
            JsNativeError::typ()
                .with_message("{errmsg}")
        }})?;

        match char_data.{method}({args}) {{
            Ok({ok_var}) => Ok({ok_result}),
            Err(err) => Err(JsNativeError::range()
                .with_message(err)
                .into()),
        }}'''

content = re.sub(pattern, replace, content, flags=re.MULTILINE)

with open('src/dom/character_data.rs', 'w') as f:
    f.write(content)

print("Fixed match patterns in character_data.rs")
