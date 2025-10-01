#!/bin/bash
# Fix all remaining lifetime errors by applying the standard pattern transformations

# Pattern 1: Setters with to_string conversion - add type check first
find src -name "*.rs" -exec perl -i -0pe 's/([ \t]+)let (\w+) = ([^\n]+\.to_string\(context\)\?);?\n([ \t]+)let (\w+) = ([^\n]+);?\n\n([ \t]+)if let Some\((\w+)\) = this_obj\.downcast_ref::<(\w+)>\(\) \{\n([ \t]+)\8\.(\w+)\(\5\);?\n([ \t]+)Ok\(JsValue::undefined\(\)\)\n([ \t]+)\} else \{\n([^\}]+)\n([ \t]+)\}/\1\/\/ Type check first\n\1if this_obj.downcast_ref::<\9>().is_none() {\n\13\n\1}\n\n\1let \2 = \3;\n\4let \5 = \6;\n\n\7\/\/ Set value with fresh borrow\n\7if let Some(\8) = this_obj.downcast_ref::<\9>() {\n\10\8.\11(\5);\n\7}\n\n\12Ok(JsValue::undefined())/gs' {} \;

echo "Fixed setter patterns"

# Run cargo check to see remaining errors
cargo check 2>&1 | grep "^error\[E0597\]" | wc -l
