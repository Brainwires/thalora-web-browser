# Custom V8 Fork Integration for Thalora

## 🎯 Overview

This document explains how Thalora integrates with your custom V8 fork at [https://github.com/nightness/v8](https://github.com/nightness/v8) instead of using pre-built Chrome V8 binaries.

## 🏗️ Custom V8 Fork Architecture

### Why Custom V8 Fork?

1. **Full Control**: Complete control over V8 build configuration and features
2. **Custom Modifications**: Ability to add Thalora-specific optimizations
3. **Direct Comparison**: Side-by-side performance comparison with Boa engine
4. **Future-Proofing**: Independence from Chrome's V8 release cycle

### Build Process Flow

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   cargo build   │───▶│   build.rs       │───▶│  V8 Fork Clone  │
│  --features     │    │  (V8_FROM_SOURCE │    │  nightness/v8   │
│   v8-engine     │    │   = 1)           │    │                 │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                                │
                                ▼
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Thalora V8    │◀───│   V8 Compile     │◀───│   GN + Ninja    │
│   Integration   │    │   (15-30 mins)   │    │   Build System  │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

## 🔧 Build Configuration

The `engines/v8/build.rs` script automatically:

1. **Forces Source Build**: Sets `V8_FROM_SOURCE=1`
2. **Configures Repository**: Points to `https://github.com/nightness/v8.git`
3. **Sets Build Flags**: Optimizes V8 for Thalora usage
4. **Checks Dependencies**: Validates build tool availability

### Key Build Settings

```rust
// In build.rs
env::set_var("V8_FROM_SOURCE", "1");
env::set_var("RUSTY_V8_MIRROR", "https://github.com/nightness/v8.git");

let gn_args = [
    "v8_enable_sandbox=false",           // Better interop with Rust
    "v8_expose_symbols=true",            // Enable debugging symbols
    "v8_use_external_startup_data=false", // Embed startup data
    "v8_enable_pointer_compression=false", // Compatibility
    "use_custom_libcxx=false",           // Use system C++ library
    "treat_warnings_as_errors=false"     // Allow V8 warnings
].join(" ");
```

## 📦 Environment Variables

### Primary Configuration
```bash
# Use your custom V8 fork (default)
export THALORA_V8_REPO="https://github.com/nightness/v8.git"
export THALORA_V8_BRANCH="main"

# Build Thalora with V8
cargo build --features v8-engine
```

### Alternative Repository
```bash
# Use a different fork/branch for testing
export THALORA_V8_REPO="https://github.com/your-experimental-fork/v8.git"
export THALORA_V8_BRANCH="experimental-features"
cargo build --features v8-engine
```

### Legacy Compatibility
```bash
# Also supports these environment variables for compatibility
export CUSTOM_V8_REPOSITORY="https://github.com/nightness/v8.git"
export CUSTOM_V8_BRANCH="main"
```

## 🚀 Usage Examples

### Command Line Usage

```bash
# Run with default Boa engine
./thalora server

# Run with your custom V8 fork
./thalora --use-v8-engine server

# Run browser session with V8
./thalora --use-v8-engine session --session-id test --socket-path /tmp/test.sock

# Check available engines
./thalora --help
```

### Programmatic Usage

```rust
use thalora::engine::{EngineFactory, EngineType};

// Create engine using your custom V8 fork
#[cfg(feature = "v8-engine")]
{
    let mut v8_engine = EngineFactory::create_engine(EngineType::V8)?;
    println!("Using V8: {}", v8_engine.version_info());
    
    let result = v8_engine.execute("console.log('Hello from custom V8!')")?;
    println!("Result: {:?}", result);
}
```

## 🔍 Build Process Details

### Phase 1: Dependency Check
The build script verifies all required tools are available:
- Python 3.6+
- Git
- Ninja build system  
- C++ compiler (clang/gcc)

### Phase 2: V8 Source Download
- Clones your V8 fork from GitHub
- Checks out specified branch
- Downloads V8 dependencies

### Phase 3: V8 Compilation
- Runs GN to generate build files
- Uses Ninja to compile V8 (15-30 minutes first time)
- Creates static libraries for linking

### Phase 4: Rust Integration
- Links compiled V8 with Rust code
- Creates Thalora V8 engine wrapper
- Runs integration tests

## 📊 Performance Comparison

### Engine Capabilities Comparison

| Feature | Boa Engine | V8 Engine (Custom Fork) |
|---------|------------|------------------------|
| **Language** | Pure Rust | C++ (via bindings) |
| **Build Time** | ~2 minutes | ~25 minutes first time |
| **Binary Size** | Smaller | Larger |
| **Performance** | Good | Excellent |
| **Memory Usage** | Lower | Higher |
| **Customization** | High (source available) | High (your fork) |
| **Web API Support** | Native implementations | Polyfills required |

### Benchmark Example

```rust
use std::time::Instant;

// Benchmark JavaScript execution
let test_code = r#"
    let sum = 0;
    for (let i = 0; i < 1000000; i++) {
        sum += Math.sqrt(i);
    }
    sum;
"#;

// Test Boa
let start = Instant::now();
let boa_result = boa_engine.execute(test_code)?;
let boa_time = start.elapsed();

// Test your custom V8
let start = Instant::now();
let v8_result = v8_engine.execute(test_code)?;
let v8_time = start.elapsed();

println!("Boa: {:?} in {:?}", boa_result, boa_time);
println!("V8 (custom): {:?} in {:?}", v8_result, v8_time);
```

## 🛠️ Development Workflow

### Making Changes to Your V8 Fork

1. **Modify V8 Source**:
   ```bash
   cd ~/v8-development
   # Make your changes to V8 C++ code
   git commit -am "Add custom feature"
   git push origin main
   ```

2. **Test in Thalora**:
   ```bash
   cd thalora-web-browser
   cargo clean
   cargo build --features v8-engine
   cargo test --features v8-engine
   ```

3. **Benchmark Performance**:
   ```bash
   cargo test test_engine_compatibility --features v8-engine -- --nocapture
   ```

### Testing Different V8 Branches

```bash
# Test experimental branch
export THALORA_V8_BRANCH="experimental"
cargo clean && cargo build --features v8-engine

# Test specific commit
export THALORA_V8_BRANCH="commit-hash-here"  
cargo clean && cargo build --features v8-engine

# Return to main
export THALORA_V8_BRANCH="main"
cargo clean && cargo build --features v8-engine
```

## 🐛 Troubleshooting Custom V8 Build

### Common Issues

1. **Clone Failure**:
   ```
   Error: Failed to clone https://github.com/nightness/v8.git
   Solution: Verify repository exists and is accessible
   ```

2. **Build Tool Missing**:
   ```
   Error: ninja not found
   Solution: Install ninja-build package
   ```

3. **Memory Issues**:
   ```
   Error: Out of memory during V8 compilation
   Solution: Limit build jobs: export CARGO_BUILD_JOBS=1
   ```

4. **Permission Issues**:
   ```
   Error: Permission denied accessing V8 sources
   Solution: Check GitHub access tokens and repository permissions
   ```

### Debug Build Information

```bash
# Enable verbose build output
export RUST_LOG=debug
cargo build --features v8-engine 2>&1 | tee build.log

# Check build configuration
grep -A 10 "Thalora V8 Build Configuration" build.log
```

## 🎯 Integration Benefits

### For Thalora Development
- **Performance Testing**: Direct comparison with Boa engine
- **Feature Validation**: Test JavaScript compatibility
- **Debugging**: V8's excellent debugging tools
- **Production Alternative**: Proven engine for critical workloads

### For V8 Fork Development  
- **Real-world Testing**: Thalora provides comprehensive test suite
- **Performance Metrics**: Built-in benchmarking capabilities
- **Integration Examples**: See how V8 integrates with Rust applications
- **API Compatibility**: Ensure changes don't break existing functionality

This custom V8 integration gives you complete control over the JavaScript engine while maintaining compatibility with Thalora's architecture and your ongoing Boa updates.