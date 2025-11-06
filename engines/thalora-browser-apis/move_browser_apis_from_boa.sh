#!/bin/bash
# Move all browser/DOM APIs from Boa to thalora-browser-apis
# These were added to Boa but should be in the browser-apis crate

BOA_BUILTINS="../boa/core/engine/src/builtins"
DEST_BASE="src"

echo "Moving browser APIs from Boa to thalora-browser-apis..."

# DOM APIs
echo "Moving DOM APIs..."
cp -n "$BOA_BUILTINS/element.rs" "$DEST_BASE/dom/" 2>/dev/null || echo "element.rs already exists or failed"
cp -n "$BOA_BUILTINS/document.rs" "$DEST_BASE/dom/" 2>/dev/null || echo "document.rs already exists"
cp -n "$BOA_BUILTINS/document_fragment.rs" "$DEST_BASE/dom/" 2>/dev/null || echo "document_fragment.rs already exists"
cp -n "$BOA_BUILTINS/text.rs" "$DEST_BASE/dom/" 2>/dev/null || echo "text.rs already exists"
cp -n "$BOA_BUILTINS/character_data.rs" "$DEST_BASE/dom/" 2>/dev/null || echo "character_data.rs already exists"
cp -n "$BOA_BUILTINS/range.rs" "$DEST_BASE/dom/" 2>/dev/null || echo "range.rs already exists"

# Shadow DOM
echo "Moving Shadow DOM APIs..."
mkdir -p "$DEST_BASE/dom/shadow"
cp -n "$BOA_BUILTINS/shadow_root.rs" "$DEST_BASE/dom/shadow/" 2>/dev/null || echo "shadow_root.rs already exists"
cp -n "$BOA_BUILTINS/shadow_css_scoping.rs" "$DEST_BASE/dom/shadow/" 2>/dev/null || echo "shadow_css_scoping.rs already exists"
cp -n "$BOA_BUILTINS/shadow_tree_traversal.rs" "$DEST_BASE/dom/shadow/" 2>/dev/null || echo "shadow_tree_traversal.rs already exists"
cp -n "$BOA_BUILTINS/html_slot_element.rs" "$DEST_BASE/dom/shadow/" 2>/dev/null || echo "html_slot_element.rs already exists"
cp -n "$BOA_BUILTINS/declarative_shadow_dom.rs" "$DEST_BASE/dom/shadow/" 2>/dev/null || echo "declarative_shadow_dom.rs already exists"
cp -n "$BOA_BUILTINS/slotchange_event.rs" "$DEST_BASE/dom/shadow/" 2>/dev/null || echo "slotchange_event.rs already exists"

# Events (extend existing)
echo "Moving Event APIs..."
cp -n "$BOA_BUILTINS/event.rs" "$DEST_BASE/events/" 2>/dev/null || echo "event.rs already exists"
cp -n "$BOA_BUILTINS/event_target.rs" "$DEST_BASE/events/" 2>/dev/null || echo "event_target.rs already exists"
cp -n "$BOA_BUILTINS/message_event.rs" "$DEST_BASE/events/" 2>/dev/null || echo "message_event.rs already exists"
cp -n "$BOA_BUILTINS/pageswap_event.rs" "$DEST_BASE/events/" 2>/dev/null || echo "pageswap_event.rs already exists"

# File APIs (extend existing)
echo "Moving File APIs..."
cp -n "$BOA_BUILTINS/blob.rs" "$DEST_BASE/file/" 2>/dev/null || echo "blob.rs already exists"
cp -n "$BOA_BUILTINS/file.rs" "$DEST_BASE/file/" 2>/dev/null || echo "file.rs already exists"
cp -n "$BOA_BUILTINS/file_reader.rs" "$DEST_BASE/file/" 2>/dev/null || echo "file_reader.rs already exists"

# Fetch/Network APIs (extend existing)
echo "Moving Fetch/Network APIs..."
cp -n "$BOA_BUILTINS/fetch.rs" "$DEST_BASE/fetch/" 2>/dev/null || echo "fetch.rs already exists"
cp -n "$BOA_BUILTINS/websocket.rs" "$DEST_BASE/fetch/" 2>/dev/null || echo "websocket.rs already exists"
cp -n "$BOA_BUILTINS/websocket_stream.rs" "$DEST_BASE/fetch/" 2>/dev/null || echo "websocket_stream.rs already exists"
cp -n "$BOA_BUILTINS/event_source.rs" "$DEST_BASE/fetch/" 2>/dev/null || echo "event_source.rs already exists"
cp -n "$BOA_BUILTINS/xmlhttprequest.rs" "$DEST_BASE/fetch/" 2>/dev/null || echo "xmlhttprequest.rs already exists"

# Browser APIs (extend existing)
echo "Moving Browser APIs..."
cp -n "$BOA_BUILTINS/window.rs" "$DEST_BASE/browser/" 2>/dev/null || echo "window.rs already exists"
cp -n "$BOA_BUILTINS/history.rs" "$DEST_BASE/browser/" 2>/dev/null || echo "history.rs already exists"
cp -n "$BOA_BUILTINS/performance.rs" "$DEST_BASE/browser/" 2>/dev/null || echo "performance.rs already exists"
cp -n "$BOA_BUILTINS/performance_old.rs" "$DEST_BASE/browser/" 2>/dev/null || echo "performance_old.rs already exists"
cp -n "$BOA_BUILTINS/selection.rs" "$DEST_BASE/browser/" 2>/dev/null || echo "selection.rs already exists"
cp -n "$BOA_BUILTINS/frame_selection.rs" "$DEST_BASE/browser/" 2>/dev/null || echo "frame_selection.rs already exists"

# Console (extend existing)
echo "Moving Console API..."
cp -n "$BOA_BUILTINS/console.rs" "$DEST_BASE/console/" 2>/dev/null || echo "console.rs already exists"

# Crypto (extend existing)
echo "Moving Crypto API..."
cp -n "$BOA_BUILTINS/crypto.rs" "$DEST_BASE/crypto/" 2>/dev/null || echo "crypto.rs already exists"

# Storage (extend existing)
echo "Moving Storage APIs..."
cp -n "$BOA_BUILTINS/cookie_store.rs" "$DEST_BASE/storage/" 2>/dev/null || echo "cookie_store.rs already exists"

# Timers (extend existing)
echo "Moving Timers..."
cp -n "$BOA_BUILTINS/timers.rs" "$DEST_BASE/timers/" 2>/dev/null || echo "timers.rs already exists"

# Worker APIs (extend existing)
echo "Moving Worker APIs..."
cp -n "$BOA_BUILTINS/worker.rs" "$DEST_BASE/worker/" 2>/dev/null || echo "worker.rs already exists"
cp -n "$BOA_BUILTINS/worker_error.rs" "$DEST_BASE/worker/" 2>/dev/null || echo "worker_error.rs already exists"
cp -n "$BOA_BUILTINS/worker_events.rs" "$DEST_BASE/worker/" 2>/dev/null || echo "worker_events.rs already exists"
cp -n "$BOA_BUILTINS/worker_global_scope.rs" "$DEST_BASE/worker/" 2>/dev/null || echo "worker_global_scope.rs already exists"
cp -n "$BOA_BUILTINS/worker_navigator.rs" "$DEST_BASE/worker/" 2>/dev/null || echo "worker_navigator.rs already exists"
cp -n "$BOA_BUILTINS/worker_script_loader.rs" "$DEST_BASE/worker/" 2>/dev/null || echo "worker_script_loader.rs already exists"
cp -n "$BOA_BUILTINS/shared_worker.rs" "$DEST_BASE/worker/" 2>/dev/null || echo "shared_worker.rs already exists"
cp -n "$BOA_BUILTINS/service_worker.rs" "$DEST_BASE/worker/" 2>/dev/null || echo "service_worker.rs already exists"
cp -n "$BOA_BUILTINS/service_worker_container.rs" "$DEST_BASE/worker/" 2>/dev/null || echo "service_worker_container.rs already exists"
cp -n "$BOA_BUILTINS/worklet.rs" "$DEST_BASE/worker/" 2>/dev/null || echo "worklet.rs already exists"

# WebRTC
echo "Moving WebRTC APIs..."
mkdir -p "$DEST_BASE/webrtc"
cp -n "$BOA_BUILTINS/rtc_peer_connection.rs" "$DEST_BASE/webrtc/" 2>/dev/null || echo "rtc_peer_connection.rs already exists"
cp -n "$BOA_BUILTINS/rtc_data_channel.rs" "$DEST_BASE/webrtc/" 2>/dev/null || echo "rtc_data_channel.rs already exists"
cp -n "$BOA_BUILTINS/rtc_ice_candidate.rs" "$DEST_BASE/webrtc/" 2>/dev/null || echo "rtc_ice_candidate.rs already exists"
cp -n "$BOA_BUILTINS/rtc_session_description.rs" "$DEST_BASE/webrtc/" 2>/dev/null || echo "rtc_session_description.rs already exists"
cp -n "$BOA_BUILTINS/webrtc_tests.rs" "$DEST_BASE/webrtc/" 2>/dev/null || echo "webrtc_tests.rs already exists"

# Streams
echo "Moving Streams APIs..."
mkdir -p "$DEST_BASE/streams"
cp -n "$BOA_BUILTINS/readable_stream.rs" "$DEST_BASE/streams/" 2>/dev/null || echo "readable_stream.rs already exists"
cp -n "$BOA_BUILTINS/readable_stream_reader.rs" "$DEST_BASE/streams/" 2>/dev/null || echo "readable_stream_reader.rs already exists"
cp -n "$BOA_BUILTINS/writable_stream.rs" "$DEST_BASE/streams/" 2>/dev/null || echo "writable_stream.rs already exists"
cp -n "$BOA_BUILTINS/transform_stream.rs" "$DEST_BASE/streams/" 2>/dev/null || echo "transform_stream.rs already exists"
cp -n "$BOA_BUILTINS/queuing_strategy.rs" "$DEST_BASE/streams/" 2>/dev/null || echo "queuing_strategy.rs already exists"

# Observers
echo "Moving Observer APIs..."
mkdir -p "$DEST_BASE/observers"
cp -n "$BOA_BUILTINS/mutation_observer.rs" "$DEST_BASE/observers/" 2>/dev/null || echo "mutation_observer.rs already exists"
cp -n "$BOA_BUILTINS/intersection_observer.rs" "$DEST_BASE/observers/" 2>/dev/null || echo "intersection_observer.rs already exists"
cp -n "$BOA_BUILTINS/resize_observer.rs" "$DEST_BASE/observers/" 2>/dev/null || echo "resize_observer.rs already exists"

# Messaging
echo "Moving Messaging APIs..."
mkdir -p "$DEST_BASE/messaging"
cp -n "$BOA_BUILTINS/message_channel.rs" "$DEST_BASE/messaging/" 2>/dev/null || echo "message_channel.rs already exists"
cp -n "$BOA_BUILTINS/message_port.rs" "$DEST_BASE/messaging/" 2>/dev/null || echo "message_port.rs already exists"
cp -n "$BOA_BUILTINS/broadcast_channel.rs" "$DEST_BASE/messaging/" 2>/dev/null || echo "broadcast_channel.rs already exists"

# Miscellaneous
echo "Moving miscellaneous APIs..."
mkdir -p "$DEST_BASE/misc"
cp -n "$BOA_BUILTINS/abort_controller.rs" "$DEST_BASE/misc/" 2>/dev/null || echo "abort_controller.rs already exists"
cp -n "$BOA_BUILTINS/css.rs" "$DEST_BASE/misc/" 2>/dev/null || echo "css.rs already exists"
cp -n "$BOA_BUILTINS/form.rs" "$DEST_BASE/misc/" 2>/dev/null || echo "form.rs already exists"
cp -n "$BOA_BUILTINS/structured_clone.rs" "$DEST_BASE/misc/" 2>/dev/null || echo "structured_clone.rs already exists"

echo "Browser API files copied!"
echo "Note: Many of these may already exist from the previous extraction."
echo "Next steps:"
echo "1. Update module declarations in src/lib.rs"
echo "2. Fix import paths in all copied files"
echo "3. Remove browser APIs from Boa's builtins/mod.rs"
