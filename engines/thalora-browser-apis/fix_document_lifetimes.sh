#!/bin/bash
# Fix specific lifetime patterns in document.rs

file="src/dom/document.rs"

# Create backup
cp "$file" "$file.bak"

# Use sed to do in-place replacements for the common pattern
# Pattern: if let Some(document) = this_obj.downcast_ref::<DocumentData>() { ... } else { Err(...) }

# This is complex, let's just identify the lines and note them for manual fixing
echo "Lines with lifetime errors in document.rs:"
echo "354, 371, 396, 432, 643, 666, 872"
echo "All follow the same pattern - need manual fixing"
