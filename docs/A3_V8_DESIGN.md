# A3 design note: page JavaScript on V8 (Brainwires deno fork)

Status: **accepted with conditions** by the denext PM session (2026-11-23, answers below). The security section must be agreed before fork code merges. Targets the runtime release after `3.0.0-denext.30`.

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
| Page runtime | inside `deno_web_page`: a small pub API, `PageRuntime::new(options)` | Builds a *page context*: a `deno_core::JsRuntime` with only web extensions (`deno_webidl`, `deno_console`, `deno_url`, `deno_web`, `deno_fetch`, `deno_websocket`, `deno_webstorage`, `deno_crypto`, `deno_webrtc`, `deno_canvas`, `deno_web_page`) and **no** `deno_runtime` worker, `Deno` namespace or Node. |

`deno_web_page` depends on `thalora-dom` as a git dependency pinned by `rev` (like `rtc`), behind the `web-page` feature, so libdenort's default size doesn't change. Conditions: `thalora-dom` lives in a Brainwires-owned repo, is MIT/Apache-2.0, has its own green CI, and passes the fork's cargo-audit/deny. Cargo fetches and locks optional git deps even with the feature off, so its dependency tree stays lean (html5ever, selectors, and little else).

### Why the binding lives in the fork

- In A5 the page contexts must run on libdenort's V8. The only way to put ops into isolates that libdenort creates is to compile them into libdenort. A fork extension behind a feature does that, and laufey's host reaches it through the vtable the same way B1's native tree does.
- Standalone `thalora` links the same fork crates, so there is one implementation, not two.

### Page contexts

- One `JsRuntime` per top-level page (frames: one realm per frame on the same isolate, later), owned by Thalora's existing `BrowserThread` (V8 isolates are thread-bound; the thread model already fits).
- Script fetching, CSP, SRI and loopback policy stay in Thalora (Rust); the page runtime gets scripts as source plus origin.
- Network permissions are a real `deno_permissions::PermissionsContainer` built from parsed descriptors (an allowlist), so `deno_fetch`/`deno_websocket`'s existing `check_net_url` paths do the enforcing. Nothing reimplements or bypasses the checks.
- Page fetches go through a client built with `denyAddresses` (in .30; `ext/fetch/deny_addresses.rs`), denying loopback, private, link-local and cloud-metadata addresses by default. Thalora's `test-hooks` loopback override is the only way to lift it, and only in test builds.
- Timers and the event loop: deno_core's event loop, pumped by the browser thread; Thalora's "wait for idle" maps to polling the loop until no pending ops/timers (with a budget).

### Bindings: hand-written first, WebIDL-generated next

- Start hand-written for the core (Node, Element, Document, Text, events, `querySelector*`, `innerHTML`/`textContent`, attributes, classList, forms) to get the MCP tools (eval, click, type, wait) back quickly.
- Then a generator from WebIDL (the `webidl` crate or the WPT `interfaces/` files) emits the JS interface shells with correct descriptors, so the fingerprint diff against Chrome stays near-empty by construction.

### A5 path (later, not in this change)

- Pages run in a separate, confined page process (see Security). It's built on libdenort with `web-page` enabled and driven over IPC: create page, load, dispatch input, get display list.
- The laufey host exposes that through new vtable entries (an API version bump, like B1), and the GPUI host paints the display list (A2). JS and DOM stay out of the host, as in B1.

## Security: page JS is hostile

Page contexts run untrusted web JavaScript. What isolates them:

| Boundary | Status | Notes |
|---|---|---|
| No privileged ops | by construction | Page runtimes compose only web exts plus `deno_web_page`. No `Deno`/Node namespace and no fs, env, ffi, run, net-listen, kv or os ops are registered, so none are reachable from JS. A test asserts the op table of a page runtime against an allowlist. |
| Network | enforced | A `PermissionsContainer` allowlist plus `denyAddresses` (above). |
| Storage | origin-scoped | `deno_webstorage` and the cache are keyed by page origin, under a per-profile directory owned by Thalora (std::fs). |
| Isolation between pages | per-page isolate | One isolate per top-level page. Cross-origin frames get their own isolate (not just a realm) when frames land. |
| Memory / CPU per page | to build | The fork's `--max-memory`/`--max-cpu-time` end the **whole process**, so they don't fit. `PageRuntime` sets per-isolate heap limits (`CreateParams` heap limits plus a near-heap-limit callback that terminates the page), and a watchdog calls `terminate_execution` on long tasks. |
| V8 sandbox | **off today** | The fork's `deno_v8` builds without `v8_enable_sandbox` (only `simdutf`). Turning it on needs the sandbox variant of the prebuilt rusty_v8 archive and changes ArrayBuffer allocation for every embedder, so it's a fork-wide decision, not part of this change. Without it, a V8 memory-corruption bug in page JS is a full compromise of the process. |

Because of that last row:

- **Standalone `thalora`:** the process is the boundary. It holds no app state beyond the profile, and it should run under the fork's `deno_sandbox` confinement (Landlock + seccomp on Linux, Seatbelt on macOS) with a filesystem view limited to its profile directory. Tracked as an A3 follow-up.
- **A5 (inside a Denext app): pages run in a separate process (decided).** Same-process isolates next to the app's backend inside libdenort are too weak a boundary for hostile web content while the V8 sandbox is off. The laufey host spawns a Thalora page process (libdenort with `web-page`, confined by `deno_sandbox`) and talks to it over IPC: input in, display lists out. This lines up with the board's "process-isolated workers after 5.0". The in-process `PageRuntime` stays the building block either way; only where it runs changes.
- In-process A5 is off the table unless the PM session's V8-sandbox evaluation lands.

## Alternatives considered

- **Binding in the thalora repo only** (a `deno_core::extension!` in thalora, linking the fork's deno_core): simplest for standalone, but can't run inside libdenort's isolates in A5. Could be a starting point if the fork placement is rejected; moving it later is mostly a file move.
- **Run pages in a `deno_runtime` WebWorker**: brings the `Deno` namespace and permission model pages must never see; stripping it is more work than composing ext crates.
- **A second V8 in the host for A5**: rejected (one V8 per process).

## Delivery

1. `thalora-dom` crate in this repo (no fork changes).
2. Fork branch `thalora/web-page`: `ext/web_page` + the page-context builder, behind `web-page`; DENEXT.md item `5NN.`; tests in `tests/unit/web_page_test.ts` and Rust unit tests. No CI dispatch without telling the PM session.
3. Thalora links the fork (deno_core 0.413 + the ext crates, `web-page` on) and restores the JS MCP tools; the corpus, fingerprint and WPT baselines return to CI.

## Answers from the PM session (2026-11-23)

1. Git dependency on `thalora-dom`, pinned by `rev`, no vendoring, on the conditions above.
2. The builder lives inside `deno_web_page` as `PageRuntime::new(options)`. A5's vtable entries call it; nothing else depends on it.
3. Thalora-supplied permissions only as a real `PermissionsContainer` from parsed descriptors, plus `denyAddresses` by default.
4. Security: (a)-(f) accepted as written. **Decided: A5 runs pages in a separate, confined page process over IPC; no in-process page isolates next to the app backend.** Turning the V8 sandbox on fork-wide is not on the table: it breaks Node-API addons that hand out external ArrayBuffers (which packaged denext apps support), costs performance, and needs the sandbox archive for every target. The PM session tracks it as a separate evaluation; in-process A5 is reconsidered only if that lands.

`thalora-dom` notes: it depends on `selectors` (MPL-2.0), which the fork's `Cargo.lock` already contains.
