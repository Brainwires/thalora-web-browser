# A3 step 3: linking the fork's V8 — integration map

Working notes for delivery step 3 of [A3_V8_DESIGN.md](A3_V8_DESIGN.md)
("Thalora links the fork and restores the JS MCP tools"). Sections 1-8 are
the map drawn before the link (line numbers against `origin/main` at 412eba2,
2026-11-24) and are kept as written; **section 9 is what was done**, and
where it differs from the proposals in sections 5 and 6 it wins.

## 1. Where JavaScript is refused today

`src/engine/mod.rs:23` defines `JS_UNAVAILABLE`, `src/engine/mod.rs:27`
`js_unavailable()`. Every use:

| Site | Reached from | Refuses / degrades | Page-runtime capability needed |
|---|---|---|---|
| `src/protocols/browser_tools/handlers/javascript.rs:32` (`handle_evaluate_javascript`) | MCP `cdp_runtime_evaluate` (`src/protocols/mcp_server/tools/routing.rs:29`), `browser_eval` (`routing.rs:99`) | Always errors after validating `expression` / `session_id`. Does **not** go through `HeadlessWebBrowser::execute_javascript`. | **eval-to-JSON**: evaluate an expression in the page realm, await a returned promise, serialize to `serde_json::Value`, map exceptions to an error. Must run as a `run_in` job on the session's `BrowserThread`. |
| `src/engine/browser/core.rs:219` (`HeadlessWebBrowser::execute_javascript`, returns `Result<String>`) | Only the session-process protocol: `BrowserCommand::ExecuteJs` (`src/main.rs:409`) → `BrowserSessionHandler::execute_javascript` (`src/main.rs:566-570`) | Always errors | Same eval, string result. Natural home for the shared eval used by both paths. |
| `src/engine/browser/navigation/forms.rs:37` (`type_text_into_element`, unnamed field) | `browser_type_text` (`handlers/interaction.rs:190`), `browser_fill_form` (`interaction.rs:276`), `browser_fill` (`interaction.rs:396`), session protocol (`src/main.rs:602`) | An `<input>`/`<textarea>` without `name` | **set value**: set `.value` on the live node and dispatch `input`/`change` (and key events if typing is emulated). |
| `forms.rs:366` (`text_field`, not `input`/`textarea`) | same callers as above, plus `fill_secret` (`forms.rs:240`, used by `browser_fill_credential` in `src/protocols/mcp_server/credential_fill.rs:116,121`; it maps the error to "Could not fill the field") | contenteditable, custom elements | **set value / type** on arbitrary focused element. |
| `forms.rs:202`, `forms.rs:209` (`perform_action` `SelectOption` / `SetChecked`, unnamed control) | `browser_select_option`, `browser_check` (`handle_element_action`, `interaction.rs:431`, call at `interaction.rs:503`) | Unnamed `<select>` / checkbox | **set value / checked** + `input`/`change` events. |
| `forms.rs:229` (`PressKey` other than Enter-in-single-line-field) | `browser_press_key` (`interaction.rs:503`) | Every key except implicit-submit Enter | **press_key**: `keydown`/`keypress`/`keyup` on the focused (or selected) element, then default action. |
| `forms.rs:232` (`Hover`, `Scroll`) | `browser_hover`, `browser_scroll` (`interaction.rs:503`) | Always | **hover** (`pointerover`/`mouseover`/`mouseenter`/`mousemove`), **scroll** (`window.scrollBy` + `scroll` event); needs layout coordinates from the renderer. |
| `forms.rs:313` (`click_element`, no default action and not a named checkbox/radio) | `browser_click_element` / `browser_click` (`interaction.rs:97`), session protocol `BrowserCommand::Click` (`src/main.rs:431`, `590`) | Any element whose behaviour is a script handler | **click on node**: dispatch `pointerdown`/`mousedown`/`pointerup`/`mouseup`/`click`, honour `preventDefault`, then run the existing default-action logic (`click_default_action`, `forms.rs:612`). |
| `src/engine/browser/navigation/javascript.rs:200` (debug log) | `navigate_to_with_js_option(.., wait_for_js)` (`javascript.rs:21`) | `wait_for_js` is ignored | **wait-for-idle** after load: run scripts, pump the event loop until no pending ops/timers (budgeted). |
| `src/protocols/browser_tools/handlers/interaction.rs:535` (`handle_console_messages`) | `browser_console_messages` (`routing.rs:87`) | Always errors | **console capture**: buffered `console.*` records (level, text, timestamp) per page, drained/returned by this tool. |
| `interaction.rs:667` (`handle_wait_for_element`, message only) | `browser_wait_for_element` | Returns `found: false` immediately with the JS note | **wait-for-idle / mutation polling** until selector matches or timeout. |
| `src/main.rs:148` | startup log | — | Remove when V8 lands. |
| `src/main.rs:565` | doc comment | — | Update. |
| `forms.rs:850` | unit test asserting the error | — | Rewrite as a real-JS test. |

Not using the constant but equally JS-dependent:

- `javascript.rs:217-235` `wait_for_condition`: `NetworkIdle` is hard-wired to
  `Ok(true)` (`javascript.rs:226`); `Text` / `UrlContains` check once.
  Reached from `browser_wait` (`interaction.rs:539`, call `interaction.rs:604`).
  Needs **wait-for-idle** (event loop + in-flight `fetch` count).
- `javascript.rs:240` `wait_for_element` ignores `_timeout_ms` (single check).
- `src/protocols/browser_tools/handlers/snapshot.rs:143` `resolve_ref_param`
  and `src/engine/browser/snapshot.rs:145` `RefTable::resolve`: refs resolve to
  a structural CSS selector re-checked against the current HTML; with a live
  DOM the ref table should key on `thalora_dom::NodeId` instead.

## 2. Where the page's document lives today

- The page is a **string**: `HeadlessWebBrowser::current_content: String`
  (`src/engine/browser/core.rs:22`), set from the response body in
  `navigate_to_with_js_option` (`javascript.rs:133`) and `navigate_internal`
  (`javascript.rs:268`). `current_url` is `core.rs:21`.
- There is **no persistent DOM**. Every operation re-parses the string with
  `scraper::Html::parse_document` (scraper 0.24, html5ever 0.27 —
  `Cargo.toml:39-40`); 55 call sites across `src/`. Examples: `forms.rs:348`
  (`has_attr`), `forms.rs:358` (`text_field`), `javascript.rs:243`
  (`wait_for_element`), `snapshot.rs:145`.
- Static form interaction: typed/selected/checked values are recorded per field
  `name` in `filled_values: HashMap<String,String>` keyed to
  `filled_values_url` (`core.rs:49-50`; `record_filled_value`, `forms.rs:253`).
  `submit_form_at_index` (`forms.rs:81`) builds the browser's entry list from
  the static form controls, overlays `filled_values` then explicit
  `form_data`, and submits over reqwest. `click_element` (`forms.rs:278`)
  computes the default action from static HTML (`click_default_action`,
  `forms.rs:612`): submit button → submit, link → navigate, checkbox/radio →
  toggle recorded value. The HTML string is never mutated.
- `FormAnalyzer` (`core.rs:27-28`, run at `javascript.rs:171`) pre-analyses
  forms (target=_blank detection).
- Renderer/screenshot consume the string too: `screenshot_png`
  (`core.rs:224`) → `compute_page_layout_with_css(&current_content, ..,
  &external_stylesheets)`; the accessibility tree
  (`routing.rs:110` `get_accessibility_tree`) and `page_state`
  (`browser_tools/core.rs:34`) read `get_current_content()`.
- `crates/thalora-dom` (the live DOM from A3 step 1, html5ever **0.40.1** /
  selectors 0.43, `crates/thalora-dom/Cargo.toml:15-16`) is a workspace member
  but **not yet a dependency of the `thalora` crate** — nothing in `src/` uses
  it. Step 3 has to make the `thalora_dom::Document` owned by the page runtime
  the source of truth and serialize it (`outer_html`) wherever `current_content`
  is read, or move readers onto the DOM. Two html5ever versions (0.27 via
  scraper, 0.40 via thalora-dom) will coexist until scraper users migrate.

## 3. Threading model (where a `!Send` V8 runtime lives)

- `src/engine/browser/session_thread.rs`: `BrowserThread` = one OS thread per
  browser (`thalora-<name>`, 16 MiB stack, `session_thread.rs:56,60-80`) with a
  current-thread tokio runtime + `LocalSet` (`:67-78`). `run_loop` (`:206`)
  creates `HeadlessWebBrowser::new()` (an `Rc<Mutex<_>>`) on that thread and
  runs `Job`s (`FnOnce(BrowserHandle) -> LocalBoxFuture`) one at a time to
  completion; the browser is dropped on the same thread. The module doc
  (`:1-12`) already anticipates a thread-bound JS heap.
- Callers: MCP sessions (`src/protocols/browser_tools/core.rs:81`
  `BrowserThread::spawn(session_id)`; jobs via `run_in`, `core.rs:23`), the
  session process (`src/main.rs:325`), and per-search-engine browsers
  (`src/protocols/mcp_server/scraping/search/mod.rs:20`).
- So the `deno_core::JsRuntime` (from `PageRuntime::new`) can be a field of
  `HeadlessWebBrowser` (or a sibling owned by `run_loop`), created/replaced on
  navigation, on the browser thread. Caveats: jobs hold the browser's
  `MutexGuard` across `.await`, so event-loop pumping happens only inside a
  job; there is no background pumping between jobs (timers/fetches stall
  between MCP calls unless `run_loop` polls the runtime while idle — e.g.
  `select!` on `rx.recv()` and `JsRuntime::run_event_loop`). Each
  `BrowserThread` gets its own isolate; V8 platform init must happen once per
  process before the first thread creates one.

## 4. Script loading today

- **No `<script>` is fetched or collected.** `navigate_to_with_js_option`
  only fetches stylesheets (`fetch_all_stylesheets`, `javascript.rs:401`,
  called at `javascript.rs:139` and `:273`). The only `<script>` reads are
  JSON-LD extraction (`src/engine/browser/scraper.rs:169`) and skipping script
  text (snapshot, page_text, readability, layout).
- Ready-made pieces the script loader can reuse:
  - `resolve_script_url` (`javascript.rs:324`) — resolves relative /
    protocol-relative URLs (currently used for stylesheets).
  - CSP: `CspPolicy` parsed per page (`javascript.rs:76`, stored in
    `core.rs:38`) with `allows_inline_script(nonce, hash)` (`csp.rs:117`) and
    `allows_external_script(url, page_url)` (`csp.rs:157`); both are only
    exercised by unit tests today.
  - SRI: `sri::verify_integrity` (`sri.rs:13`), used for stylesheets at
    `javascript.rs:528`.
  - Mixed-content blocking and the stylesheet fetch path (`javascript.rs:350`
    / `:401-545`) incl. `ResourceCache` (`types.rs:147`, documented as
    "stylesheets and scripts", only stylesheets use it).
  - The SSRF-safe reqwest client (`core.rs:58-80`, `PublicOnlyResolver`).
- Per the design note, script fetching/CSP/SRI stays in Rust and the page
  runtime receives source + origin; a new `fetch_page_scripts` beside
  `fetch_all_stylesheets` (document order, `async`/`defer`/`type=module`,
  nonce) is the missing piece.

## 5. Cargo: `[patch.crates-io]` Thalora's root `Cargo.toml` will need

From the fork's root `Cargo.toml` (Brainwires/deno, `thalora/web-page`
checkout at 91a9363, lines 784-802). `[patch]` sections of a git dependency's
workspace are ignored by cargo, so Thalora must repeat the ones it needs.

```toml
[patch.crates-io]
v8 = { git = "https://github.com/Brainwires/rusty_v8", rev = "f72a5f53280688d3101ab6f82e4f7b5ccf528ecd" }
```

`v8` is required: the fork's `deno_v8` (`libs/deno_v8`, the workspace's `v8`
alias, `Cargo.toml:123`) depends on the crates.io `v8`, which must resolve to
Brainwires/rusty_v8 `156.0.0+denext.6` (fork `Cargo.lock:12865-12867`). The
release tag `v156.0.0-denext.6` in `scripts/rusty_v8.sha256` must match; the
fetch script enforces that once Cargo.lock has a `v8` package.

The other fork patches point at fork-local `libs/vendor/*` paths, which do not
exist in Thalora:

| Patch (fork l.) | Vendored | Needed by the page runtime? | Proposal |
|---|---|---|---|
| `url` (801) | 2.5.7 + RUSTSEC-2026-0592 backport | Yes, `url` is everywhere (deno_core, deno_permissions, Thalora). Thalora's `Cargo.lock` has url **2.5.8**, which is affected by RUSTSEC-2026-0592. | The fork pins 2.5.7 only because 2.5.9's `^` percent-encoding breaks deno_semver/deno_graph specifiers; none of the page-runtime crates (libs/core, ext/{webidl,console,url,web,fetch,websocket,webstorage,crypto,webrtc,canvas}, runtime/permissions) depend on those directly. Preferred: take crates.io url >= 2.5.9 (`cargo update -p url`) and verify with `cargo tree -i deno_semver` / `-i deno_graph` that neither is in the tree. Fallback: `url = { git = "https://github.com/Brainwires/deno", rev = "<fork rev>" }` (cargo finds `libs/vendor/url` by package name), which pins 2.5.7 — only works if nothing in Thalora requires url >= 2.5.8. |
| `sys_traits` (793) | 0.1.28 + Windows DOS-device paths | deno_core, deno_permissions, ext/fetch use it; the fix matters only for Windows filesystem access (Node compat). | Skip for now (Thalora pages have no fs). If needed: same git+rev form. |
| `denokv_sqlite` (788) | rusqlite 0.41 | No (ext/kv not in the page set) | Skip. |
| `dprint-core` (789) | — | No (CLI formatter) | Skip. |
| `tokio-postgres` (794) | — | No (node/ext only) | Skip. |

Other fork settings the link step must mirror:

- `deno_core` features `reactor-tokio`, `v8_use_custom_libcxx` (fork
  `Cargo.toml:173`): the prebuilt archive is built against the fork's
  `std::__Cr` libc++, so the custom-libcxx feature has to be on in Thalora's
  graph too (it comes with `deno_web_page`'s own deps if the fork sets it
  there; check).
- The fork's `.cargo/config.toml` `[env]` `MACOSX_DEPLOYMENT_TARGET = "12.0"`
  and the macOS `-fixup_chains` linker args are fork-binary choices, not link
  requirements.

## 6. The `RUSTY_V8_MIRROR` guard (not active)

The fork's `.cargo/config.toml` forces a broken mirror so a build without a
verified archive fails instead of silently downloading an upstream release:

```toml
[env]
RUSTY_V8_MIRROR = { value = "denext-refused-unverified-v8/run-scripts/fetch_rusty_v8.py-first/{file}", force = true }
RUSTY_V8_MIRROR_FALLBACK = { value = "0", force = true }
```

Decision: **not added yet.** With no `v8` crate in the graph it changes
nothing, but it is meaningless until the dependency lands and would make local
`cargo build` fail without explanation for anyone who has the variables unset
once it does. Add it in the same commit as the `deno_web_page` dependency and
the `v8` patch, with a README note: `eval "$(python3 scripts/fetch_rusty_v8.py
--print-env)"` before building. `force = true` only affects the two mirror
variables; `RUSTY_V8_ARCHIVE` / `RUSTY_V8_SRC_BINDING_PATH` from the fetch
script still win (the build script never consults the mirror when they are
set). Note `fetch_rusty_v8.py` refuses to run if `RUSTY_V8_MIRROR` is in its
own environment; cargo's `[env]` applies only to processes cargo spawns, so
the two do not collide.

## 7. What this change already does

- `scripts/fetch_rusty_v8.py`: port of the fork's
  `.github/denext-runtime/fetch_rusty_v8.py` (216c1a6). Differences: lives in
  `scripts/` (Thalora's xtask is Rust and would need an HTTP client and sha2
  for no gain; CI runs Python on ubuntu-latest), `--target` defaults to
  `rustc -vV`'s host, `--dest` defaults to `target/rusty_v8`, and while
  Cargo.lock has no `v8` package the tag check and `--seed-dir` are skipped
  with a note.
- `scripts/rusty_v8.sha256`: the fork's manifest verbatim, with a provenance
  header.
- `.github/workflows/ci.yml`: a "Fetch prebuilt V8" step in Check, Clippy,
  Test and Documentation (all GitHub-hosted `ubuntu-latest`), after
  rust-cache restore, with `--dest $RUNNER_TEMP/rusty_v8 --seed-dir
  target/debug --github-env`. Until the dependency lands it downloads and
  verifies ~41 MB per job and exports unused variables.

## 8. Update: the current page is a live `thalora_dom::Document`

Sections 1-2 describe `origin/main`; on this branch the page model changed:

- `HeadlessWebBrowser::current_content` / `current_url` / `filled_values`
  are gone. The page is `page: PageDom` (`src/engine/browser/page_dom.rs`):
  `load(url, html)` on navigation / submit / history, `with_doc(|&Document|)`,
  `with_doc_mut(|&mut Document|)`, `url()`, `is_empty()`, `version_key()`
  (URL + source hash, keys snapshot refs), `to_html()` (serialization; what
  `get_current_content()` returns), and a never-serialized secret-value side
  table for `browser_fill_credential`. When `deno_web_page` lands, `PageDom`
  wraps the `PageRuntime` (`with_doc` borrows its `Rc<RefCell<Document>>`,
  `load` hands it the HTML); callers stay as they are.
- Ported to the DOM: forms (`navigation/forms.rs`: fill/select/check/click
  write `value`/`checked`/`selected` attributes and `<textarea>` text;
  submission builds the entry list from the DOM), snapshot + ref resolution,
  accessibility tree, visible text, `WebScraper`, `FormAnalyzer`, title,
  `wait_for_element`, stylesheet discovery. `browser_get_page_content` and
  `get_accessibility_tree` compute on the browser thread (`with_page` in
  `protocols/browser_tools/core.rs`); only results cross threads.
- Still on scraper (`TODO(a3)`): the layout engine (screenshot gets
  `to_html()`), readability, `browser_extract`/`snapshot_url` extraction,
  search-engine parsers, `web_search.rs`. They read serialized HTML.

## 9. The link: what landed

### Cargo

```toml
deno_web_page = { git = "https://github.com/Brainwires/deno", rev = "<scripts/deno-fork.rev>", optional = true }
deno_core     = { git = …, rev = …, default-features = false,
                  features = ["v8", "reactor-tokio", "v8_use_custom_libcxx"], optional = true }

[patch.crates-io]
v8         = { git = "https://github.com/Brainwires/rusty_v8", rev = "f72a5f53…" }
sys_traits = { git = "https://github.com/Brainwires/deno", rev = "<same rev>" }

[patch."https://github.com/Brainwires/thalora-web-browser"]
thalora-dom = { path = "crates/thalora-dom" }
```

- Both are optional and enabled by the `core` feature. The rev appears three
  times in `Cargo.toml` (deno_web_page, deno_core, sys_traits) and in
  `scripts/deno-fork.rev`; bump them together.
- **Features.** A git dependency's own `.workspace = true` dependencies are
  resolved in the fork's workspace, so `deno_core` already arrives with
  `reactor-tokio` + `v8_use_custom_libcxx` and `deno_v8` with `simdutf`
  (`cargo tree -e features -i v8`). Thalora still names `deno_core` itself,
  with those features spelled out: it calls `JsRuntime::init_platform` and
  `v8::IsolateHandle`, and the custom-libc++ requirement of the prebuilt
  archive should not depend on how the fork happens to declare its workspace.
- **thalora-dom: the patch, one crate.** `deno_web_page` depends on
  thalora-dom by git rev of this repository; the `[patch]` resolves it to the
  workspace crate, so `Cargo.lock` has a single `thalora-dom` (no `source`)
  and `deno_web_page::thalora_dom::Document` *is* `thalora_dom::Document`.
  The workspace crate must stay source-compatible with what the pinned fork
  rev uses (additive changes only until the fork is bumped).
- **`sys_traits` is required**, not optional as section 5 guessed:
  `deno_fs` is in the graph (through `deno_web`/`deno_fetch`/`deno_io`) and
  calls `sys_traits::impls::dos_device_safe_path`, which only the fork's
  vendored 0.1.28 has. Cargo finds the package by name inside the fork repo.
- **`url`: crates.io 2.5.9** (`cargo update -p url`), the release with the
  RUSTSEC-2026-0592 fix. The fork stays on a patched 2.5.7 only because
  2.5.9's `^` percent-encoding breaks `deno_semver`/`deno_graph`; neither is
  in Thalora's graph (`cargo tree -i deno_semver`, `-i deno_graph`: no
  match), so the vendored copy is not needed. Page-visible effect: `new
  URL()` in pages encodes `^` as the current URL standard does.
- `denokv_sqlite`, `dprint-core`, `tokio-postgres`: not in the graph, not
  patched.
- **Cargo.lock.** The fork pins some crates exactly (`rand =0.8.6`, …), so
  several of Thalora's locked versions moved. Crates the fork introduced were
  then set to the versions in the fork's own `Cargo.lock` where the
  requirements allow (59 of 76), so the page runtime runs on the dependency
  versions the fork tests; among others this leaves one `aws-lc-sys` instead
  of two.
- **Toolchain.** The fork pins 1.99.1, which is the current stable; Thalora
  builds on stable with no `rust-toolchain.toml`. The fork's
  `--cfg tokio_unstable` and macOS linker flags are not needed. The
  Dockerfile's builder moved from 1.91.0 to 1.99.1.

### The V8 archive guard

`.cargo/config.toml` now forces `RUSTY_V8_MIRROR` to a location that does not
exist and `RUSTY_V8_MIRROR_FALLBACK=0`: a build that was not handed the
verified archive fails. Locally: `eval "$(python3 scripts/fetch_rusty_v8.py
--print-env)"`. CI's "Fetch prebuilt V8" step does the same with
`--github-env`; the Dockerfile runs the script before `cargo build`.

### Runtime shape

| Piece | Where | Notes |
|---|---|---|
| `JsSettings`, V8 root thread, watchdog, `JsPage` | `src/engine/browser/page_js.rs` | see below |
| Two-mode `PageDom` (JavaScript / static) | `page_dom.rs` | `with_doc` borrows the runtime's document; `static_doc_mut()` replaces `with_doc_mut` |
| Script loading, settle, script navigations, evaluate | `navigation/scripts.rs` | CSP (incl. nonces), SRI, mixed content, SSRF guard, `nosniff`, `ResourceCache` |
| Interaction through the runtime | `navigation/forms.rs` | click, set value, key/hover/scroll events |
| Idle pump | `session_thread.rs` | between jobs, cancel-safe |

- **V8 root thread.** V8 wants every thread that runs an isolate to descend
  from the thread that initialized the platform (the JIT's memory protection
  key is inherited at thread creation on x86-64 Linux). A library cannot
  count on `main` (test threads are siblings), so `page_js` initializes the
  platform on a dedicated thread and `BrowserThread`s are spawned *by* it. A
  page loaded on any other thread is static (`JsPage::new` refuses).
- **One `PageRuntime` per thread at a time**: V8 isolates must be dropped in
  reverse order of creation, so `PageDom` drops the old page's runtime before
  it creates the next.
- **Options.** `net_allow = ["*"]`; `deny_addresses` is the fork's default
  (loopback, private, link-local, metadata), minus `loopback` when the
  `test-hooks` / debug override `THALORA_ALLOW_LOOPBACK=1` is on (the
  override lifts loopback only, as for navigation). `storage_dir` is
  `<THALORA_PROFILE_DIR or ~/.thalora/profile>/storage`, created with
  `std::fs`. Heap limit 512 MiB.
- **Watchdog.** One process-wide thread; every call into JavaScript is armed
  with the script timeout (10 s), event-loop pumping with budget + timeout.
  It calls `terminate_execution()`; the call fails with a timeout error and
  the page stays usable.
- **Loading.** Classic scripts only: inline and blocking external scripts in
  document order, then `defer`, `DOMContentLoaded`, `async`, `load`, then the
  event loop for up to 1 s (5 s with `wait_for_js`). External scripts get
  `load`/`error` events. Scripts inserted by scripts are found by looking at
  the document after scripts and tasks ran. `nomodule` scripts run (modules
  do not, so Thalora is a non-module browser for now).
- **Navigation stays in Rust.** Link and form activations come back from
  `click()`; `location.*` and `form.submit()` requests from the queue. At most
  five script navigations in a row are followed; fragment-only changes load
  nothing.
- **Secrets** (`browser_fill_credential`) stay in the side table in both
  modes and are merged into a submission only for fields of the form whose
  action is being submitted to. Page JavaScript never sees them, so
  `browser_eval` cannot read them back; a login whose script reads the
  password field itself gets an empty value.

### What the fork should grow (requests, not done here)

1. `evaluate_json_async` (await a returned promise) — `browser_eval` is
   synchronous until the pinned rev has it.
2. A host hook for inserted `<script>` elements (and `document.currentScript`,
   `document.write`), so they run at insertion time and a script removed
   right after insertion (JSONP) is not missed.
3. Module scripts (a module loader fed by the host).
4. Idle detail: pending requests vs timers, so `NetworkIdle` can ignore a
   repeating timer.
5. A way to tell the page its URL changed (fragment navigations,
   `hashchange`), and to hand it cookies (`document.cookie`, `fetch`
   credentials) from Thalora's cookie store.
6. A startup snapshot: every navigation boots an isolate and evaluates the
   extension JavaScript from source.
7. `MouseEvent` coordinates / layout from the host, for hover and scroll.
