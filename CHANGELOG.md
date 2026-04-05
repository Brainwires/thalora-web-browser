# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **CSS layout bridge** — `getBoundingClientRect()`, `offsetWidth/Height/Left/Top`, `clientWidth/Height`, `scrollWidth/Height` now return real values computed by the taffy layout engine instead of zeroes
- **Layout geometry injection** — `update_document_html()` runs taffy layout and caches geometry on `DocumentData`; elements created via `querySelector` automatically receive their computed bounding rects
- **SRI for stylesheets** — Subresource Integrity hash verification (sha256/384/512) now applies to `<link rel="stylesheet">` tags, not just scripts
- **Mixed content blocking** — HTTP subresources (scripts, stylesheets, images, fonts) are blocked on HTTPS pages per the W3C Mixed Content specification
- **`::part()` pseudo-element** — Shadow DOM CSS scoping now supports the `::part(name)` selector for styling elements with `part` attributes inside shadow trees (respects shadow boundary)
- **`element.shadowRoot` default accessor** — returns `null` per spec when `attachShadow()` has not been called (previously returned `undefined`)
- **MutationObserver notification dispatch** — added `notify_attribute_mutation()` and `notify_child_list_mutation()` functions with thread-local observer registry and `subtree: true` support via parent-chain walking

### Changed
- **SRI verification** extracted from `javascript.rs` into shared `sri.rs` module, reused for both scripts and stylesheets
- **Stylesheet fetching** now checks mixed content policy before fetching and verifies SRI integrity after fetching

## [0.2.0] - 2026-03-28

### Added
- **rmcp 1.3.0 integration** — replace hand-rolled stdio MCP loop with the official Anthropic Rust MCP SDK
- **HTTP MCP transport** (`--transport http`) — thread-per-session model; each connected AI agent gets an isolated OS thread with its own JavaScript engine instance, enabling multiple agents per container
- **`--transport`, `--host`, `--port` CLI flags** on the `server` subcommand (defaults: `stdio`, `0.0.0.0`, `8080`)
- **`GET /health`** endpoint returning `{"status":"ok"}` for Docker health checks
- **BrainClaw preset** — agent-friendly tool alias set enabled via `THALORA_PRESET=brainclaw`
- **`McpServerService`** — thin `rmcp::ServerHandler` wrapper around `McpServer` using `RefCell` for interior mutability (safe under rmcp's `"local"` feature serial execution guarantee)

### Changed
- `McpResponse` is now a plain struct `{ content: Vec<Value>, is_error: bool }` with a `From<McpResponse> for rmcp::model::CallToolResult` conversion at the service boundary — eliminates the hand-rolled JSON-RPC framing layer
- `McpServer::run()` now takes `self` (owned) and drives the rmcp stdio transport via `LocalSet::run_until`
- Docker `HEALTHCHECK` updated to use correct `curl -sf` flags and proper timing options
- `docker/README.md` updated with transport mode documentation, new CLI flags, corrected `/mcp` endpoint URL, and multi-session model description

### Removed
- Hand-rolled stdio JSON-RPC loop (`handle_request`, `handle_notification`, `McpRequest`, `McpNotification`, `McpMessage`, `McpMessageContent`, `ToolCall`, `InitializeResult`, `ListToolsResult` types)
- `McpResponseExt` extension trait — superseded by `McpResponse` inherent methods

### Technical Notes
- `McpServer` is `!Send` because `boa_engine::Context` uses `Rc` internally. The HTTP transport works around this with a thread-per-session model (OS thread + single-threaded tokio runtime + `LocalSet`) rather than `rmcp::StreamableHttpService` (which requires `S: Send + 'static`).
- Session routing uses `std::sync::mpsc::SyncSender` for requests and `tokio::sync::oneshot` for responses, both of which are `Send`.

## [0.1.0] - 2026-02-01

### Added
- Initial release
- Boa JavaScript engine integration
- Headless browser with CSS/layout engine
- MCP server with stdio transport
- AI memory heap (bookmarks, credentials, notes, research, sessions)
- Chrome DevTools Protocol (CDP) tools
- Web scraping and search tools
- Browser automation and session management
- VFS (virtual filesystem) with encrypted session persistence
- Rate limiting and security controls
- WASM debug tools (`wasm-debug` feature)
- Docker support
