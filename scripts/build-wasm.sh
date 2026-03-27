#!/bin/bash
set -e
cd "$(dirname "$0")/.."

echo "Building thalora-web-browser for WebAssembly..."

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "wasm-pack not found. Installing..."
    cargo install wasm-pack
fi

# Build for different targets with wasm feature enabled
echo ""
echo "Building for bundler (webpack, rollup, vite, etc.)..."
wasm-pack build --target bundler --out-dir pkg/bundler --features wasm --no-default-features

echo ""
echo "Building for Node.js..."
wasm-pack build --target nodejs --out-dir pkg/nodejs --features wasm --no-default-features

echo ""
echo "Building for web (ES modules)..."
wasm-pack build --target web --out-dir pkg/web --features wasm --no-default-features

echo ""
echo "✓ WASM build complete!"
echo ""
echo "Output directories:"
echo "  - pkg/bundler  (for webpack/rollup/vite)"
echo "  - pkg/nodejs   (for Node.js)"
echo "  - pkg/web      (for ES modules)"
echo ""
echo "Note: The WASM build excludes:"
echo "  - MCP server (requires native networking)"
echo "  - wasmtime (can't run WASM inside WASM)"
echo "  - Native HTTP/WebSocket (use Web API delegation instead)"
echo ""
