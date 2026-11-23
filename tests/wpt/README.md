# Web Platform Tests

`cargo xtask wpt` runs a subset of [web-platform-tests](https://github.com/web-platform-tests/wpt)
against the `thalora` binary and reports how many pass. Numbers are **report-only** for now: nothing
fails CI on them. They exist to measure where we stand during the JS engine re-architecture.

## Running

```sh
cargo xtask wpt                          # default dirs, debug build, print summary
cargo xtask wpt --dirs url,encoding      # just some directories (or single files)
cargo xtask wpt --filter Node-cloneNode  # only tests whose path contains a substring
cargo xtask wpt --compare                # diff against results.json (regressions / improvements)
cargo xtask wpt --write                  # update results.json
```

Other options: `--timeout-ms N` (per test, default 10000; 6x for `timeout=long` tests), `--jobs N`,
`--wpt-dir PATH` (use an existing WPT checkout), `--release`, `--no-build`, `--verbose`,
`--log-dir PATH` (keep each thalora process's stderr), `--no-shims` (see below).
Setting `THALORA_ENGINE` passes it through to `thalora` and records it in `results.json`.

The first run fetches web-platform-tests at the commit in `WPT_COMMIT` into `target/wpt`
(shallow, blobless, sparse; never committed). Bump `WPT_COMMIT` to move to a newer WPT.

## How it works

- The runner is engine-neutral: it spawns `thalora` and talks to it only over MCP stdio
  (`browser_navigate_to` to load a test, `cdp_runtime_evaluate` to poll for results).
- The WPT tree is served from an in-process HTTP server on `127.0.0.1` (thalora runs with
  `THALORA_ALLOW_LOOPBACK=1`, which debug builds honour; `--release` builds with `test-hooks`).
- `/resources/testharnessreport.js` is replaced by one that stores
  `{status, message, tests: [{name, status, message}]}` on `window.__wpt_results`.
  `.any.js` (window scope only) and `.window.js` tests get generated wrapper pages, as wptserve does.
- A hung or crashed `thalora` process is killed and restarted; that test counts as TIMEOUT / CRASH.
  Thalora rate-limits tool calls (about 20 evaluations a minute per process), so results are
  polled sparsely and rate-limit replies are waited out.
- `thalora` gets `THALORA_PAGE_SCRIPT_TIMEOUT_MS=30000` (unless already set): evaluating
  testharness.js takes longer than the default 3 s per script in unoptimised builds.

## Harness shim

Thalora currently leaves `window.parent` undefined. testharness.js walks `self.parent` before the
first test starts and again on completion, so without a workaround *no* test can report a result.
Our testharnessreport.js therefore defines `parent` as the window itself when (and only when) it
is missing. This is recorded as `"harness_shims": ["window.parent"]` in `results.json`;
`--no-shims` turns it off (expect every test to TIMEOUT until the engine bug is fixed).

## What is measured

Only testharness.js tests. Skipped (and counted as skipped, with a reason): reftests, manual tests,
`.https.`, `.sub.` (need wptserve substitution), `.h2.`, `.www.`, worker-only tests, tests using
testdriver, and XHTML/SVG documents. There are no Python handlers, so tests that depend on
`.py` resources fail.

`results.json` has per-directory and per-file counts. File status is one of PASS (harness OK, every
subtest passed), FAIL, ERROR, TIMEOUT, CRASH or SKIP. Subtest counts are pass / fail / timeout /
notrun / skip (PRECONDITION_FAILED). The committed numbers come from a debug build.
