# Thalora V8 Engine Integration

This directory contains the V8 JavaScript engine integration for Thalora, using a **custom V8 fork** at https://github.com/nightness/v8. The V8 engine is **ALWAYS** built from source code, never from precompiled binaries.

## Overview

The V8 engine integration uses `rusty_v8` (published as the `v8` crate), but is **always configured to build from your custom V8 fork** rather than using pre-built Chrome binaries. The build script automatically sets `V8_FROM_SOURCE=1` to ensure this behavior.

## Why Always Build From Source

- **Custom Modifications**: Your V8 fork includes specific patches and optimizations
- **Full Control**: Complete control over V8 build configuration and features  
- **Latest Changes**: Always get the latest changes from your V8 development
- **Security**: No reliance on potentially compromised binary artifacts
- **Debugging**: Better debugging capabilities with custom symbols and configurations
- **Reproducible Builds**: Consistent builds across different environments

## Build Configuration

### Environment Variables

- `V8_FROM_SOURCE=1`: **Automatically set by build script** - forces building from source (NEVER prebuilt binaries)
- `CUSTOM_V8_REPOSITORY`: Override the V8 repository URL (defaults to nightness/v8 fork)
- `CUSTOM_V8_BRANCH`: Specify a custom branch to build from
- `GN_ARGS`: Additional GN build arguments for V8

### Build Process

The build script (`build.rs`) automatically configures the build to:

1. **Force Source Build**: Sets `V8_FROM_SOURCE=1` to avoid prebuilt binaries
2. **Custom Repository**: Points to https://github.com/nightness/v8
3. **Build Configuration**: Configures V8 with appropriate flags for Thalora
4. **Symbol Exposure**: Enables symbol exposure for better debugging

## Build Requirements

Building V8 from source requires significant system resources and dependencies:

### System Requirements
- **RAM**: At least 16GB recommended (V8 compilation is memory-intensive)
- **Disk**: ~20GB free space for build artifacts
- **Time**: Initial build takes 15-30 minutes depending on system

### Dependencies
- **Python 3.8+**: Required for V8 build system
- **Ninja**: Build system (install with `pip install ninja`)
- **Git**: For cloning repositories
- **C++ Compiler**: GCC 9+ or Clang 12+
- **pkg-config**: For dependency management

### Installation Commands

```bash
# Ubuntu/Debian
sudo apt update
sudo apt install build-essential python3 python3-pip git pkg-config
pip3 install ninja

# macOS
brew install python ninja git pkg-config

# Arch Linux
sudo pacman -S base-devel python python-pip git pkgconf ninja
```

## Usage

### Building with Custom V8

```bash
# Build with default custom fork
cargo build --features v8-engine

# Use different repository
CUSTOM_V8_REPOSITORY=https://github.com/your-org/v8 cargo build --features v8-engine

# Use specific branch
CUSTOM_V8_BRANCH=your-feature-branch cargo build --features v8-engine

# Debug build with symbols
V8_FORCE_DEBUG=1 cargo build --features v8-engine
```

### Running with V8

```bash
# Use V8 engine
./thalora --use-v8-engine server

# With debug output
RUST_LOG=debug ./thalora --use-v8-engine server
```

## Architecture

- **`engine.rs`**: Core V8 engine wrapper implementing the same interface as the Boa engine
- **`runtime.rs`**: Higher-level runtime management with async support
- **`context.rs`**: Context management for JavaScript execution isolation
- **`polyfills.rs`**: Web API polyfills and browser-like functionality
- **`build.rs`**: Custom build script for V8 source compilation

## Features

- **Custom V8 Build**: Uses your V8 fork instead of Chrome binaries
- **API Compatibility**: Implements the same interface as Thalora's Boa engine
- **Web APIs**: Provides browser APIs that are native in Boa but need polyfills in V8:
  - Console API (`console.log`, etc.)
  - Timer functions (`setTimeout`, `setInterval`)
  - Storage APIs (`localStorage`, `sessionStorage`)
  - Event system (`Event`, `EventTarget`)
  - WebSocket (placeholder)
  - Fetch (placeholder)
- **Async Support**: Tokio-based async execution model
- **Testing Support**: Includes test utilities for engine comparison

## Performance Comparison

The custom V8 integration allows for direct performance comparison between:

- **Boa Engine**: Pure Rust implementation, smaller binary, faster compilation
- **Custom V8 Engine**: Production-proven with your modifications, extensive optimization

```rust
// Benchmark the same code on both engines
let test_code = "/* complex JavaScript */";

let start = std::time::Instant::now();
let boa_result = boa_engine.execute(test_code)?;
let boa_time = start.elapsed();

let start = std::time::Instant::now(); 
let v8_result = v8_engine.execute(test_code)?;
let v8_time = start.elapsed();

eprintln!("Boa: {:?} in {:?}", boa_result, boa_time);
eprintln!("Custom V8: {:?} in {:?}", v8_result, v8_time);
```

## Development Workflow

### Testing Custom V8 Changes

1. **Make changes** in your V8 fork at https://github.com/nightness/v8
2. **Push changes** to your fork
3. **Rebuild Thalora** with `cargo clean && cargo build --features v8-engine`
4. **Test integration** with `cargo test --features v8-engine`

### Debugging

```bash
# Enable V8 debug symbols
V8_FORCE_DEBUG=1 cargo build --features v8-engine

# Enable Rust debug logs
RUST_LOG=thalora_v8_engine=debug ./thalora --use-v8-engine server

# GDB debugging
gdb --args ./target/debug/thalora --use-v8-engine server
```

## Troubleshooting

### Common Issues

**Build Fails - Missing Dependencies**
```
error: failed to run custom build command for `v8`
```
Solution: Install all V8 build dependencies listed above.

**Out of Memory During Build**
```
c++: fatal error: Killed
```
Solution: Increase swap space or use a machine with more RAM.

**Custom Fork Not Used**
```
warning: falling back to prebuilt binaries
```
Solution: Ensure `V8_FROM_SOURCE=1` is set and build dependencies are installed.

**Build Takes Too Long**
- This is normal for V8 builds (15-30 minutes)
- Use `ninja -j$(nproc)` to utilize all CPU cores
- Consider using a powerful build machine

### Advanced Configuration

For advanced V8 build configuration, you can set custom GN arguments:

```bash
# Example: Enable specific V8 features
export GN_ARGS="v8_enable_sandbox=false v8_expose_symbols=true v8_enable_debugging_features=true"
cargo build --features v8-engine
```

## Integration with Thalora

The V8 engine integrates seamlessly with Thalora's existing architecture:

- **Engine Selection**: Runtime switching via `--use-v8-engine` flag
- **API Compatibility**: Same interface as Boa engine
- **Web API Polyfills**: Provides APIs that Boa implements natively
- **Testing Framework**: Comprehensive tests for both engines

This custom V8 integration provides the foundation for advanced JavaScript execution while maintaining compatibility with Thalora's ongoing Boa development.