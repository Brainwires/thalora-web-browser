#!/bin/bash
# Script to extract API modules from Boa engine to thalora-browser-apis crate
# Run this from the thalora-browser-apis directory

set -e

BOA_BUILTINS="../boa/core/engine/src/builtins"
DEST_SRC="./src"

echo "🔍 Extracting Browser APIs from Boa engine..."

# Create directory structure
mkdir -p "$DEST_SRC"/{dom,fetch,storage,worker,file,events,browser,crypto,console,timers}

# DOM APIs
echo "📦 Extracting DOM APIs..."
cp -r "$BOA_BUILTINS/document" "$DEST_SRC/dom/" 2>/dev/null || echo "  ⚠️ document/ not found"
cp -r "$BOA_BUILTINS/element" "$DEST_SRC/dom/" 2>/dev/null || echo "  ⚠️ element/ not found"
cp -r "$BOA_BUILTINS/node" "$DEST_SRC/dom/" 2>/dev/null || echo "  ⚠️ node/ not found"
cp -r "$BOA_BUILTINS/nodelist" "$DEST_SRC/dom/" 2>/dev/null || echo "  ⚠️ nodelist/ not found"
cp -r "$BOA_BUILTINS/domtokenlist" "$DEST_SRC/dom/" 2>/dev/null || echo "  ⚠️ domtokenlist/ not found"
cp -r "$BOA_BUILTINS/attr" "$DEST_SRC/dom/" 2>/dev/null || echo "  ⚠️ attr/ not found"
cp -r "$BOA_BUILTINS/document_fragment" "$DEST_SRC/dom/" 2>/dev/null || echo "  ⚠️ document_fragment/ not found"
cp -r "$BOA_BUILTINS/css" "$DEST_SRC/dom/" 2>/dev/null || echo "  ⚠️ css/ not found"
cp "$BOA_BUILTINS/document.rs" "$DEST_SRC/dom/" 2>/dev/null || echo "  ⚠️ document.rs not found"
cp "$BOA_BUILTINS/element.rs" "$DEST_SRC/dom/" 2>/dev/null || echo "  ⚠️ element.rs not found"
cp "$BOA_BUILTINS/document_parse.rs" "$DEST_SRC/dom/" 2>/dev/null || echo "  ⚠️ document_parse.rs not found"

# Fetch & Networking
echo "📦 Extracting Fetch & Networking APIs..."
cp -r "$BOA_BUILTINS/fetch" "$DEST_SRC/fetch/" 2>/dev/null || echo "  ⚠️ fetch/ not found"
cp -r "$BOA_BUILTINS/websocket" "$DEST_SRC/fetch/" 2>/dev/null || echo "  ⚠️ websocket/ not found"
cp -r "$BOA_BUILTINS/websocket_stream" "$DEST_SRC/fetch/" 2>/dev/null || echo "  ⚠️ websocket_stream/ not found"
cp "$BOA_BUILTINS/fetch.rs" "$DEST_SRC/fetch/" 2>/dev/null || echo "  ⚠️ fetch.rs not found"
cp "$BOA_BUILTINS/websocket.rs" "$DEST_SRC/fetch/" 2>/dev/null || echo "  ⚠️ websocket.rs not found"
cp "$BOA_BUILTINS/websocket_stream.rs" "$DEST_SRC/fetch/" 2>/dev/null || echo "  ⚠️ websocket_stream.rs not found"
cp "$BOA_BUILTINS/event_source.rs" "$DEST_SRC/fetch/" 2>/dev/null || echo "  ⚠️ event_source.rs not found"

# Storage
echo "📦 Extracting Storage APIs..."
cp -r "$BOA_BUILTINS/storage" "$DEST_SRC/storage/" 2>/dev/null || echo "  ⚠️ storage/ not found"
cp -r "$BOA_BUILTINS/storage_event" "$DEST_SRC/storage/" 2>/dev/null || echo "  ⚠️ storage_event/ not found"
cp -r "$BOA_BUILTINS/storage_manager" "$DEST_SRC/storage/" 2>/dev/null || echo "  ⚠️ storage_manager/ not found"
cp -r "$BOA_BUILTINS/cookie_store" "$DEST_SRC/storage/" 2>/dev/null || echo "  ⚠️ cookie_store/ not found"
cp "$BOA_BUILTINS/cookie_store.rs" "$DEST_SRC/storage/" 2>/dev/null || echo "  ⚠️ cookie_store.rs not found"

# Workers
echo "📦 Extracting Worker APIs..."
cp -r "$BOA_BUILTINS/worker" "$DEST_SRC/worker/" 2>/dev/null || echo "  ⚠️ worker/ not found"
cp -r "$BOA_BUILTINS/worker_error" "$DEST_SRC/worker/" 2>/dev/null || echo "  ⚠️ worker_error/ not found"
cp "$BOA_BUILTINS/worker.rs" "$DEST_SRC/worker/" 2>/dev/null || echo "  ⚠️ worker.rs not found"
cp "$BOA_BUILTINS/worker_global_scope.rs" "$DEST_SRC/worker/" 2>/dev/null || echo "  ⚠️ worker_global_scope.rs not found"
cp "$BOA_BUILTINS/worker_navigator.rs" "$DEST_SRC/worker/" 2>/dev/null || echo "  ⚠️ worker_navigator.rs not found"
cp "$BOA_BUILTINS/worker_script_loader.rs" "$DEST_SRC/worker/" 2>/dev/null || echo "  ⚠️ worker_script_loader.rs not found"
cp "$BOA_BUILTINS/worker_events.rs" "$DEST_SRC/worker/" 2>/dev/null || echo "  ⚠️ worker_events.rs not found"
cp "$BOA_BUILTINS/worker_error.rs" "$DEST_SRC/worker/" 2>/dev/null || echo "  ⚠️ worker_error.rs not found"

# Files
echo "📦 Extracting File APIs..."
cp -r "$BOA_BUILTINS/file" "$DEST_SRC/file/" 2>/dev/null || echo "  ⚠️ file/ not found"
cp -r "$BOA_BUILTINS/file_reader" "$DEST_SRC/file/" 2>/dev/null || echo "  ⚠️ file_reader/ not found"
cp -r "$BOA_BUILTINS/file_system" "$DEST_SRC/file/" 2>/dev/null || echo "  ⚠️ file_system/ not found"
cp -r "$BOA_BUILTINS/blob" "$DEST_SRC/file/" 2>/dev/null || echo "  ⚠️ blob/ not found"
cp "$BOA_BUILTINS/file.rs" "$DEST_SRC/file/" 2>/dev/null || echo "  ⚠️ file.rs not found"
cp "$BOA_BUILTINS/file_reader.rs" "$DEST_SRC/file/" 2>/dev/null || echo "  ⚠️ file_reader.rs not found"
cp "$BOA_BUILTINS/blob.rs" "$DEST_SRC/file/" 2>/dev/null || echo "  ⚠️ blob.rs not found"

# Events
echo "📦 Extracting Event APIs..."
cp -r "$BOA_BUILTINS/event" "$DEST_SRC/events/" 2>/dev/null || echo "  ⚠️ event/ not found"
cp -r "$BOA_BUILTINS/event_target" "$DEST_SRC/events/" 2>/dev/null || echo "  ⚠️ event_target/ not found"
cp -r "$BOA_BUILTINS/message_event" "$DEST_SRC/events/" 2>/dev/null || echo "  ⚠️ message_event/ not found"
cp "$BOA_BUILTINS/event.rs" "$DEST_SRC/events/" 2>/dev/null || echo "  ⚠️ event.rs not found"
cp "$BOA_BUILTINS/event_target.rs" "$DEST_SRC/events/" 2>/dev/null || echo "  ⚠️ event_target.rs not found"

# Browser objects
echo "📦 Extracting Browser APIs..."
cp -r "$BOA_BUILTINS/navigator" "$DEST_SRC/browser/" 2>/dev/null || echo "  ⚠️ navigator/ not found"
cp -r "$BOA_BUILTINS/performance" "$DEST_SRC/browser/" 2>/dev/null || echo "  ⚠️ performance/ not found"
cp "$BOA_BUILTINS/window.rs" "$DEST_SRC/browser/" 2>/dev/null || echo "  ⚠️ window.rs not found"
cp "$BOA_BUILTINS/history.rs" "$DEST_SRC/browser/" 2>/dev/null || echo "  ⚠️ history.rs not found"
cp "$BOA_BUILTINS/performance.rs" "$DEST_SRC/browser/" 2>/dev/null || echo "  ⚠️ performance.rs not found"
cp "$BOA_BUILTINS/performance_old.rs" "$DEST_SRC/browser/" 2>/dev/null || echo "  ⚠️ performance_old.rs not found"

# Crypto
echo "📦 Extracting Crypto APIs..."
cp "$BOA_BUILTINS/crypto.rs" "$DEST_SRC/crypto/" 2>/dev/null || echo "  ⚠️ crypto.rs not found"

# Console
echo "📦 Extracting Console API..."
cp -r "$BOA_BUILTINS/console" "$DEST_SRC/console/" 2>/dev/null || echo "  ⚠️ console/ not found"
cp "$BOA_BUILTINS/console.rs" "$DEST_SRC/console/" 2>/dev/null || echo "  ⚠️ console.rs not found"

# Timers
echo "📦 Extracting Timer APIs..."
cp -r "$BOA_BUILTINS/timers" "$DEST_SRC/timers/" 2>/dev/null || echo "  ⚠️ timers/ not found"

echo ""
echo "✅ Extraction complete!"
echo ""
echo "⚠️  IMPORTANT: Next steps:"
echo "1. Update imports in copied files to use 'boa_engine::' prefix"
echo "2. Create module exports in lib.rs"
echo "3. Update Boa to import from thalora-browser-apis"
echo "4. Run tests to ensure everything works"
echo ""
echo "See README.md for full extraction plan"
