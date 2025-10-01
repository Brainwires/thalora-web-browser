import re

with open('src/dom/character_data.rs', 'r') as f:
    content = f.read()

# Pattern: if let Some(char_data) = this_obj.downcast_ref::<CharacterDataData>() { method_call; Ok(...) } else { Err(...) }
pattern = r'if let Some\(char_data\) = this_obj\.downcast_ref::<CharacterDataData>\(\) \{\s+char_data\.(\w+)\(([^)]*)\);\s+Ok\(JsValue::undefined\(\)\)\s+\} else \{\s+Err\(JsNativeError::typ\(\)\s+\.with_message\("([^"]+)"\)\s+\.into\(\)\)\s+\}'

def replace(m):
    method = m.group(1)
    args = m.group(2)
    errmsg = m.group(3)
    return f'''{{
            let char_data = this_obj.downcast_ref::<CharacterDataData>().ok_or_else(|| {{
                JsNativeError::typ()
                    .with_message("{errmsg}")
            }})?;
            char_data.{method}({args});
        }}
        Ok(JsValue::undefined())'''

content = re.sub(pattern, replace, content, flags=re.MULTILINE)

with open('src/dom/character_data.rs', 'w') as f:
    f.write(content)

print("Fixed remaining character_data.rs patterns")
