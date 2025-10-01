#!/bin/bash

# Phase 3: Fix internal crate references
# These should reference local modules (crate::dom::element) or still-in-Boa modules (boa_engine::builtins::form)

echo "Fixing internal crate references..."

find src -name "*.rs" -type f | while read file; do
    # Fix references to modules that are in THIS crate (extracted)
    sed -i 's|use crate::element::|use crate::dom::element::|g' "$file"
    sed -i 's|use crate::document::|use crate::dom::document::|g' "$file"
    sed -i 's|use crate::node::|use crate::dom::node::|g' "$file"
    sed -i 's|use crate::attr::|use crate::dom::attr::|g' "$file"
    sed -i 's|use crate::domtokenlist::|use crate::dom::domtokenlist::|g' "$file"
    sed -i 's|use crate::nodelist::|use crate::dom::nodelist::|g' "$file"
    sed -i 's|use crate::blob::|use crate::file::blob::|g' "$file"
    sed -i 's|use crate::file::|use crate::file::file::|g' "$file"
    sed -i 's|use crate::file_reader::|use crate::file::file_reader::|g' "$file"
    sed -i 's|use crate::event::|use crate::events::event::|g' "$file"
    sed -i 's|use crate::event_target::|use crate::events::event_target::|g' "$file"
    sed -i 's|use crate::fetch::|use crate::fetch::fetch::|g' "$file"
    sed -i 's|use crate::websocket::|use crate::fetch::websocket::|g' "$file"
    sed -i 's|use crate::storage::|use crate::storage::storage::|g' "$file"
    sed -i 's|use crate::console::|use crate::console::console::|g' "$file"
    sed -i 's|use crate::worker::|use crate::worker::worker::|g' "$file"
    sed -i 's|use crate::navigator::|use crate::browser::navigator::|g' "$file"
    sed -i 's|use crate::performance::|use crate::browser::performance::|g' "$file"

    # Fix standalone use statements (not paths)
    sed -i 's|use crate::Array;|use boa_engine::builtins::Array;|g' "$file"
    sed -i 's|use crate::element;|use crate::dom::element;|g' "$file"
    sed -i 's|use crate::blob;|use crate::file::blob;|g' "$file"

    # Fix references to modules still in Boa (not extracted)
    sed -i 's|use crate::form::|use boa_engine::builtins::form::|g' "$file"
    sed -i 's|use crate::array::|use boa_engine::builtins::array::|g' "$file"
    sed -i 's|crate::form::|boa_engine::builtins::form::|g' "$file"

    # Fix trait imports
    sed -i 's|use crate::IntrinsicObject|use boa_engine::builtins::IntrinsicObject|g' "$file"
    sed -i 's|use crate::BuiltInObject|use boa_engine::builtins::BuiltInObject|g' "$file"
    sed -i 's|use crate::BuiltInConstructor|use boa_engine::builtins::BuiltInConstructor|g' "$file"
done

echo "Internal reference fixing complete!"
