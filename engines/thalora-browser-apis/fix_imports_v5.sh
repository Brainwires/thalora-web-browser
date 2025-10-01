#!/bin/bash

# Phase 5: Fix remaining crate:: references for traits and un-extracted modules

echo "Fixing remaining crate:: references..."

find src -name "*.rs" -type f | while read file; do
    # Fix trait imports that still use crate::
    sed -i 's|crate::BuiltInConstructor|boa_engine::builtins::BuiltInConstructor|g' "$file"
    sed -i 's|crate::BuiltInObject|boa_engine::builtins::BuiltInObject|g' "$file"
    sed -i 's|crate::IntrinsicObject|boa_engine::builtins::IntrinsicObject|g' "$file"
    sed -i 's|crate::BuiltInBuilder|boa_engine::builtins::BuiltInBuilder|g' "$file"

    # Fix references to modules still in Boa (not extracted)
    sed -i 's|crate::web_locks|boa_engine::builtins::web_locks|g' "$file"
    sed -i 's|crate::service_worker_container|boa_engine::builtins::service_worker_container|g' "$file"
    sed -i 's|crate::storage_manager|crate::storage::storage_manager|g' "$file"
    sed -i 's|crate::indexed_db|boa_engine::builtins::indexed_db|g' "$file"
    sed -i 's|crate::selection|boa_engine::builtins::selection|g' "$file"
    sed -i 's|crate::form|boa_engine::builtins::form|g' "$file"

    # Fix FileData reference
    sed -i 's|crate::file::FileData|crate::file::file_system::FileData|g' "$file"

    # Fix JsNativeError references that might still be wrong
    sed -i 's|crate root::JsNativeError|boa_engine::JsNativeError|g' "$file"
    sed -i 's|crate::JsNativeError|boa_engine::JsNativeError|g' "$file"

    # Fix JsPromise (might be private)
    sed -i 's|crate::JsPromise|boa_engine::object::builtins::JsPromise|g' "$file"
done

echo "Remaining crate:: reference fixing complete!"
