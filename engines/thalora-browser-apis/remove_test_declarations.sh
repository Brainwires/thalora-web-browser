#!/bin/bash

# Remove #[cfg(test)] mod tests; declarations from source files

echo "Removing test module declarations from source files..."

find src -name "*.rs" -type f | while read file; do
    # Remove #[cfg(test)] mod tests; patterns
    sed -i '/^#\[cfg(test)\]$/,/^mod tests;$/d' "$file"
    # Also remove standalone patterns
    sed -i '/#\[cfg(test)\]\s*$/d' "$file"
    sed -i '/^mod tests;$/d' "$file"
done

echo "Test module declarations removed!"
