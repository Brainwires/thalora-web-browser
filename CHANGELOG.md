# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed
- **Boa GC underflow panic** — `dec_ref_count()` in `boa_gc` had no guard against decrementing a zero ref count, causing `attempt to subtract with overflow` panics on complex pages (Wikipedia, Twitter, etc.). Fixed by saturating at zero.
- **Boa debug assertions disabled in dev** — `debug_assert!` macros in `boa_gc`, `boa_engine`, `boa_parser` etc. were firing as panics on real-world JS (GitHub React bundle, Wikipedia MediaWiki scripts). Added `debug-assertions = false` per-package under `[profile.dev.package."boa_*"]` in `Cargo.toml`, matching Boa's own release behavior.
- **SVG logo invisible** — inline `<svg>` elements with `display: inline` (CSS default) were routed to `AddInlineContent` which had no SVG handler, silently dropping them. Fixed by moving `"svg"` into `AlwaysBlockTags` and checking that set before the display override in `IsInlineElement`.
- **Submit buttons / form controls inside inline spans dropped** — `<input type="submit">` nested inside `<span>` fell through `AddInlineContent`'s `default` handler producing an empty `Span`. Fixed by adding explicit `input`/`button`/`select`/`textarea` cases to `AddInlineContent` that delegate to `BuildControl` wrapped in `InlineUIContainer`.
- **Form controls inheriting Avalonia Dark theme** — `TextBox`, `Button`, `CheckBox`, `ComboBox`, `RadioButton` defaulted to dark FluentTheme colors when CSS provided no `background-color`. Fixed by setting web-standard defaults (`Background = Brushes.White`, `Foreground = Brushes.Black`) on all form controls.
- **Image `MaxWidth` not propagating to Border wrapper** — when `border-radius` was present, the `<img>` was wrapped in a `Border` that didn't inherit `MaxWidth`, allowing it to stretch full-width. Fixed by copying sizing constraints from image to wrapper.
- **Text vertically misaligned in flex rows** — `SelectableTextBlock` defaulted to `VerticalAlignment.Top`, causing text to appear at the cell top edge instead of centered. Fixed by setting `VerticalAlignment = VerticalAlignment.Center` in `BuildInlineContent`.
- **App deadlock on window close** — `BrowserControlServer.Dispose()` called `_listenTask.Wait()` on the UI thread while in-flight HTTP handlers were blocked on `Dispatcher.UIThread.InvokeAsync()`. Fixed by removing the blocking `.Wait()`.
- **`async void` use-after-free on close** — `WebContentControl.OnHtmlContentChanged()` could resume after window teardown. Fixed with `_disposed` flag checked after every `await`.
- **Engine destroyed during in-flight FFI call** — `BrowserTabViewModel.Dispose()` destroyed the engine immediately. Fixed by delaying disposal 150 ms via `Task.Delay`.
- **Data URI images failed to load** — `ImageCache` attempted HTTP GET on `data:` URIs. Fixed by detecting the scheme and decoding base64 directly.
- **Timing diagnostics always printed** — render timing logs were unconditional. Moved behind `#if DEBUG`.

### Added
- **`cargo xtask gui-screenshot [URL]`** — build, launch, capture PNG screenshot. Options: `--out`, `--port`, `--delay`, `--no-build`, `--no-kill`.
- **`cargo xtask gui-compare [URL]`** — screenshot + open side-by-side with `--ref` reference image.

### Removed
- **Dead old rendering pipeline** — deleted `HtmlRenderer.cs`, `PaintContext.cs`, `LayoutBox.cs`, `CssComputedStyle.cs`, `StyleResolver.cs`, `TextLayout.cs`, `LayoutEngine.cs`, `HitTester.cs`.

### Changed
- **Structured logging** — replaced all `eprintln!` debug output in MCP tool dispatch, browser session management, CDP event emission, and HTTP transport with `tracing` macros (`debug!`/`info!`/`warn!`), eliminating `format!()` allocations on the hot path when debug level is filtered out
- **Tracing `EnvFilter`** — tracing subscriber now respects `RUST_LOG` env var for log-level control (default: `info`; silent mode: `warn`)
- **HTTP session limit** — MCP HTTP transport now caps concurrent sessions (default: 64, configurable via `THALORA_MAX_MCP_SESSIONS`); returns HTTP 503 with JSON-RPC error when limit is reached, preventing unbounded OS thread creation
- **Socket readiness polling** — replaced hardcoded 100ms sleep after browser process spawn with exponential-backoff polling (10ms–200ms steps, 2s max timeout), reducing session creation latency in the common case
- **CDP event diagnostics** — `CdpServer::emit_event()` now logs dropped events at `debug` level instead of silently discarding send failures

### Added
- **CSS layout bridge** — `getBoundingClientRect()`, `offsetWidth/Height/Left/Top`, `clientWidth/Height`, `scrollWidth/Height` now return real values computed by the taffy layout engine instead of zeroes
- **Layout geometry injection** — `update_document_html()` runs taffy layout and caches geometry on `DocumentData`; elements created via `querySelector` automatically receive their computed bounding rects
- **SRI for stylesheets** — Subresource Integrity hash verification (sha256/384/512) now applies to `<link rel="stylesheet">` tags, not just scripts
- **Mixed content blocking** — HTTP subresources (scripts, stylesheets, images, fonts) are blocked on HTTPS pages per the W3C Mixed Content specification
- **`::part()` pseudo-element** — Shadow DOM CSS scoping now supports the `::part(name)` selector for styling elements with `part` attributes inside shadow trees (respects shadow boundary)
- **`element.shadowRoot` default accessor** — returns `null` per spec when `attachShadow()` has not been called (previously returned `undefined`)
- **MutationObserver notification dispatch** — added `notify_attribute_mutation()` and `notify_child_list_mutation()` functions with thread-local observer registry and `subtree: true` support via parent-chain walking
- **CSSStyleSheet constructor** — constructable stylesheets with `insertRule()`, `deleteRule()`, `replace()`, `replaceSync()`, and `cssRules` property per CSSOM spec
- **`document.adoptedStyleSheets`** — getter/setter for applying constructable stylesheets to the document
- **SVG DOM support** — SVG child elements (path, rect, circle, g, etc.) are now preserved in the DOM tree instead of being skipped; `querySelector('svg path')` works correctly
- **SVGElement / SVGSVGElement** global constructors with SVG namespace and viewBox property
- **Service Worker fetch interception wiring** — `navigator.serviceWorker.register()` now registers a fetch handler in the global SW registry, enabling fetch interception via `FetchEvent` dispatch for matching URL scopes

### Changed
- **SRI verification** extracted from `javascript.rs` into shared `sri.rs` module, reused for both scripts and stylesheets
- **Stylesheet fetching** now checks mixed content policy before fetching and verifies SRI integrity after fetching
- **SVG elements in page_layout.rs** no longer return early; SVG dimensions are still extracted but children are recursed into for proper DOM tree construction

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
