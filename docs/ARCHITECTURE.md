# Thalora architecture

This document describes where Thalora is and where it is going. It is the
reference for the re-architecture that started in October 2026; update it
when a phase lands or a decision changes.

## Goals and non-goals

- One browser that AI agents (over MCP, later CDP) and humans (a GPUI desktop shell) use
  together, on the same live sessions.
- We own the engine end to end, so we decide what pages can observe. Nothing
  page-visible may reveal that an agent is driving.
- **Non-goal: Chromium, CEF or Blink**, in any form. Using them would give
  up the reason the project exists.
- Pure-Rust where it is mature (style, layout, paint, networking); V8 for
  JavaScript, because no Rust engine can run real-world SPA bundles at
  usable speed.

## Today

**There is no JavaScript engine in the tree.** Boa and `thalora-browser-apis`
(the Web APIs and DOM arena bound to it) were deleted on 2026-11-23, ahead of
V8 being wired in, together with the page-WebAssembly runtime (wasmtime), the
async `webrtc` crate and the media backends that only served those APIs.
Until V8 lands, Thalora is a static-HTML browser:

```
MCP tool ─▶ BrowserThread (per session) ─▶ HeadlessWebBrowser
                                             ├─ reqwest fetch, cookies, SSRF guard,
                                             │    CSP/HSTS/SRI header handling
                                             ├─ current_content: the fetched HTML
                                             └─ filled_values: form values typed by the agent
snapshot / page text / readability / a11y / screenshot ─▶ parse current_content
```

- Navigation fetches the page and its stylesheets and stops there; `<script>`
  elements are not executed (`wait_for_js` is accepted and ignored).
- Interaction is the native default action worked out from the HTML
  (`src/engine/browser/navigation/forms.rs`): typed values are remembered per
  field name and sent on submit, submit buttons submit their form, links
  navigate, checkboxes and radio buttons change state, Enter submits.
  No events are dispatched.
- Everything that needs script fails with the error in
  `engine::JS_UNAVAILABLE` ("JavaScript execution is unavailable: the Boa
  engine was removed; V8 support is in progress"): `cdp_runtime_evaluate`,
  `browser_eval`, `browser_console_messages`, `browser_hover`,
  `browser_scroll`, other key presses, and clicks on elements without a
  default action. The tools stay registered so the tool list is stable.
- `BrowserThread` (one OS thread per browser) is kept: a V8 isolate is as
  thread-bound as the Boa heap was.

Main problems:

- **No JavaScript**, so client-rendered pages show only their static shell.
- **Hand-rolled CSS.** The cascade in `src/engine/renderer/css/` ignores
  `!important` and declaration order; layout (Taffy) has no incremental
  relayout.
- **Untrusted input.** There is no event dispatch at all yet; trusted input
  comes with hit testing (phase 6).
- **No desktop UI.** The Avalonia GUI was removed; the desktop UI moves to
  GPUI ([GPUI_PLAN.md](GPUI_PLAN.md)).

## Target

```
              ┌──────────── BrowserThread (one per browser/tab) ─────────────┐
 MCP tools ──▶│  V8 isolate (deno_core)  ◀─bindings─▶  DomTree (single source)│
 GUI input ──▶│        │ HTML event loop: tasks → microtasks → render steps    │
 CDP endpoint │        ▼                                  │ dirty marks        │
              │  Stylo (cascade, invalidation) → Taffy+Parley (layout) → Vello │
              └───────────────┬──────────────────────────────┬───────────────┘
                     snapshot/a11y/text read DOM+layout     frames (RGBA / GPU)
                     directly (no HTML re-parse)            → GUI shell, screenshots
```

New crates:

| Crate | Purpose |
|---|---|
| `engines/thalora-dom` | Engine-neutral DOM: tree, events and dispatch, mutation journal, ranges, tree walkers, form-control state |
| `engines/thalora-net` | fetch/XHR/WebSocket/EventSource, CORS/CSP/SSRF, cookies, module fetching |
| `engines/thalora-render` | Stylo + Taffy + Parley + Vello, hit testing, screenshots |
| `engines/thalora-webidl` | Vendored WebIDL, Chrome member-order overlay, codegen to V8 templates and engine-neutral `*Impl` traits |
| `engines/thalora-v8` | deno_core runtime, generated bindings, HTML event loop, isolated worlds for agent scripts, inspector |

Principles:

- Logic lives in engine-neutral crates; JS bindings are generated, not
  hand-written.
- A page sees an API only if it is real, or if a **fingerprint profile**
  (`profiles/chrome-<ver>.toml`) explicitly declares it. Silent fakes (for
  example no-op WebGL) are detectable and get deleted.
- No phase lands without measurement: WPT, the page corpus, the fingerprint
  diff and reftests.

## Phases

| Phase | Content | Exit criterion |
|---|---|---|
| 0 | Cleanup (wasm target, mock CDP, regex transpiler, AI memory → vault only, stale docs) and baselines (WPT runner, interactive corpus, fingerprint dump/diff) | Baselines recorded; CI reports them |
| 1 | `thalora-dom`; every reader uses the live tree; `selectors` crate matching; mutation journal and dirty bits; NodeId-backed snapshot refs; real `PageEngine` trait | Stale-after-click corpus pages pass; no HTML re-parse on snapshot or extraction paths |
| 2 | Spikes: deno_core cleanliness; blitz-dom vs `DomTree`; wrapper lifetime; event loop | Written decisions here |
| 3 | `thalora-render`: Stylo, Taffy + Parley, Vello; hit testing, geometry APIs, real `getComputedStyle`; screenshots from the real pipeline | Reftests above baseline |
| 4 | `thalora-v8` + `thalora-webidl`, ported in tiers T0–T3; fingerprint profile. (Boa is already gone, so there is no side-by-side run; the last Boa results are the committed baselines) | V8 ≥ the Boa baseline on every tracked WPT directory, ≥98% corpus parity, clean fingerprint diff |
| 5 | GPUI desktop shell over Rust-painted frames ([GPUI_PLAN.md](GPUI_PLAN.md)); one session host; humans can watch or take over agent sessions | Human sees the same pixels as `browser_screenshot` and can hand back |
| 6 | Trusted input through hit testing; incremental snapshot diffs; real CDP WebSocket endpoint; encrypted session persistence | Playwright can drive Thalora with no page-visible automation signal |
| 7 | Long tail (WebGL/WebGPU on wgpu, Web Audio, media, WebRTC with an IP policy, service workers). Deleting Boa and `thalora-browser-apis` was done early, on 2026-11-23 | — |

## Decisions log

- 2026-10-15: V8 via deno_core (bare `v8` as a fallback if deno_core leaks
  Deno-isms to pages). Boa is frozen: build fixes only.
- 2026-10-15: No Chromium/CEF, permanently.
- 2026-10-15: The wasm (web-sys) build target is dropped. wasmtime for page
  WebAssembly stays.
- 2026-10-15: AI memory is reduced to the credential vault and
  `browser_fill_credential`. The vault lives on the real filesystem; the
  page-facing `vfs` is in-memory and must not hold host state.
- 2026-10-15: The mock CDP domains are removed; `cdp_runtime_evaluate` stays
  until the real CDP endpoint (phase 6) replaces it.
- 2026-10-15: The GUI stays Avalonia, reduced to a shell that shows frames
  painted by Rust. *Superseded 2026-11-23:* the Avalonia GUI is removed and
  the desktop UI moves to GPUI; see [GPUI_PLAN.md](GPUI_PLAN.md).
- 2026-11-23: Boa is deleted now rather than in phase 7, together with
  `thalora-browser-apis`, wasmtime (page WebAssembly needs a JS engine), the
  async `webrtc` crate (WebRTC returns through the deno fork's `ext/webrtc`)
  and the `wasm-debug` MCP tools. This leaves `thalora` without JavaScript
  execution until V8 (the Brainwires deno fork, through `deno_core`) is wired
  in. Tools that need script stay registered and return a clear error. This
  supersedes "wasmtime for page WebAssembly stays" above.
- Pending (phase 2): deno_core vs bare v8; `DomTree` vs blitz-dom as the
  tree type under `thalora-dom`.

## Measurement

These all measure the JavaScript engine, so none of them runs in CI while
there is none; the committed results are the last Boa baselines and the tools
return with V8.

| Tool | What it measures |
|---|---|
| `cargo xtask wpt` | web-platform-tests testharness results per directory, against a pinned WPT commit |
| `cargo test --test mcp_tests corpus -- --ignored` | Page corpus through the real MCP binary, including interactive `steps`; writes `tests/corpus/scoreboard.json` with `THALORA_CORPUS_WRITE=1` |
| `cargo xtask fingerprint-dump` / `fingerprint-diff` | Global-object shape (descriptors, `toString`, enumeration order) against a Chrome reference dump |

Real-site recordings (HAR replay) are deferred: replaying HTTPS sites
deterministically needs a local MITM proxy design first.
