# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **Page JavaScript on V8.** Thalora links the page runtime of the Brainwires deno fork (`deno_web_page`, pinned by git revision in `Cargo.toml` and `scripts/deno-fork.rev`): one V8 isolate per page on the browser's thread, web APIs only, bound to the live `thalora-dom` document. Navigation runs the page's classic scripts (inline and external, with the page's CSP, SRI, mixed-content and SSRF checks and the resource cache; `defer`/`async` ordering; scripts inserted by scripts), fires `DOMContentLoaded` and `load`, and lets timers and requests run (1 s, 5 s with `wait_for_js`). Navigations scripts ask for (`location.href = …`, `form.submit()`) are followed.
- **The JavaScript tools work again:** `cdp_runtime_evaluate` / `browser_eval` (completion value as JSON; a returned promise or top-level `await` is awaited), `browser_console_messages` (console output, uncaught errors, script load failures), `browser_click_element` (events, handlers, default action), `browser_type_text` / `browser_fill` (`input`/`change`; unnamed fields, selects, contenteditable), `browser_select_option`, `browser_check`, `browser_press_key` (key events and default actions), `browser_hover`, `browser_scroll` (events only: no layout in script yet), `browser_wait` / `browser_wait_for_element` (the page runs while waiting).
- **A page keeps running between tool calls:** the browser thread pumps the current page's event loop while it has pending work (for 30 s after the last call).
- **Limits:** a watchdog terminates a script, evaluation or event-loop task after `THALORA_JS_SCRIPT_TIMEOUT_MS` (10 s); `THALORA_JS_HEAP_LIMIT_MB` (512) caps a page's heap. `THALORA_JS=0` disables scripts; `THALORA_PROFILE_DIR` (default `~/.thalora/profile`) holds pages' `localStorage`.
- `HeadlessWebBrowser::evaluate_javascript`, `set_javascript_enabled`, `js_settings[_mut]`, `console_messages`, `javascript_status`; `PageDom::has_js`, `js_note`, `static_doc_mut`, `live_form_states`.

### Changed
- **BREAKING (build): V8 is linked.** Builds need the verified prebuilt V8 archive: `eval "$(python3 scripts/fetch_rusty_v8.py --print-env)"` before `cargo build` (a build without it fails instead of downloading an unverified archive, see `.cargo/config.toml`). `[patch.crates-io]` pins `v8` to Brainwires/rusty_v8 and `sys_traits` to the fork's vendored copy; `url` is 2.5.9 (RUSTSEC-2026-0592).
- **Navigation results:** the page URL is the response's final URL (after redirects), and when scripts ran the returned content is the document as they left it. Tool outputs gain a `javascript` field (`running`, or `static: <why>`).
- **`thalora::engine::JS_UNAVAILABLE`** is now `JavaScript execution is unavailable: this page was loaded as static HTML`, returned (with the reason) only on static pages.
- **`PageDom::with_doc_mut` is replaced by `static_doc_mut()`** (`None` on a page running JavaScript, whose document changes through the runtime).
- **Search engine browsers do not run page scripts** (the result parsers read the served markup).
- **Docker:** the builder image is `rust:1.99.1-bookworm` (the fork's toolchain) and fetches the V8 archive.

### Fixed
- **Boa GC underflow panic** — `dec_ref_count()` in `boa_gc` had no guard against decrementing a zero ref count, causing `attempt to subtract with overflow` panics on complex pages (Wikipedia, Twitter, etc.). Fixed by saturating at zero.
- **Boa debug assertions disabled in dev** — `debug_assert!` macros in `boa_gc`, `boa_engine`, `boa_parser` etc. were firing as panics on real-world JS (GitHub React bundle, Wikipedia MediaWiki scripts). Added `debug-assertions = false` per-package under `[profile.dev.package."boa_*"]` in `Cargo.toml`, matching Boa's own release behavior.
- **Navigation crash from cross-thread Boa GC corruption** — navigating away from a complex page (GitHub → Google) caused a SIGSEGV in `boa_gc::VTable::trace_fn`. Root cause: each `NavigateAsync` creates a new OS thread; when the old `RustRenderer` (created on the previous thread) was dropped on the new thread, its finalizers created GC objects holding stale pointers to the old thread's `GcBox` allocations. The next GC sweep on the new thread followed those dangling pointers and crashed. Fixed by `mem::forget`-ing the old renderer (skipping all finalizers) before creating the fresh context for each navigation.

### Removed
- **BREAKING: the Boa JavaScript engine and everything bound to it** — V8 (the Brainwires deno fork, through `deno_core`) supersedes Boa and is being wired in; Boa is deleted now, so **`thalora` has no JavaScript execution until V8 lands**. Removed:
  - the `engines/boa` git submodule (and `.gitmodules`), the `engines/thalora-browser-apis` crate (Web APIs, DOM arena, event loop), `src/apis/` (polyfills, media, service worker, geolocation, credentials and URL bindings), `src/features/webgl.rs`, the JS renderer (`RustRenderer`, the JS security validator, the ES module loader), `JavaScriptEngine` and the `EngineConfig` / `EngineType` / `EngineFactory` / `ThaloraBrowserEngine` abstractions;
  - page WebAssembly (`wasmtime`, `wasmtime-wasi`, `wasmtime-wast`, `wat`, `wasmparser`, `wasm-encoder`) and the `wasm-debug` feature with its `wasm_debug_*` MCP tools (`wasmprinter`);
  - the async `webrtc` dependency (`Brainwires/webrtc-rs`); WebRTC returns through the deno fork's `ext/webrtc`;
  - the media and geolocation backends that only served those APIs (`cpal`, `hound`, `symphonia`, `rodio`, `ffmpeg-next`, `sysinfo`). The `native` feature remains as an alias of `desktop`;
  - the tests, fixtures and example that exercised JavaScript behaviour;
  - the environment variables `THALORA_EVENT_LOOP`, `THALORA_IDLE_PUMP_MS`, `THALORA_LEAK_RENDERERS`, `THALORA_DOM`, `THALORA_PAGE_SCRIPT_TIMEOUT_MS`, `THALORA_ENGINE` and `THALORA_ENABLE_WASM_DEBUG`.
- **BREAKING: Avalonia desktop GUI** — deleted `gui/` (ThaloraBrowser, BrowserController, ThaloraBrowser.Tests, the .sln) and everything in Rust that existed only for it: the C ABI in `src/ffi/` (`thalora_*` exports for P/Invoke), the `cdylib` crate type, the `cargo xtask gui-screenshot` / `gui-compare` commands and the `scripts/ralph` visual-parity loop. The desktop UI moves to GPUI; see [docs/GPUI_PLAN.md](docs/GPUI_PLAN.md). The MCP server, headless sessions and the CLI are unchanged.

### Changed
- **BREAKING: pages are loaded without running scripts.** Navigation fetches and parses the page (and its stylesheets) and stops; `wait_for_js` is accepted and ignored. Snapshots, page text, readability, the accessibility tree and screenshots read the static HTML.
- **BREAKING: tools that need JavaScript return an error instead of running.** `cdp_runtime_evaluate`, `browser_eval`, `browser_console_messages`, `browser_hover`, `browser_scroll`, `browser_press_key` (other than Enter in a form field) and clicks on elements without a native default action stay registered but return `JavaScript execution is unavailable: the Boa engine was removed; V8 support is in progress` (also exported as `thalora::engine::JS_UNAVAILABLE`).
- **Interaction is the native default action, from static HTML.** `browser_type_text` / `browser_fill` / `browser_fill_credential` record the value for the field's `name` (sent on submit) instead of setting it through script; clicks submit forms, follow links or toggle checkboxes/radio buttons; `browser_select_option` and `browser_check` update the submitted form data. No DOM events are dispatched.
- **`browser_wait` / `browser_wait_for_element` answer immediately** — without scripts the page cannot change while waiting.
- **API:** `BrowserThread::spawn(name)` and `BrowserThread::run_once(name, f)` no longer take an engine type; `McpServer::new_with_engine`, `McpServerService::with_engine` and `HeadlessWebBrowser::new_with_engine` are gone (use `new()`); `run_http_transport(host, port)` no longer takes an engine config.
- **CI:** the "Baselines (reporting)" job (page corpus, fingerprint dump/diff, WPT) is removed until V8 is in; the corpus test is `#[ignore]`d. `cargo xtask fingerprint-*` and `cargo xtask wpt` still build.
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
