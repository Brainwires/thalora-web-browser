# Thalora

[![CI](https://github.com/Brainwires/thalora-web-browser/actions/workflows/ci.yml/badge.svg)](https://github.com/Brainwires/thalora-web-browser/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A web browser written in Rust for AI agents and humans, exposed to agents as an
MCP server. Thalora owns its engine end to end, so it decides what pages can
observe about who is driving it. It does not use Chrome, Chromium or CEF.

## Status (October 2026)

Thalora works today for reading pages and for scripted interaction with
server-rendered and lightly scripted sites. It is **mid re-architecture**; read
[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) before contributing.

What works now:

- MCP server over stdio or streamable HTTP, with page reading (readable
  markdown, links, metadata), web search, named browser sessions, agent
  snapshots with element refs, clicks, typing, forms, waits, console capture
  and an accessibility tree.
- JavaScript on the [Boa](https://github.com/boa-dev/boa) engine (our fork),
  with an event loop for timers, microtasks and fetch/XHR settlement.
- An encrypted credential vault whose secrets are filled into pages without
  ever being returned to the model.
- SSRF protection, per-origin credential filling and a JS security policy for
  agent-injected scripts.

Known gaps, which the re-architecture addresses:

- **Stale reads after interaction.** Snapshots, page text and extraction read
  the HTML captured at load time, not the live DOM, so changes made by page
  JavaScript after a click usually do not appear. The interactive pages in
  `tests/corpus` record this as `known_failing`.
- **JavaScript performance and coverage.** Boa is an interpreter. Large SPA
  bundles often hit the per-evaluation time budget, and many Web APIs are
  partial or stubbed.
- **Rendering.** CSS cascade and layout are hand-rolled and incomplete
  (no `!important`, floats, inline formatting or tables), so screenshots are
  approximate. `getComputedStyle` returns defaults.
- **Input.** Clicks and key presses are dispatched as plain events, not
  trusted pointer/keyboard sequences.
- **Fingerprint.** Compared with Chrome, the global object is missing about a
  thousand interfaces (`cargo xtask fingerprint-diff`).
- **GUI.** The Avalonia desktop app in `gui/` runs its own engine instance and
  its own layout; it cannot yet attach to an agent's session.
- Session persistence (`save_session`) is not implemented.

The target is V8 (through deno_core, with bindings generated from WebIDL),
Stylo + Taffy + Parley + Vello for style, layout and paint, one live DOM that
every reader uses, and a GUI that shows frames painted by the same pipeline
the agent sees. See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

## Build

Requires a recent stable Rust toolchain (edition 2024) and `pkg-config`.
The optional `native` feature (media, WebRTC, page WebAssembly) also needs
FFmpeg development libraries.

```bash
git clone --recursive https://github.com/Brainwires/thalora-web-browser.git
cd thalora-web-browser
cargo build --release          # MCP server binary + FFI library
./target/release/thalora       # MCP over stdio
./target/release/thalora server --transport http --port 8080
```

| Feature | Description |
|---------|-------------|
| `desktop` (default) | MCP server binary, HTTP transport and the FFI library used by the GUI |
| `native` | Adds audio/video, WebRTC and page WebAssembly (wasmtime); may need FFmpeg |
| `wasm-debug` | MCP tools for inspecting and running WebAssembly modules |

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
| `snapshot_url` | Load a page (optionally running its JS) and extract links, images, metadata, custom selectors, structured data and readable markdown, with output-size limits |
| `web_search` | Web search returning title, URL and snippet per result |
| `image_search` | Image search |

### 🧭 Browser sessions & automation (`THALORA_ENABLE_SESSIONS`)
| Tool | Description |
|------|-------------|
| `browser_session_management` | Create, list, inspect and close named browser sessions |
| `browser_navigate_to` | Navigate a session to a URL, optionally running page JavaScript |
| `browser_navigate_back` / `browser_navigate_forward` | Move through session history |
| `browser_refresh_page` | Reload the current page |
| `browser_snapshot` | Compact page outline for agents: interactive elements with refs (`[ref=e12]`), headings, landmarks and text, within a token budget |
| `browser_screenshot` | PNG screenshot from Thalora's current layout code (approximate; images as placeholders) |
| `browser_get_page_content` | Current URL and content of a session |
| `browser_click_element` | Click an element by snapshot ref or CSS selector; submit buttons submit their form and links navigate unless `preventDefault()` is called |
| `browser_type_text` | Type into an input (fires `input`/`change`) |
| `browser_fill_form` | Fill named fields of a form and (by default) submit it, including hidden fields such as CSRF tokens |
| `browser_select_option` | Choose a `<select>` option by value or visible text |
| `browser_check` | Check or uncheck a checkbox or radio button |
| `browser_press_key` | Press a key; Enter in a single-line field submits its form |
| `browser_hover` | Hover an element (mouseover/mouseenter/mousemove) |
| `browser_scroll` | Scroll an element into view or the page by a number of pixels |
| `browser_console_messages` | Console output logged by the page |
| `browser_wait` | Wait for a ref/selector, page text, URL substring or network idle while running timers and fetches |
| `browser_wait_for_element` | Wait until a selector matches |
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
| `cdp_runtime_evaluate` | Evaluate JavaScript in a session's page (default session `"default"`) |

The name is historical. The earlier mock CDP domains were removed; a real CDP
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
| `browser_eval` | Alias for `cdp_runtime_evaluate` |
| `browser_extract` | Alias for `snapshot_url` |
| `browser_search` | `web_search` with sensible defaults |
<!-- tools:end -->

### Examples

```json
{"method": "tools/call", "params": {"name": "snapshot_url", "arguments": {"url": "https://news.ycombinator.com", "extract_readable": true, "format": "markdown"}}}
{"method": "tools/call", "params": {"name": "browser_navigate_to", "arguments": {"url": "https://example.com/login", "session_id": "work"}}}
{"method": "tools/call", "params": {"name": "browser_snapshot", "arguments": {"session_id": "work"}}}
{"method": "tools/call", "params": {"name": "browser_fill_credential", "arguments": {"session_id": "work", "service": "example", "username_ref": "e3", "password_ref": "e4"}}}
{"method": "tools/call", "params": {"name": "cdp_runtime_evaluate", "arguments": {"session_id": "work", "expression": "document.title"}}}
```

## Repository layout

| Path | Contents |
|------|----------|
| `src/` | Browser core (navigation, renderer, snapshot, extraction), MCP server, FFI for the GUI |
| `engines/boa` | Boa fork (git submodule). Frozen: build fixes only until V8 replaces it |
| `engines/thalora-browser-apis` | Web APIs and the DOM arena, bound to Boa |
| `vfs/` | Sandboxed in-memory filesystem used by page-facing storage |
| `gui/` | Avalonia desktop browser (.NET); see [gui/README.md](gui/README.md) |
| `tests/` | Unit, integration and MCP end-to-end tests; `tests/corpus` page corpus and scoreboard; `tests/fingerprint` global-object dumps |
| `xtask/` | `cargo xtask` developer commands (CI, fingerprint dump/diff, GUI screenshots) |
| `docs/archive/` | Old status reports, kept for history; not maintained |

## Testing

```bash
cargo test --workspace                                  # everything
cargo test --test mcp_tests                             # MCP end-to-end (spawns the binary)
THALORA_CORPUS_WRITE=1 cargo test --test mcp_tests corpus   # page corpus + scoreboard
cargo xtask fingerprint-dump && cargo xtask fingerprint-diff tests/fingerprint/chrome-155.json tests/fingerprint/thalora-boa.json
```

See [docs/TESTING.md](docs/TESTING.md).

## Security

See [docs/SECURITY.md](docs/SECURITY.md). Navigation to private and loopback
addresses is blocked by default. Scripts injected by agents run under a
stricter policy than page scripts. Vault secrets never leave the process
except as keystrokes into a page of the credential's own origin.

## License

MIT; see [LICENSE](LICENSE).

Built on [Boa](https://github.com/boa-dev/boa), [Tokio](https://tokio.rs),
[reqwest](https://github.com/seanmonstar/reqwest),
[html5ever](https://github.com/servo/html5ever),
[Taffy](https://github.com/DioxusLabs/taffy),
[wasmtime](https://github.com/bytecodealliance/wasmtime) and
[webrtc-rs](https://github.com/webrtc-rs/webrtc).
