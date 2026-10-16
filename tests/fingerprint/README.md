# Fingerprint surface dump

Measures how Thalora's JavaScript environment differs from a real browser's,
as seen by page scripts (bot detection reads exactly this surface).

| File | What it is |
| --- | --- |
| `dump.js` | Self-contained walker. Defines `__fingerprintDump()`, returning canonical JSON (sorted keys) with a SHA-256 `digest` of the rest |
| `dump.html` | Loads `dump.js` and prints the JSON into `<pre id="out">`, for capture from any browser |
| `chrome-<major>.json` | Reference dump from real headless Google Chrome (test reference only) |
| `thalora-boa.json` | Dump from the current Thalora engine (Boa) |

## What is captured

- `globals`: `Object.getOwnPropertyNames(globalThis)` count and enumeration
  `order`, plus one compact descriptor per global (see below) and global
  symbol keys.
- `window`: whether `window === globalThis` and
  `Object.getOwnPropertyNames(window).length` (names listed only when
  `window` is not the global object). The walk itself covers `globalThis`.
- `interfaces`: for each global constructor with a `prototype` object
  (`HTMLElement`, `Navigator`, …): descriptors of the prototype's and the
  constructor's own members, the prototype chain and the parent interface.
- `namespaces`: own members of object-valued globals (`Math`, `JSON`,
  `Intl`, `CSS`, `console`, `chrome`, …).
- `values`: scalar `navigator` and `screen` values (read through their
  getters), viewport sizes, time zone and locale.
- `chains`: prototype-chain constructor names of `window`, `document`,
  `navigator`, `location`, an `Error`, ….
- `errors`: message, own keys and (URL-normalized, innermost three frames)
  `stack` of a constructed `Error`, a `TypeError` and a `ReferenceError`;
  `Error.stackTraceLimit`, `Error.captureStackTrace`.
- `toStringSamples`: `Function.prototype.toString` output for built-ins,
  bound functions and a native getter.

Descriptor encoding (one string per property):

```text
v|wec|number                         data property, typeof, flags writable/enumerable/configurable
v|w-c|object|Navigator>Object        object value: prototype chain
v|w-c|function|fetch|1|N             function: name, length, N = toString is native, X = not
a|-ec|get:N|set:-                    accessor: getter/setter native (N), non-native (X) or absent (-)
```

Getters are never invoked during the walk except for the `navigator`,
`screen` and `window` scalars listed above.

## Commands

```sh
# Thalora: spawns the thalora MCP stdio server (newest target/{debug,release}/thalora,
# or --bin PATH), serves this directory on a loopback port, navigates to
# dump.html and evaluates __fingerprintDump() with browser_eval
# (THALORA_PRESET=brainclaw, THALORA_ALLOW_LOOPBACK=1).
cargo xtask fingerprint-dump --out tests/fingerprint/thalora-boa.json

# Reference: the same page in headless Google Chrome (--dump-dom).
# --chrome=/path/to/chrome for a non-default install.
cargo xtask fingerprint-dump --chrome --out tests/fingerprint/chrome-155.json

# Categorized diff, A = reference, B = subject. Exits non-zero only with --strict.
cargo xtask fingerprint-diff tests/fingerprint/chrome-155.json tests/fingerprint/thalora-boa.json
```

Without the xtask, open `dump.html` in any browser (served over HTTP or
`file://`) and copy the `<pre>` contents, or run
`chrome --headless=new --dump-dom http://127.0.0.1:PORT/dump.html` and
HTML-unescape the `<pre id="out">` text.

Diff categories: missing globals, extra globals, missing members, extra
members (interface prototype/static and namespace members of objects both
sides have), descriptor mismatches (data vs accessor, flags, typeof,
getter/setter presence), non-native toString (subject functions whose
`toString` does not end in `{ [native code] }` where the reference's does),
value mismatches (function name/length, prototype chains, navigator/screen
values, error formats, toString samples, global count).

The Chrome reference is headless, so its `userAgent` says `HeadlessChrome`
and some values (screen size, `outerWidth`) are headless defaults; treat
those value mismatches as informational.

## Current numbers

`chrome-155.json` (HeadlessChrome 155, macOS) vs `thalora-boa.json` (debug
build, 2026-10-15):

| Category | Count |
| --- | --- |
| Global properties | Chrome 1235, Thalora 226 |
| Missing globals | 1028 |
| Extra globals | 19 (e.g. `__globalEventTarget__`, `originalError`, `originalRegExp`, `ai`, `Record`, `Tuple`, `TypedArray`, `addEventListener`) |
| Missing members (interfaces both have) | 3062 |
| Extra members | 171 |
| Descriptor mismatches | 867 |
| Non-native toString | 85 (including `Error`, `RegExp`) |
| Value mismatches | 388 |

Notable single findings:

- `window !== globalThis`: `window` is a separate object with 5 own
  properties; its prototype chain is just `Object` (Chrome:
  `Window > WindowProperties > EventTarget > Object`), and
  `Object.prototype.toString.call(globalThis)` is `[object Object]`.
- `window` is a writable data property (Chrome: a non-configurable getter).
- `document.body`/`documentElement` chains are `Element > Object` (no
  `HTMLBodyElement`/`HTMLElement`/`Node`/`EventTarget`).
- `screen` is missing; `navigator.webdriver`, `deviceMemory`,
  `maxTouchPoints` are missing; `devicePixelRatio`/`innerWidth`/`outerWidth`
  are undefined; the time zone is `utc`.
- Errors have no `stack` for engine-thrown `TypeError`/`ReferenceError`,
  `Error.stackTraceLimit` is undefined, and messages differ (`cannot convert
  'null' or 'undefined' to object` vs `Cannot read properties of null …`).
- Interfaces with the wrong `name` (`Image` is named `HTMLImageElement`,
  `FileSystemHandle` has an empty name) or `length`.

Thalora caps each JavaScript evaluation at 5 s, which the whole walk exceeds
in a debug build (dump.html's own inline run times out and leaves
`pending`), so the xtask drives `__fingerprintDump.begin()/step()/finish()`
over several `browser_eval` calls and computes the digest in Rust over the
same canonical form dump.js uses (the Chrome digest matches dump.js's own).
