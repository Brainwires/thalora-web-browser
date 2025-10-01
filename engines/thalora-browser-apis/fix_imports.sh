#!/bin/bash

# Script to fix import paths in extracted browser APIs
# Converts Boa internal imports to use boa_engine:: prefix

echo "Fixing import paths in thalora-browser-apis..."

# Find all .rs files in src/
find src -name "*.rs" -type f | while read file; do
    echo "Processing: $file"

    # Replace crate:: imports with boa_engine:: for Boa types
    # But keep relative module imports (crate::dom, crate::fetch, etc.)
    sed -i 's/use crate::{$/use boa_engine::{/g' "$file"
    sed -i 's/use crate::Context/use boa_engine::Context/g' "$file"
    sed -i 's/use crate::JsResult/use boa_engine::JsResult/g' "$file"
    sed -i 's/use crate::JsValue/use boa_engine::JsValue/g' "$file"
    sed -i 's/use crate::JsObject/use boa_engine::JsObject/g' "$file"
    sed -i 's/use crate::JsNativeError/use boa_engine::JsNativeError/g' "$file"
    sed -i 's/use crate::JsString/use boa_engine::JsString/g' "$file"
    sed -i 's/use crate::js_string/use boa_engine::js_string/g' "$file"
    sed -i 's/use crate::property/use boa_engine::property/g' "$file"
    sed -i 's/use crate::object/use boa_engine::object/g' "$file"
    sed -i 's/use crate::value/use boa_engine::value/g' "$file"
    sed -i 's/use crate::builtins::/use crate::/g' "$file"

    # Handle multi-line use statements
    sed -i '/^use crate::{$/,/^}/ {
        s/Context,/boa_engine::Context,/g
        s/JsResult,/boa_engine::JsResult,/g
        s/JsValue,/boa_engine::JsValue,/g
        s/JsObject,/boa_engine::JsObject,/g
        s/JsNativeError,/boa_engine::JsNativeError,/g
        s/JsString,/boa_engine::JsString,/g
    }' "$file"
done

echo "Import path fixing complete!"
