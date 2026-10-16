# Thalora architecture

This document describes where Thalora is and where it is going. It is the
reference for the re-architecture that started in October 2026; update it
when a phase lands or a decision changes.

## Goals and non-goals

- One browser that AI agents (over MCP, later CDP) and humans (the GUI) use
  together, on the same live sessions.
- We own the engine end to end, so we decide what pages can observe. Nothing
  page-visible may reveal that an agent is driving.
- **Non-goal: Chromium, CEF or Blink**, in any form. Using them would give
  up the reason the project exists.
- Pure-Rust where it is mature (style, layout, paint, networking); V8 for
  JavaScript, because no Rust engine can run real-world SPA bundles at
  usable speed.

## Today

```
MCP tool ─▶ BrowserThread (per session) ─▶ HeadlessWebBrowser
                                             ├─ reqwest fetch, cookies, SSRF guard
                                             ├─ Boa context + thalora-browser-apis
                                             │    └─ DomTree arena (live, JS-mutated)
                                             └─ current_content: outerHTML string
                                                  captured once after load
snapshot / page text / readability / a11y / screenshot ─▶ re-parse current_content
GUI (Avalonia) ─▶ its own engine instance via FFI, own C# layout (ControlTreeBuilder)
```

Main problems:

- **Many copies of the DOM.** JavaScript mutates the `DomTree` arena
  (`engines/thalora-browser-apis/src/dom/tree/`), but every other reader
  re-parses an HTML string captured at load (`src/engine/browser/core.rs`,
  `navigation/javascript.rs`). Agent reads go stale after a click.
  `querySelector` serializes and re-parses the tree per call
  (`dom/tree/select.rs`).
- **Boa coupling.** The ~129k lines of `thalora-browser-apis` are written
  directly against Boa internals (~940 references; ~200 intrinsics slots
  patched into our fork). Boa is an interpreter, so large bundles hit the
  per-evaluation time budget.
- **Hand-rolled CSS and two layout engines.** The cascade in
  `src/engine/renderer/css/` ignores `!important` and declaration order;
  layout exists in Rust (Taffy) and again in the GUI's C#. Neither has
  incremental relayout.
- **Untrusted input.** Agent clicks are plain `Event`s, never `isTrusted`.
- **Separate sessions.** The GUI and MCP run separate engines.

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
| 4 | `thalora-v8` + `thalora-webidl`, ported in tiers T0–T3; fingerprint profile; `THALORA_ENGINE=boa\|v8` side by side | V8 ≥ Boa on every tracked WPT directory, ≥98% corpus parity, clean fingerprint diff |
| 5 | GUI becomes a thin shell over Rust-painted frames; one session host; humans can watch or take over agent sessions | Human sees the same pixels as `browser_screenshot` and can hand back |
| 6 | Trusted input through hit testing; incremental snapshot diffs; real CDP WebSocket endpoint; encrypted session persistence | Playwright can drive Thalora with no page-visible automation signal |
| 7 | Delete Boa and `thalora-browser-apis`; long tail (WebGL/WebGPU on wgpu, Web Audio, media, WebRTC with an IP policy, service workers) | — |

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
  painted by Rust.
- Pending (phase 2): deno_core vs bare v8; `DomTree` vs blitz-dom as the
  tree type under `thalora-dom`.

## Measurement

| Tool | What it measures |
|---|---|
| `cargo xtask wpt` | web-platform-tests testharness results per directory, against a pinned WPT commit |
| `cargo test --test mcp_tests corpus` | Page corpus through the real MCP binary, including interactive `steps`; writes `tests/corpus/scoreboard.json` with `THALORA_CORPUS_WRITE=1` |
| `cargo xtask fingerprint-dump` / `fingerprint-diff` | Global-object shape (descriptors, `toString`, enumeration order) against a Chrome reference dump |
| `cargo xtask gui-compare` | Screenshot comparison against Chrome references (moves to headless reftests in phase 3) |

Real-site recordings (HAR replay) are deferred: replaying HTTPS sites
deterministically needs a local MITM proxy design first.
