# Thalora V8 Engine Integration

This directory contains the V8 JavaScript engine integration for Thalora, providing a production-ready alternative to the Boa engine for direct comparison and testing.

## Custom V8 Fork Integration

This implementation uses a **custom V8 fork** from [https://github.com/nightness/v8](https://github.com/nightness/v8) instead of the standard Chrome V8 binaries. This allows for:

- Custom V8 modifications specific to Thalora's needs
- Full control over V8 build configuration
- Ability to add custom APIs or optimizations
- Direct comparison with the evolving Boa engine

## Build Requirements

Since we build V8 from source using the custom fork, you'll need:

### Linux/macOS:
```bash
# Essential build tools
sudo apt-get install python3 ninja-build git build-essential  # Ubuntu/Debian
# or
brew install python3 ninja git  # macOS

# C++ compiler (clang recommended)
sudo apt-get install clang  # Ubuntu/Debian
```

### Windows:
- Visual Studio 2019 or later with C++ build tools
- Python 3.6+
- Git

## Building with Custom V8 Fork

### Default Build (uses nightness/v8 fork):
```bash
# Build Thalora with V8 engine support
cargo build --features v8-engine
```

### Custom Repository/Branch:
```bash
# Use a different repository
export THALORA_V8_REPO="https://github.com/your-fork/v8.git"
export THALORA_V8_BRANCH="your-branch"
cargo build --features v8-engine

# Or use the legacy environment variables
export CUSTOM_V8_REPOSITORY="https://github.com/your-fork/v8.git"
export CUSTOM_V8_BRANCH="your-branch"
cargo build --features v8-engine
```

### Build Configuration

The build system automatically configures V8 with optimizations for Thalora:

- `v8_enable_sandbox=false` - Disable sandboxing for better interop
- `v8_expose_symbols=true` - Enable symbol exposure for debugging
- `v8_use_external_startup_data=false` - Embed startup data
- `v8_enable_pointer_compression=false` - Disable for compatibility
- `use_custom_libcxx=false` - Use system C++ library
- `treat_warnings_as_errors=false` - Allow warnings during build

## Architecture

- **`engine.rs`**: Core V8 engine wrapper implementing the same interface as the Boa engine
- **`runtime.rs`**: Higher-level runtime management with async support
- **`context.rs`**: Context management for JavaScript execution isolation
- **`polyfills.rs`**: Web API polyfills for APIs that are native in Boa

## Web API Compatibility

Since Boa has many native Web API implementations, the V8 engine provides polyfills for:

- **Console API**: `console.log`, `console.error`, etc.
- **Timer Functions**: `setTimeout`, `setInterval`, `clearTimeout`, `clearInterval`
- **Storage APIs**: `localStorage`, `sessionStorage` (placeholders)
- **Event System**: `Event`, `EventTarget`, `CustomEvent`
- **WebSocket API**: Basic placeholder implementation
- **Fetch API**: Placeholder (returns rejected promises)

## Performance Comparison

The V8 integration allows direct performance comparison:

```bash
# Run with Boa (default)
./thalora server

# Run with V8 (custom fork)
./thalora --use-v8-engine server

# Benchmark both engines
cargo test --features v8-engine test_engine_compatibility
```

## Build Times

**First Build**: 15-30 minutes (V8 compilation)
**Subsequent Builds**: 1-3 minutes (incremental compilation)

The long initial build time is due to compiling the entire V8 engine from source. Subsequent builds are much faster due to Cargo's incremental compilation.

## Troubleshooting

### Build Issues

1. **Python Not Found**:
   ```bash
   # Install Python 3.6+
   sudo apt-get install python3  # Linux
   brew install python3          # macOS
   ```

2. **Ninja Not Found**:
   ```bash
   # Install ninja build system
   sudo apt-get install ninja-build  # Linux
   brew install ninja                # macOS
   ```

3. **C++ Compiler Issues**:
   ```bash
   # Install clang (recommended for V8)
   sudo apt-get install clang  # Linux
   xcode-select --install      # macOS
   ```

4. **Memory Issues During Build**:
   ```bash
   # Limit parallel jobs to reduce memory usage
   export CARGO_BUILD_JOBS=2
   cargo build --features v8-engine
   ```

### Runtime Issues

1. **V8 Initialization Failed**:
   - Ensure the build completed successfully
   - Check that all required libraries are available

2. **Performance Issues**:
   - Try debug mode: `cargo build --features v8-engine,debug`
   - Enable V8 profiling flags in `build.rs`

## Development

### Adding New V8 Features

1. Modify your V8 fork at https://github.com/nightness/v8
2. Update `THALORA_V8_BRANCH` if using a different branch
3. Rebuild: `cargo clean && cargo build --features v8-engine`

### Testing Custom V8 Changes

```bash
# Test with specific V8 commit
export THALORA_V8_BRANCH="commit-hash-or-branch"
cargo test --features v8-engine

# Compare with Boa
cargo test test_engine_compatibility --features v8-engine
```

## Integration with Thalora's Boa Updates

As Thalora migrates from polyfills to native Boa implementations, this V8 engine:

- ✅ Provides polyfills for APIs that Boa handles natively
- ✅ Maintains the same interface as the Boa wrapper
- ✅ Allows performance comparison during migration
- ✅ Serves as a reference implementation for Web API compatibility

This ensures that as Boa continues to evolve and add native implementations, the V8 engine remains a viable alternative for comparison and testing.