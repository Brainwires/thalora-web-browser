# A3 design note: page JavaScript on V8 (Brainwires deno fork)

Status: **proposal**, for review by the denext PM session before any fork code lands. Targets the runtime release after `3.0.0-denext.30`.

## Goal

Thalora runs page JavaScript on V8 again, after Boa's removal (Train 2, #20). The same code must work in two hosts:

1. **Standalone `thalora`** (the headless MCP browser), which links the fork directly.
2. **Inside a Denext app (A5)**, where a laufey host has loaded `libdenort` and V8 already lives there. Thalora must not bring a second V8 into that process.

## Constraints

- One V8 per process: the Brainwires `rusty_v8` pin (`156.0.0+denext.6`, custom libc++ `std::__Cr`). Never upstream `rusty_v8`.
- Page JS is **web content, not Deno**: no `Deno.*`, no Node compat, no permissions-bypassing ops. It gets web-platform APIs only.
- Fingerprint control is the reason Thalora exists, so every global, property descriptor and error message seen by pages is ours to decide (the fingerprint diff tooling comes back to CI on V8).
- Fork conventions (from the PM session): `ext/<name>/`, crate `deno_<name>`, `extension!`, `op_<name>_<verb>`, numeric-prefixed JS, optional cargo feature, `thalora/<topic>` branches, design note first.

## Proposal

### Three layers

| Layer | Where | What |
|---|---|---|
| `thalora-dom` | thalora repo, new crate, engine-neutral | The live DOM: an html5ever-built mutable tree with node ids, mutation API, event-target registry, selector matching (`selectors`), later Stylo style data. No V8 types. |
| `deno_web_page` | **fork**, `ext/web_page/`, optional feature `web-page` | The V8 binding of `thalora-dom`: ops `op_web_page_*` plus JS (`00_node.js`, `01_element.js`, `02_document.js`, `10_events.js`, …) implementing DOM/HTML/CSSOM interfaces over the ops, WebIDL conversions via `deno_webidl`. |
| Page runtime | fork crate API, used by both hosts | Builds a *page context*: a `deno_core::JsRuntime` with only web extensions (`deno_webidl`, `deno_console`, `deno_url`, `deno_web`, `deno_fetch`, `deno_websocket`, `deno_webstorage`, `deno_crypto`, `deno_webrtc`, `deno_canvas`, `deno_web_page`) and **no** `deno_runtime` worker, `Deno` namespace or Node. |

`deno_web_page` depends on `thalora-dom` as a git dependency (pinned rev), behind the `web-page` feature, so libdenort's default size doesn't change.

### Why the binding lives in the fork

- In A5 the page contexts must run on libdenort's V8. The only way to put ops into isolates that libdenort creates is to compile them into libdenort. A fork extension behind a feature does that, and laufey's host reaches it through the vtable the same way B1's native tree does.
- Standalone `thalora` links the same fork crates, so there is one implementation, not two.

### Page contexts

- One `JsRuntime` per top-level page (frames: one realm per frame on the same isolate, later), owned by Thalora's existing `BrowserThread` (V8 isolates are thread-bound; the thread model already fits).
- Script fetching, CSP, SRI and loopback policy stay in Thalora (Rust); the page runtime gets scripts as source plus origin. Network ops in `deno_fetch`/`deno_websocket` go through a Thalora-provided permission/policy object (`PermissionsContainer`-shaped, so no new permission kinds), which enforces Thalora's origin and loopback rules and its HTTP fingerprint client.
- Timers and the event loop: deno_core's event loop, pumped by the browser thread; Thalora's "wait for idle" maps to polling the loop until no pending ops/timers (with a budget).

### Bindings: hand-written first, WebIDL-generated next

- Start hand-written for the core (Node, Element, Document, Text, events, `querySelector*`, `innerHTML`/`textContent`, attributes, classList, forms) to get the MCP tools (eval, click, type, wait) back quickly.
- Then a generator from WebIDL (the `webidl` crate or the WPT `interfaces/` files) emits the JS interface shells with correct descriptors, so the fingerprint diff against Chrome stays near-empty by construction.

### A5 path (later, not in this change)

- libdenort with `web-page` enabled exposes "create page context / load / dispatch input / get display list" through new laufey vtable entries (an API version bump, like B1).
- The GPUI host paints the display list (A2). Same split as B1: JS and DOM in libdenort, pixels in the host.

## Alternatives considered

- **Binding in the thalora repo only** (a `deno_core::extension!` in thalora, linking the fork's deno_core): simplest for standalone, but can't run inside libdenort's isolates in A5. Could be a starting point if the fork placement is rejected; moving it later is mostly a file move.
- **Run pages in a `deno_runtime` WebWorker**: brings the `Deno` namespace and permission model pages must never see; stripping it is more work than composing ext crates.
- **A second V8 in the host for A5**: rejected (one V8 per process).

## Delivery

1. `thalora-dom` crate in this repo (no fork changes).
2. Fork branch `thalora/web-page`: `ext/web_page` + the page-context builder, behind `web-page`; DENEXT.md item `5NN.`; tests in `tests/unit/web_page_test.ts` and Rust unit tests. No CI dispatch without telling the PM session.
3. Thalora links the fork (deno_core 0.413 + the ext crates, `web-page` on) and restores the JS MCP tools; the corpus, fingerprint and WPT baselines return to CI.

## Open questions for the PM session

1. OK with `ext/web_page` / `deno_web_page` and a git dependency from the fork on `thalora-dom`? (Alternative: vendor `thalora-dom` into the fork as `ext/web_page/dom/`.)
2. Is exposing a "page context builder" as a public Rust API of the fork acceptable, or should it live in `deno_web_page` itself?
3. Any objection to page contexts composing `deno_fetch`/`deno_websocket` with a Thalora-supplied permissions object rather than the CLI's?
