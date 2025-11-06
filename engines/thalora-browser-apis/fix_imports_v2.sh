#!/bin/bash

# Phase 2: Fix builtin module references
# These should reference our local crate modules, not boa_engine::builtins::

echo "Fixing builtin module references..."

find src -name "*.rs" -type f | while read file; do
    # Fix boa_engine::builtins:: to use local crate:: paths
    sed -i 's/boa_engine::builtins::worker_events/crate::worker::worker_events/g' "$file"
    sed -i 's/boa_engine::builtins::event/crate::events::event/g' "$file"
    sed -i 's/boa_engine::builtins::event_target/crate::events::event_target/g' "$file"
    sed -i 's/boa_engine::builtins::message_event/crate::events::message_event/g' "$file"
    sed -i 's/boa_engine::builtins::fetch/crate::fetch/g' "$file"
    sed -i 's/boa_engine::builtins::websocket/crate::fetch::websocket/g' "$file"
    sed -i 's/boa_engine::builtins::storage/crate::storage/g' "$file"
    sed -i 's/boa_engine::builtins::worker/crate::worker/g' "$file"
    sed -i 's/boa_engine::builtins::dom/crate::dom/g' "$file"
    sed -i 's/boa_engine::builtins::file/crate::file/g' "$file"
    sed -i 's/boa_engine::builtins::console/crate::console/g' "$file"
    sed -i 's/boa_engine::builtins::crypto/crate::crypto/g' "$file"
    sed -i 's/boa_engine::builtins::browser/crate::browser/g' "$file"
    sed -i 's/boa_engine::builtins::timers/crate::timers/g' "$file"

    # Also fix any remaining bare crate::builtins references
    sed -i 's/crate::builtins::worker_events/crate::worker::worker_events/g' "$file"
    sed -i 's/crate::builtins::event/crate::events::event/g' "$file"
    sed -i 's/crate::builtins::event_target/crate::events::event_target/g' "$file"
    sed -i 's/crate::builtins::message_event/crate::events::message_event/g' "$file"
    sed -i 's/crate::builtins::fetch/crate::fetch/g' "$file"
    sed -i 's/crate::builtins::websocket/crate::fetch::websocket/g' "$file"
    sed -i 's/crate::builtins::storage/crate::storage/g' "$file"
    sed -i 's/crate::builtins::worker/crate::worker/g' "$file"
    sed -i 's/crate::builtins::dom/crate::dom/g' "$file"
    sed -i 's/crate::builtins::file/crate::file/g' "$file"
    sed -i 's/crate::builtins::console/crate::console/g' "$file"
    sed -i 's/crate::builtins::crypto/crate::crypto/g' "$file"
    sed -i 's/crate::builtins::browser/crate::browser/g' "$file"
    sed -i 's/crate::builtins::timers/crate::timers/g' "$file"
done

echo "Builtin reference fixing complete!"
