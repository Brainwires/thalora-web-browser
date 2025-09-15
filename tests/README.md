# Test Structure

This directory contains tests organized to match the `src/` directory structure.

## Directory Organization

```
tests/
├── engine/                  # Tests for src/engine/
│   ├── browser_test.rs     # Tests src/engine/browser.rs
│   ├── browser_scraping_test.rs # Tests browser scraping functionality
│   ├── renderer_test.rs    # Tests src/engine/renderer.rs
│   ├── dom_test.rs        # Tests src/engine/dom.rs
│   ├── engine_js_test.rs  # Tests src/engine/engine.rs (JavaScript engine)
│   ├── engine_js_simple_test.rs    # Simple JS engine tests
│   └── engine_js_advanced_test.rs  # Advanced JS engine tests
│
├── apis/                   # Tests for src/apis/
│   ├── websocket_test.rs   # Tests src/apis/websocket.rs
│   ├── storage_test.rs     # Tests src/apis/storage.rs
│   ├── events_test.rs      # Tests src/apis/events.rs
│   └── polyfills/          # Tests for src/apis/polyfills/
│       └── console_test.rs # Tests src/apis/polyfills/console.rs
│
├── features/               # Tests for src/features/
│   ├── ai_memory_test.rs   # Tests src/features/ai_memory.rs
│   ├── react_processor_test.rs # Tests src/features/react_processor.rs
│   └── fingerprinting_test.rs  # Tests src/features/fingerprinting.rs
│
└── protocols/              # Tests for src/protocols/
    ├── mcp_test.rs        # Tests src/protocols/mcp.rs
    └── cdp_test.rs        # Tests src/protocols/cdp.rs
```

## Test Naming Convention

- **File naming**: `{module}_test.rs` - matches the source file being tested
- **Test module naming**: `{module}_tests` - consistent module names
- **Test function naming**: `test_{functionality}` - descriptive test names

## Running Tests

```bash
# Run all tests
cargo test

# Run tests for specific module
cargo test --test engine_tests
cargo test --test browser_test

# Run tests in specific directory
cargo test tests/engine
cargo test tests/apis
cargo test tests/features
cargo test tests/protocols
```

## Test Coverage

Each test file corresponds to a source file in `src/` and tests the key functionality of that module. Tests are organized by the architectural layers:

1. **Engine Tests**: Core browser functionality
2. **APIs Tests**: Web standards and JavaScript APIs
3. **Features Tests**: Advanced browser capabilities
4. **Protocols Tests**: Communication protocols (MCP, CDP)
