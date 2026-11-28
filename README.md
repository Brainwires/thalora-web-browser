# Thalora

[![CI](https://github.com/Brainwires/thalora-web-browser/actions/workflows/ci.yml/badge.svg)](https://github.com/Brainwires/thalora-web-browser/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A web browser written in Rust for AI agents and humans, exposed to agents as an
MCP server. Thalora owns its engine end to end, so it decides what pages can
observe about who is driving it. It does not use Chrome, Chromium or CEF.

## Status (October 2026)

Thalora works today for reading pages and for scripted interaction with
server-rendered sites. It is **mid re-architecture**; read
[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) before contributing.

> **Page JavaScript runs on V8.** Thalora links the page runtime of the
> Brainwires deno fork (`deno_web_page`: a V8 isolate per page with web APIs
> only, no `Deno` namespace, no Node), bound to Thalora's live DOM. Page
> scripts run on navigation, events are dispatched for clicks, typing and key
> presses, and timers and `fetch` keep running between tool calls. It is the
> first cut: see the gaps below. `THALORA_JS=0` turns scripts off (pages are
> then the static HTML the server sent).

What works now:

- MCP server over stdio or streamable HTTP, with page reading (readable
  markdown, links, metadata), web search, named browser sessions, agent
  snapshots with element refs, form filling and submission, clicks with their
  handlers and default actions, JavaScript evaluation, console capture, waits
  on the live page, and an accessibility tree.
- An encrypted credential vault whose secrets are filled into pages without
  ever being returned to the model (or shown to page scripts).
- SSRF protection (navigation, scripts, and the page's own `fetch`) and
  per-origin credential filling.

Known gaps, which the re-architecture addresses:

- **JavaScript is a first cut.** Classic scripts only (module scripts are
  skipped), no layout in script (`getBoundingClientRect`, `getComputedStyle`,
  scroll positions), no `MutationObserver`, `document.cookie`, `WebSocket`,
  workers or canvas, and the fingerprint is not Chrome's yet. Evaluation does
  not await promises. Page-loaded WebAssembly and WebRTC return through the
  deno fork's extensions. Scripts inserted by scripts run after the inserting
  script, not during it.
- **Rendering.** CSS cascade and layout are hand-rolled and incomplete
  (no `!important`, floats, inline formatting or tables), so screenshots are
  approximate.
- **Input.** Events carry no coordinates (no layout in script yet), hovering
  does not apply CSS `:hover`, and scrolling only fires the event.
- **No desktop UI.** The Avalonia app that used to live in `gui/` has been
  removed. The desktop UI moves to GPUI (a standalone browser shell over the
  same engine); see [docs/GPUI_PLAN.md](docs/GPUI_PLAN.md).
- Session persistence (`save_session`) is not implemented.

The target is V8 (through deno_core, with bindings generated from WebIDL; the
first, hand-written cut is linked),
Stylo + Taffy + Parley + Vello for style, layout and paint, one live DOM that
every reader uses, and a GPUI desktop shell that shows frames painted by the
same pipeline the agent sees. See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
and [docs/GPUI_PLAN.md](docs/GPUI_PLAN.md).

## Build

Requires a recent stable Rust toolchain (edition 2024; the deno fork pins
1.99.1), `pkg-config`, `cmake`, a C compiler and Python 3. V8 is linked from a
prebuilt, checksum-verified archive (Brainwires/rusty_v8) that
`scripts/fetch_rusty_v8.py` downloads; a build without it fails on purpose
instead of downloading an unverified one.

```bash
git clone https://github.com/Brainwires/thalora-web-browser.git
cd thalora-web-browser
eval "$(python3 scripts/fetch_rusty_v8.py --print-env)"   # verified prebuilt V8
cargo build --release          # MCP server binary + library
./target/release/thalora       # MCP over stdio
./target/release/thalora server --transport http --port 8080
```

| Feature | Description |
|---------|-------------|
| `desktop` (default) | MCP server binary and HTTP transport |
| `native` | Alias of `desktop` (the audio/video, WebRTC and WebAssembly backends it used to add were removed with Boa) |
| `test-hooks` | Test-only: lets release builds honour `THALORA_ALLOW_LOOPBACK=1` |

Page JavaScript settings (environment):

| Variable | Default | Meaning |
|----------|---------|---------|
| `THALORA_JS` | on | `0` disables page scripts: pages are static HTML |
| `THALORA_JS_SCRIPT_TIMEOUT_MS` | `10000` | Longest a script, evaluation or event-loop task may run before it is terminated |
| `THALORA_JS_HEAP_LIMIT_MB` | `512` | V8 heap limit per page (`0`: V8's default); reaching it stops the page's JavaScript, not the process |
| `THALORA_PROFILE_DIR` | `~/.thalora/profile` | Browser profile; pages' `localStorage` lives in `storage/`, one directory per origin |
| `THALORA_JS_SEED` | random | Seed for `crypto.getRandomValues` / `randomUUID` (deterministic runs) |

### Claude Desktop / MCP client configuration

```json
{
  "mcpServers": {
    "thalora": {
      "command": "/usr/local/bin/thalora",
      "env": { "THALORA_PRESET": "brainclaw" }
    }
  }
}
```

## MCP tools

Which tools are listed depends on the configuration (see
[docs/ENVIRONMENT_VARIABLES.md](docs/ENVIRONMENT_VARIABLES.md)):

| Configuration | How to enable | Tools |
|---|---|---|
| Minimal (default) | nothing | `snapshot_url`, `web_search` |
| Full | `THALORA_MCP_MODE=full` + `THALORA_ENABLE_*` flags | categories below |
| BrainClaw preset | `THALORA_PRESET=brainclaw` | sessions, automation, search, `cdp_runtime_evaluate`, accessibility, aliases |

Every tool returns standard MCP content blocks. Page-derived text is marked as
untrusted content. The tables below are checked against the server's tool
registry by `tests/protocols/readme_tools_test.rs`.

<!-- tools:start -->
### 🌐 Scraping & search (`THALORA_ENABLE_SCRAPING`, on by default; `THALORA_ENABLE_SEARCH`)
| Tool | Description |
|------|-------------|
| `snapshot_url` | Load a page and extract links, images, metadata, custom selectors, structured data and readable markdown, with output-size limits |
| `web_search` | Web search returning title, URL and snippet per result |
| `image_search` | Image search |

### 🧭 Browser sessions & automation (`THALORA_ENABLE_SESSIONS`)
| Tool | Description |
|------|-------------|
| `browser_session_management` | Create, list, inspect and close named browser sessions |
| `browser_navigate_to` | Navigate a session to a URL and run the page's scripts; `wait_for_js` lets its timers and requests run for up to 5 s instead of 1 s |
| `browser_navigate_back` / `browser_navigate_forward` | Move through session history |
| `browser_refresh_page` | Reload the current page |
| `browser_snapshot` | Compact page outline for agents: interactive elements with refs (`[ref=e12]`), headings, landmarks and text, within a token budget |
| `browser_screenshot` | PNG screenshot from Thalora's current layout code (approximate; images as placeholders) |
| `browser_get_page_content` | Current URL and content of a session |
| `browser_click_element` | Click an element by snapshot ref or CSS selector; pointer, mouse and click events are dispatched (handlers run), then the default action: submit buttons submit their form, links navigate and checkboxes/radio buttons change state |
| `browser_type_text` | Type into an input, textarea, select or contenteditable element (`input` and `change` fire) |
| `browser_fill_form` | Fill named fields of a form and (by default) submit it, including hidden fields such as CSRF tokens |
| `browser_select_option` | Choose a `<select>` option by value or visible text |
| `browser_check` | Check or uncheck a checkbox or radio button |
| `browser_press_key` | Press a key: `keydown`/`keypress`/`keyup`, then the default action (typing into a field, Enter submits a form, Enter/Space activates a button or link) |
| `browser_hover` | Dispatch the pointer and mouse events of moving over an element (script-driven hover menus; CSS `:hover` is not applied) |
| `browser_scroll` | Dispatch a `scroll` event (scroll-driven loaders); nothing moves, the page has no layout in script yet |
| `browser_console_messages` | The page's console output, uncaught errors and the browser's notes about its scripts (blocked, failed to load, timed out) |
| `browser_wait` | Wait for a ref/selector, page text, URL substring or an idle page while the page's timers and requests run; answers early when nothing pending could change the page |
| `browser_wait_for_element` | Wait until a selector matches (same rules) |
| `browser_prepare_form_submission` | Create a session for a form that opens a new window |
| `browser_validate_session` | Check that a session exists and has the expected content |
| `get_accessibility_tree` | ARIA roles, accessible names and states for the current page (always listed in full mode) |

Action tools (`browser_click_element`, `browser_type_text`, `browser_fill`,
`browser_select_option`, `browser_check`, `browser_press_key`, `browser_hover`,
`browser_scroll`, `browser_wait`, `browser_wait_for_element`) accept a `ref` from `browser_snapshot` instead of a
CSS selector. Refs stay valid until the page content changes.

Tools that act on an existing page return an error for an unknown `session_id`
instead of silently creating a blank session; `"default"` always exists.

### JavaScript (`THALORA_ENABLE_CDP`)
| Tool | Description |
|------|-------------|
| `cdp_runtime_evaluate` | Evaluate JavaScript in a session's page (default session `"default"`) and return the completion value as JSON. A returned promise or top-level `await` is awaited (up to 30 s); an uncaught exception or a script running past the timeout is an error |

On a static page (`THALORA_JS=0`, or a page whose runtime failed to start)
tools that need script return `JavaScript execution is unavailable: this page
was loaded as static HTML`, with the reason. The name is historical. The earlier mock CDP domains were removed; a real CDP
endpoint over live engine state is planned (see [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)).

### Credential vault (`THALORA_ENABLE_VAULT`, requires `THALORA_MASTER_PASSWORD`)
| Tool | Description |
|------|-------------|
| `vault_store_credential` | Store a site credential, encrypted at rest with AES-256-GCM (Argon2id-derived key); pass `origin` to allow filling it into that site |
| `vault_list_credentials` | List stored credentials (key, service, username, origin); secrets are never returned |
| `vault_delete_credential` | Delete a stored credential |
| `browser_fill_credential` | Type a stored username/password into the page server-side, only on the credential's own origin; the secret is never returned to the model |

The vault file is `~/.thalora/credentials.json` (override with `THALORA_VAULT_PATH`), written with owner-only permissions.

### 🤖 BrainClaw aliases (`THALORA_PRESET=brainclaw`)
| Tool | Description |
|------|-------------|
| `browser_read_url` | Navigate and return readable markdown in one call |
| `browser_navigate` | Alias for `browser_navigate_to` |
| `browser_click` | Alias for `browser_click_element` |
| `browser_fill` | Fill one field by selector; `submit=true` also submits its form |
| `browser_eval` | Alias for `cdp_runtime_evaluate` (currently returns the same error) |
| `browser_extract` | Alias for `snapshot_url` |
| `browser_search` | `web_search` with sensible defaults |
<!-- tools:end -->

### Examples

```json
{"method": "tools/call", "params": {"name": "snapshot_url", "arguments": {"url": "https://news.ycombinator.com", "extract_readable": true, "format": "markdown"}}}
{"method": "tools/call", "params": {"name": "browser_navigate_to", "arguments": {"url": "https://example.com/login", "session_id": "work"}}}
{"method": "tools/call", "params": {"name": "browser_snapshot", "arguments": {"session_id": "work"}}}
{"method": "tools/call", "params": {"name": "browser_fill_credential", "arguments": {"session_id": "work", "service": "example", "username_ref": "e3", "password_ref": "e4"}}}
{"method": "tools/call", "params": {"name": "browser_click", "arguments": {"session_id": "work", "ref": "e5"}}}
```

## Repository layout

| Path | Contents |
|------|----------|
| `src/` | Browser core (navigation, layout, snapshot, extraction), MCP server |
| `vfs/` | Sandboxed in-memory filesystem used by MCP sessions |
| `tests/` | Unit, integration and MCP end-to-end tests; `tests/corpus` page corpus and scoreboard; `tests/fingerprint` global-object dumps |
| `xtask/` | `cargo xtask` developer commands (CI; fingerprint dump/diff and WPT, which measure the JavaScript engine) |
| `docs/archive/` | Old status reports, kept for history; not maintained |

## Testing

```bash
cargo test --workspace                                  # everything
cargo test --test mcp_tests                             # MCP end-to-end (spawns the binary)
```

The page corpus (`cargo test --test mcp_tests corpus -- --ignored`), the
fingerprint dump/diff and the WPT runner (`cargo xtask fingerprint-dump`,
`cargo xtask wpt`) measure the JavaScript engine. They are kept but do not run
in CI yet: they return once re-baselined against V8; the committed results
(`tests/corpus/scoreboard.json`, `tests/fingerprint/thalora-boa.json`,
`tests/wpt/results.json`) are the last Boa baselines.

See [docs/TESTING.md](docs/TESTING.md).

## Security

See [docs/SECURITY.md](docs/SECURITY.md). Navigation to private and loopback
addresses is blocked by default. Vault secrets never leave the process
except as form values submitted to a page of the credential's own origin.

## License

MIT; see [LICENSE](LICENSE).

Built on [Tokio](https://tokio.rs),
[reqwest](https://github.com/seanmonstar/reqwest),
[html5ever](https://github.com/servo/html5ever) and
[Taffy](https://github.com/DioxusLabs/taffy).
