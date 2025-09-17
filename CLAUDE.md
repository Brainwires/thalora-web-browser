# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## PRIORITY DIRECTIVES

* WE DON'T DO SIMPLER APPROACHES!
* We will not be simplifing code
* We will not be mocking code. If we are missing a huge library, and a crate is not available... Writing the new code becomes the next task (REGARDLESS OF HOW LONG IT COULD TAKE!); so the main project can progress!
* This statement scares the shit out of me... Every time you say it, something breaks... "Actually, let me try a simpler approach." Let's ALWAYS take the proper approach... Prioritize doing tasks properly rather than quickly. When you have the desire to "simplify", instead use thinking to resolve the problem.

## Development Commands

### Build System
```bash
# Build the project (development) - 10 HOUR TIMEOUTS!!!!!!!!!
cargo build

# Build optimized release - 10 HOUR TIMEOUTS!!!!!!!!!
cargo build --release

# Check code without building - 10 HOUR TIMEOUTS!!!!!!!!
cargo check

# Run the MCP server
./target/release/thalora
# or during development:
cargo run
```

**CRITICAL BUILD NOTE**: Wasmtime compilation is complex and takes time. DO NOT use timeouts on cargo build/check/test commands - let them run to completion naturally.

### Testing
```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test modules
cargo test --test browser_test
cargo test --test mcp_test
cargo test engine
cargo test apis
cargo test features
cargo test protocols

# Run tests with debug output
RUST_BACKTRACE=1 cargo test

# Run tests quietly (less output)
cargo test --quiet
```

### Debugging and Analysis
```bash
# Test with debug logging
RUST_LOG=debug cargo test test_name -- --nocapture
RUST_LOG=debug cargo run

# Generate documentation
cargo doc --open
```

## Project Architecture

**Thalora** is a pure Rust headless web browser designed specifically for AI model integration through the Model Context Protocol (MCP). It provides comprehensive web browsing capabilities without any Chrome/Chromium dependencies.

### Core Structure
```
src/
├── engine/          # Core browser engine (HTTP client, DOM, JavaScript execution)
├── apis/            # Web standards implementation (Fetch, WebSocket, Storage, etc.)
├── features/        # Advanced capabilities (AI memory, fingerprinting, challenge solving)
├── protocols/       # Communication protocols (MCP server, Chrome DevTools Protocol)
├── lib.rs          # Public API exports
└── main.rs         # MCP server entry point
```

### Key Components

**Browser Engine** (`src/engine/`):
- `browser.rs` - Main HeadlessWebBrowser class with HTTP client and session management
- `renderer.rs` - JavaScript execution engine with DOM integration via Boa
- `engine.rs` - Advanced JavaScript runtime with timers, promises, async support
- `dom.rs` - DOM tree management and element manipulation

**Web APIs** (`src/apis/`):
- Modern web standards implementation (Fetch, WebSocket, Storage, Crypto, etc.)
- ES2017-2025 polyfills for JavaScript compatibility
- Comprehensive polyfill system in `polyfills/` directory

**AI Features** (`src/features/`):
- `ai_memory.rs` - Persistent AI memory heap for research, credentials, bookmarks
- `fingerprinting.rs` - Browser fingerprinting and stealth capabilities
- `react_processor.rs` - Server-side React/Next.js processing
- `solver/` - Challenge solving for Cloudflare, CAPTCHAs, etc.

**Communication Protocols** (`src/protocols/`):
- `mcp_server.rs` - Model Context Protocol server implementation
- `cdp.rs` - Chrome DevTools Protocol compatibility for debugging
- `memory_tools.rs` - Memory management tools for MCP integration

### Test Architecture

Tests are organized to mirror the `src/` directory structure:
```
tests/
├── engine/          # Core browser functionality tests
├── apis/           # Web API compliance tests
├── features/       # Advanced feature tests
└── protocols/      # MCP and CDP protocol tests
```

## MCP Integration

The project serves as an MCP server providing 17+ tools for AI models:

**AI Memory Tools**: Store research, credentials, bookmarks, and notes with persistent storage
**Chrome DevTools Protocol**: Complete CDP implementation for debugging and inspection
**Web Automation**: Full browser automation with JavaScript execution, form handling, stealth browsing

Start the MCP server: `./target/release/thalora`

### Testing MCP Tools
**IMPORTANT**: This is a stdio-based MCP server. Do NOT write test files - use stdin/stdout piping instead.

Test MCP tools by piping JSON requests directly to the executable:
```bash
# Test tools list
echo '{"jsonrpc": "2.0", "id": 1, "method": "tools/list", "params": {}}' | ./target/release/thalora

# Test Google search
echo '{"jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": {"name": "google_search", "arguments": {"query": "test", "num_results": 1}}}' | ./target/release/thalora

# Test URL scraping
echo '{"jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": {"name": "scrape_url", "arguments": {"url": "https://httpbin.org/html", "wait_for_js": false}}}' | ./target/release/thalora
```

## Development Patterns

### Adding New Features
1. Place implementation in appropriate directory (`engine/`, `apis/`, `features/`, `protocols/`)
2. Add module to respective `mod.rs` file
3. Export public APIs in `lib.rs` if needed
4. Create corresponding test in `tests/` directory
5. Run full test suite to ensure compatibility

### Modifying the Boa JavaScript Engine
**IMPORTANT**: When adding features to the Boa engine (`engines/boa/`):
1. **Update Documentation**: Always update `engines/boa/ADDED-FEATURES.md` to document new Web APIs or engine features
2. **Follow Boa Patterns**: Use Boa's builtin system (`IntrinsicObject`, `BuiltInObject`, `BuiltInConstructor`)
3. **Real Implementation**: No mocks - implement actual functionality with real networking/system calls
4. **Standards Compliance**: Follow WHATWG/W3C specifications exactly
5. **Integration Steps**:
   - Add new builtin modules to `core/engine/src/builtins/mod.rs`
   - Register intrinsics in `core/engine/src/context/intrinsics.rs`
   - Add dependencies to `core/engine/Cargo.toml` if needed
   - Update string constants in `core/string/src/common.rs` for optimization
6. **Git Workflow**: Commit changes to Boa submodule with descriptive messages

### Security Considerations
- JavaScript execution is sandboxed with 5-second timeouts
- Dangerous patterns (`eval`, `Function`) are blocked
- All credential storage is encrypted locally
- Network requests use TLS by default

### Testing Approach
- Unit tests for individual components
- Integration tests for cross-component functionality
- Protocol compliance tests for MCP and CDP
- Web standards compliance tests using browser test patterns

## Performance Characteristics

- Base runtime: ~10MB memory
- With full features: ~40MB memory
- JavaScript execution: 100-500ms per context
- Network requests: HTTP/2 with connection pooling
- Single binary deployment with no external dependencies