# GPUI plan: native Denext desktop backend, then the Thalora engine

Status: draft, 2026-11-23. This replaces the October re-architecture's GUI decision (Avalonia shell + Vello). V8 and "no Chromium" still stand.

## Decisions

| | |
|---|---|
| Order | **B** first: a GPUI-native renderer for Denext's React core. Then **A**: Thalora as a full HTML/CSS engine on GPUI. |
| Runtime | Brainwires/deno fork: deno_core 0.413 (libs/core), deno_v8 0.5.0 over Brainwires/rusty_v8 `156.0.0+denext.6` (V8 15.6, custom libc++). Never crates.io `v8`; a V8 16.0 bump is planned upstream of us. |
| GPUI pin | Follow ParolNet (`parol-social/apps/desktop`): `gpui-pre =0.3.12`, `gpui-pre-platform =0.3.12`, `gpui-component =0.10.0`, exact pins, Cargo.lock committed, and an upgrade is its own train. |
| Integration point | A new `desktop.backend: "gpui"`, next to `webview` and `cef`. The host is built like laufey: a prebuilt capi, SHA-pinned and downloaded by the fork. Packaging, permission baking, capabilities and the 5.0 kernel sandbox keep working. |
| WebRTC | JS-facing: deno `ext/webrtc` (owned by the Denext session, now on brainwires.5). Native Rust in the host: the `rtc` crates plus `rtc-stream-driver`. Pin to the **`v0.22.0+brainwires.6`** tag once it is cut; never master or 0.21.1. The .5 → .6 API is heavily breaking, so plan against .6. |
| Thalora's old deps | Avalonia and wasm are gone (A0, phase 0). Boa, `Brainwires/webrtc-rs` (async) and wasmtime go at A4. |

## Where the code lives (proposed, not decided)

| Piece | Home | Why |
|---|---|---|
| B host (GPUI window, isolate embedding, mutation ops consumer) | **Brainwires/laufey, as its own workspace member** (`backends/gpui`) that builds its own `laufey-gpui-<triple>` archive. Not folded into the shared capi build. A separate repo only if GPUI's dependency tree makes laufey's lockfile or CI painful; decide after B0. | Agreed with the denext PM session. Backends are already separate prebuilt archives per backend per target (`cli/laufey_sums.lock` pins `laufey-webview-*` / `laufey-cef-*`), chosen by `--backend` / `desktop.backend`. A new archive needs a new backend value, pinned digests and a download URL. The URL currently points at littledivy/laufey releases, so publishing ours means pointing the fork at Brainwires/laufey releases (a fork change). To keep upstream merges clean: GPUI code goes in new files/dirs only, shared capi headers get additive changes only, and those are offered upstream. GPUI's toolchain (Rust 1.95+, Metal) stays isolated to that member's CI jobs. |
| `denext/gpui` renderer | denext repo | That's where the React core and the RN compat layer live. |
| Fork ops (mutation batch, events) | Brainwires/deno, as numbered DENEXT.md items | Runtime surface. |
| A: the HTML/CSS engine | **This repo, repurposed** as a library (DOM, Stylo, layout, GPUI paint element), plus a standalone browser shell and headless/automation mode | The WPT, corpus and fingerprint tooling live here, and so does the original purpose (control over fingerprint and automation signals). laufey's "thalora" backend consumes the library. Keeping it here also keeps the deno fork from carrying a browser engine on its release cadence. |
| Shared GPUI media (video sink, decoders) | Extract into a small shared crate **when B3 starts**, not before | ParolNet is still changing `parolnet-media`; extracting early adds churn for that team. |

Not recommended: a separate "extract the APIs" step. `thalora-browser-apis` is bound to Boa, and the deno fork already covers fetch, streams, crypto, WebSocket, storage and webrtc. The rest (DOM, events, CSSOM, canvas, observers, web components) has to be rebuilt on the live DOM anyway. Port that logic during A3.

## Precedent: ParolNet desktop (Brainwires/parol-social, `apps/desktop`)

ParolNet already ships GPUI together with webrtc-rs-rtc. Its findings are in `docs/m0-spike.md` and `docs/m1-desktop.md` there. Adopt them rather than rediscover them.

**GPUI build facts**
- `gpui-pre` is a weekly snapshot of zed@main, republished by the gpui-component maintainer.
  - Don't mix it with official `gpui` 0.2.2: that links two incompatible GPUIs.
  - It needs Rust 1.95+. ParolNet keeps the desktop app as its own workspace with `rust-toolchain.toml` 1.99.1.
- Platform requirements:
  - macOS needs Xcode's Metal toolchain, or the `runtime-shaders` feature.
  - macOS text needs `gpui-platform`'s `font-kit` feature.
  - Linux needs the `x11`/`wayland` features, or nothing draws.
  - Linux renders through wgpu (`gpui-pre-wgpu`).
- Licences: MPL-2.0 exceptions for `cbindgen`, `dwrote` and `option-ext` go in `deny.toml`.
- CI: the `desktop-spike.yml` pattern builds on 5 targets, opens the window under Xvfb or the runner desktop, and screenshots it.

**Video into GPUI is solved there**
- macOS: I420 → full-range NV12 (`420f`, not `420v`, which the Metal surface pass refuses) in an IOSurface `CVPixelBuffer` → `gpui::surface()`.
- Windows/Linux: I420 → BGRA with the `yuv` crate (SIMD) → `RenderImage` → `img()`.
- Decode: system libvpx through hand-written FFI.

**Media stack choices with measurements**
- Opus: `opus-rs`.
- AEC/NS/AGC: `sonora`, chosen over the C++ webrtc-audio-processing, which has no Windows build.
- Camera: `nokhwa` on Windows/Linux, but weak on macOS, so AVFoundation directly there.
- Screen capture: `scap`, plus ScreenCaptureKit via gpui-pre.

**Native driver**
- `parolnet-rtc` is a tokio task per connection with command/event channels, plus interface gathering, mDNS, ICE restart, RTX and congestion → target bitrate.
- Compare it with `rtc-stream-driver` on rtc master before choosing one for the B host.

## B: GPUI-native renderer for Denext

The pieces:

- **Renderer:** `denext/gpui`, a subpath export in the denext repo. It is a react-reconciler-style renderer loaded through `src/compat/react-internals.ts`, uses `h()` with no JSX, and pulls zero npm. It reuses the RN props and the StyleSheet subset from `src/react-native/` and `src/react-native-compat/`.
- **Host:** a new repo for the Rust GPUI host (name TBD). The host owns the main thread and the window and runs the isolate in-process, using laufey's threading model (see board #36738).
- **Transport:** new fork ops, each a numbered DENEXT.md item in Brainwires/deno. Each React commit writes one mutation batch to a shared ArrayBuffer (op2 fast path), and events come back through the op loop.

### Phases

**B0, spike (go/no-go)**
- A Rust binary that opens a GPUI window and runs a deno_core isolate from the fork in-process.
- JS calls one op that sends a hard-coded box tree; GPUI draws it, and a click comes back to JS.
- Risks to retire:
  - Does gpui-pre link next to V8's custom libc++ (`std::__Cr`) in one binary, on all three OSes?
  - The main-thread split: the GPUI loop on main, the isolate on its own thread, or on main via a pump.
  - Build time and binary size.
  - Packaging: the 5.0 kernel-sandbox desktop profile and the macOS signing/notarization path in denext's package scripts must allow what GPUI loads (the Metal shader cache, fonts).

**B0 result (2026-11-23): GO with conditions.** `spike/gpui-b0` in Brainwires/laufey (`196883f`, not pushed); details in `backends/gpui/SPIKE.md` there.

On macOS arm64:
- It links on the first try. V8's libc++ (`std::__Cr`) is the only C++ runtime in the process, with no duplicate symbols.
- `b0-result presses=20 counter=20`.
- Commit to view: 0.25 ms. Press to repainted frame: 33 ms.
- First frame at 1.8 s (not yet explained; likely runtime shader compile).
- Binary: 73 MB raw, 54 MB stripped. Clean build ~1m40s.

Conditions before B1 is called done:
- Windows (static CRT, 4 MiB js-thread stack) and Linux (libstdc++ next to V8's libc++?) must link in a laufey train.
- Replace the whole-fork git dependency on deno_core, which costs 3.5 min / 2.3 GB of submodules. Proposed DENEXT.md item.
- Check the sandbox profile and notarization.
- Explain the 1.8 s first frame.

**B1, mutation protocol + core primitives**
- Batch format: create, insert-before, remove, set-props, set-style, set-text and commit. Node ids are u32 and the layout is versioned.
- Host side: a retained node tree mapped onto GPUI elements. RN flexbox maps onto GPUI's Taffy styles.
- Primitives: View, Text (nested spans), Image (async decode, cache), ScrollView, TextInput (IME, selection, clipboard) and Pressable (pressability states).
- Events: press, layout (onLayout), scroll, focus, key and text input.
- Tests: a golden screenshot per primitive and a mutation-fuzz test against a reference tree.

**B2, Denext integration**
- `desktop.backend: "gpui"` in the fork: download, SHA check, launch and the capabilities probe.
- `Platform.OS` / `Platform.constants.denextShell` gets a GPUI value.
- Dev reload and packaging through `denext desktop run` / `denext desktop package`.
- Works under the kernel-sandbox profile.
- Port VirtualList (FlatList, FlashList, LegendList) and `denext/navigation`. Both are DOM-based today, and they are the biggest items.
- Animation (`src/react-native/animation/`) driven from GPUI frames.
- Accessibility via GPUI's a11y, as far as it goes.

**B3, WebRTC in a GPUI app**
- `ext/webrtc` on brainwires.6, once the Denext session bumps it.
- Remote video: reuse ParolNet's path (libvpx → `420f` IOSurface → `gpui::surface()` on macOS, BGRA `RenderImage` elsewhere). The host sends PLI/FIR on decode errors.
  - Prefer extracting `parolnet-media`'s decode and video-sink pieces into a shared crate over copying them (an open decision).
  - Add hardware decoders (VideoToolbox, Media Foundation, VA-API) and H.264/AV1 later.
- Audio: adopt `rtc-audio`/`rtc-devices` (in brainwires.7) rather than building our own.
- Host duties a webview used to cover:
  - Camera, mic, screen and Local Network permissions:
    - macOS: TCC strings and entitlements, ScreenCaptureKit
    - Windows: privacy toggles
    - Linux: PipeWire and xdg-desktop-portal
  - `enumerateDevices`/`devicechange`.
  - Network-change → ICE restart.
  - Privacy defaults: mDNS host candidates, and STUN SOFTWARE switched off.
  - A keychain for DTLS certificates.

## A: Thalora as a GPUI web engine

Starts after B1, so the host, the runtime embedding and the GPUI pin are already proven. This work lives in this repo.

**A0, cleanup**
- Done on `a0/drop-avalonia`, based on `phase0-cleanup`: the Avalonia `gui/`, `src/ffi/` (C ABI), `cdylib`, the GUI xtask commands and `scripts/ralph` are removed.
- phase0-cleanup (#17) had already dropped wasm, reduced AI memory to the vault, and added the WPT, corpus and fingerprint tooling.
- Boa, `thalora-browser-apis`, wasmtime and the old `webrtc` dep stay until A4, so nothing that works today breaks before V8 replaces it.
- Leftovers for A1/A2:
  - C#/Avalonia comments, and ARGB colour order, in `styled_tree.rs` / `page_layout.rs`.
  - The `src/gui/fonts` path name.
  - The `desktop` feature name.

**A1, one live DOM and style**
- html5ever builds a single mutable tree (evaluate blitz-dom vs our own `thalora-dom`).
- Stylo for cascade and computed style, using `selectors` for matching.

**A2, layout + paint**
- Taffy (block, flex, grid) plus Parley (inline formatting, text) produce a display list.
- A custom GPUI `Element` paints it: quads, borders, text runs, images, clips and transforms.
- Scrolling, hit-testing and input are mapped back to DOM events.

**A3, JS**
- V8 via the fork, reusing Deno's web extensions: fetch, streams, URL, crypto, WebSocket and webrtc.
- DOM/CSSOM/HTML bindings are generated from WebIDL.
- Port the logic (not the bindings) from `engines/thalora-browser-apis`, which has about 775 `boa_engine` references.

**A4, conformance**
- A WPT runner, the interactive corpus, HAR replay and the fingerprint diff.
- Reach parity, then delete Boa and `thalora-browser-apis`.

**A5, laufey backend "thalora"**
- The existing bridge (`bridgeOrigins`, `Deno.desktop.*`, WS relay) runs on Thalora pages.
- A real Denext app (App Router SSR + SPA) runs unchanged.

## Open decisions (user)

- The name and owner of the B host repo, and whether B's host and A's engine share one repo later.
- Board item added 2026-11-23: "GPUI native desktop backend …(proposal first)" on Brainwires #2 (Todo, P2, later).
- B3 timing relative to brainwires.6 and the `ext/webrtc` bump.
- Whether to split ParolNet's GPUI media pieces (`parolnet-media` video sink, possibly `parolnet-rtc`) into shared Brainwires crates that ParolNet, the B host and Thalora all use. The parol-social session should agree first.

## Contacts

- Denext / fork / laufey / `ext/webrtc`: the session `deno-desktop-electron-parity`.
- webrtc-rs-rtc: the session `parol-social-59`. It also owns ParolNet, the GPUI precedent.
