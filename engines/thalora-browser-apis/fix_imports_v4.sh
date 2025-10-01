#!/bin/bash

# Phase 4: Fix double module references created by v3

echo "Fixing double module references..."

find src -name "*.rs" -type f | while read file; do
    # Fix doubled module names
    sed -i 's|crate::file::file::|crate::file::|g' "$file"
    sed -i 's|crate::dom::element::element|crate::dom::element|g' "$file"
    sed -i 's|crate::dom::document::document|crate::dom::document|g' "$file"
    sed -i 's|crate::dom::node::node|crate::dom::node|g' "$file"
    sed -i 's|crate::events::event::event|crate::events::event|g' "$file"
    sed -i 's|crate::events::event_target::event_target|crate::events::event_target|g' "$file"
    sed -i 's|crate::fetch::fetch::fetch|crate::fetch::fetch|g' "$file"
    sed -i 's|crate::fetch::websocket::websocket|crate::fetch::websocket|g' "$file"
    sed -i 's|crate::storage::storage::storage|crate::storage::storage|g' "$file"
    sed -i 's|crate::console::console::console|crate::console::console|g' "$file"
    sed -i 's|crate::worker::worker::worker|crate::worker::worker|g' "$file"
    sed -i 's|crate::browser::navigator::navigator|crate::browser::navigator|g' "$file"
    sed -i 's|crate::browser::performance::performance|crate::browser::performance|g' "$file"

    # Also fix in use statements
    sed -i 's|use crate::builtins::element|use crate::dom::element|g' "$file"
done

echo "Double reference fixing complete!"
