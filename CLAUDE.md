# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

### Build System
```bash
# Build the project (development)
cargo build

# Build optimized release
cargo build --release

# Check code without building
cargo check

# Run the MCP server
./target/release/synaptic
# or during development:
cargo run
```

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

**Synaptic** is a pure Rust headless web browser designed specifically for AI model integration through the Model Context Protocol (MCP). It provides comprehensive web browsing capabilities without any Chrome/Chromium dependencies.

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

Start the MCP server: `./target/release/synaptic`

## Development Patterns

### Adding New Features
1. Place implementation in appropriate directory (`engine/`, `apis/`, `features/`, `protocols/`)
2. Add module to respective `mod.rs` file
3. Export public APIs in `lib.rs` if needed
4. Create corresponding test in `tests/` directory
5. Run full test suite to ensure compatibility

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