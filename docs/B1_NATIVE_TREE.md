# B1 spec: the laufey native tree API and the `gpui` host

Status: draft 1, 2026-11-23. Owner: thalora-web-browser session. Reviewers: denext PM session (fork and denext parts).

## How a laufey backend actually runs

Verified against fork `train/denext30` 65ff17c8.

- A backend archive holds a **host executable** (`laufey_winit`, `laufey_webview.app/.exe`, the cef tree). It is not a library.
- The host's main thread runs the UI loop. The host `dlopen`s the app runtime `libdenort` (crate `denort_desktop`, cdylib), then calls `laufey_runtime_prestart(engine)`, `laufey_runtime_init(&backend_api)` and `laufey_runtime_start()` on a new thread.
  - See `webview/src/runtime_loader.cc:2352`, `backend-winit-common/src/lib.rs:6024`.
- **V8 lives in `libdenort`, not in the host.** The B0 spike embedded its own JsRuntime; production doesn't. The gpui host never links V8.
- JS → native: the `deno_desktop` ops (`runtime/ops/desktop.rs`) go to the `DesktopApi` impl `WefDesktopApi` (`cli/rt_desktop/lib.rs:872`), then into the `laufey` rlib, then through the C-ABI vtable the host supplied.
- Native → JS: host callback → `event_tx.try_send(DesktopEvent::…)` → `op_desktop_recv_event`.
- UI-thread work goes through the vtable's `dispatch_ui_task`.
- The template for a Rust host is the **winit ("raw") host**, `backend-winit-common`. It has no web engine and no JS bridge, which makes it the closest existing shape to gpui.

## Design

A new, **backend-generic, additive** laufey capability: **native tree**. A window can host a retained tree of native primitives that the runtime mutates in batches.
- The `gpui` host implements it. Other hosts report it as unsupported, though a winit + custom-renderer host could implement it later.
- `windowCapabilities()` reports `nativeTree: { version: 1 }`.

### Layers and owners

| Layer | Repo | Owner |
|---|---|---|
| `laufey` capi: `NativeTree` API, the vtable entries, event callback, `windowCapabilities` flag. Additive only, offered upstream. | Brainwires/laufey | thalora session |
| `backends/gpui`: host executable (vtable impl, runtime loading, GPUI tree renderer) | Brainwires/laufey | thalora session |
| Fork: `op_desktop_native_tree_commit`, a `DesktopEvent::NativeTree` variant, the `--backend gpui` value across the `desktop.rs` match sites, the kernel-sandbox engine id 3, the schema enum | Brainwires/deno | denext PM session, or drafted by the thalora session for its review |
| `denext/gpui`: react-reconciler-style renderer, batch encoder, event dispatch, RN primitives | denext | denext PM session, or drafted for its review |

### Transport

- JS: `Deno.desktop` window method `nativeTree.commit(buffer: Uint8Array)`.
  - Implemented by the sync op `op_desktop_native_tree_commit(rid, #[buffer] bytes)`.
  - The op calls `laufey::NativeTree::commit(window, &[u8])`, which goes to the vtable entry `native_tree_commit(window_id: u32, data: *const u8, len: usize) -> i32`.
- The host copies the bytes, then applies the whole batch **atomically on the UI thread** before the next frame. Ordering per window is FIFO.
- Return codes:
  - 0 = ok;
  - -1 = unsupported;
  - -2 = malformed batch. The host rejects the whole batch and the tree stays at the last good state.
- Events: the host calls the runtime's event callback with `(window_id, json_utf8_ptr, len)`. The fork wraps the payload as `DesktopEvent::NativeTree { window, json }`, and the renderer parses it. Events are low-rate apart from scroll and pointer move, which the host coalesces to one per frame per node. JSON is acceptable here; batches are where the volume is.

### Batch format, version 1

All integers little-endian. Strings are `u32 byte_len` followed by UTF-8. Node id 0 means "none".

```
header:  magic "NTB1" (4 bytes) | version u16 = 1 | flags u16 = 0 | seq u32 | op_count u32
op:      opcode u8 | payload
```

| Opcode | Name | Payload | Notes |
|---|---|---|---|
| 1 | CREATE | id u32, kind u16 | Kinds: 1 View, 2 Text, 3 RawText, 4 Image, 5 ScrollView, 6 TextInput, 7 Pressable. Ids are allocated by JS and never reused within a window's lifetime. |
| 2 | INSERT | parent u32, child u32, before u32 | `before` 0 = append. Moving an attached child = INSERT again (an implicit remove). |
| 3 | REMOVE | parent u32, child u32 | Detaches; the node stays alive. |
| 4 | DESTROY | id u32 | Frees the node and its detached subtree. |
| 5 | SET_ROOT | id u32 | The window's root view. |
| 6 | SET_STYLE | id u32, n u16, n × (prop u16, value) | Sets only the listed props. A prop given the value `unset` resets to its default. |
| 7 | SET_PROP | id u32, prop u16, value | Non-style props (below). |
| 8 | SET_TEXT | id u32, string | RawText content. |
| 9 | LISTEN | id u32, mask u32 | Event subscription bitmask (below); this replaces the old mask. |
| 10 | SET_IMAGE | id u32, image_id u32 | Points an Image node at a decoded image resource. |
| 11 | IMAGE_DATA | image_id u32, mime string, bytes (u32 len + data) | JS fetches the bytes (same-origin rules stay in JS); the host decodes and caches. |
| 12 | IMAGE_FREE | image_id u32 | |

**Value encoding:** `tag u8` followed by a payload.

| Tag | Payload |
|---|---|
| 0 unset | none |
| 1 f32 | 4 bytes |
| 2 percent | f32 |
| 3 auto | none |
| 4 enum | u8 |
| 5 color | u32 RGBA, straight alpha |
| 6 string | string |
| 7 bool | u8 |
| 8 f64 | 8 bytes |

**Style props (v1).** This is RN's StyleSheet subset, which maps onto Taffy / GPUI styles. Ids are fixed in a shared table: `backends/gpui/src/protocol/props.rs`, mirrored in `denext/gpui/props.ts`, and generated from one JSON source.
- **Layout:**
  - display, flexDirection, flexWrap, flex, flexGrow, flexShrink, flexBasis;
  - justifyContent, alignItems, alignSelf, alignContent, gap, rowGap, columnGap;
  - position, top/right/bottom/left;
  - width, height, min/max width and height, aspectRatio;
  - margin*, padding*, border*Width;
  - overflow.
- **Paint:** backgroundColor, border*Color, borderRadius (per corner), opacity.
- **Text:** color, fontFamily, fontSize, fontWeight, fontStyle, lineHeight, letterSpacing, textAlign, textDecorationLine.
- **Later:** transform and shadows (v2).

**Non-style props:**
- TextInput: value, placeholder, editable, secureTextEntry, multiline, `eventCount` (RN-style, for controlled-input races).
- ScrollView: horizontal, scrollEnabled.
- Image: resizeMode.
- Pressable: disabled.
- All nodes: accessibilityLabel, accessibilityRole, testID.

### Events (host → JS, JSON)

`{"node":id,"type":…}`, where `type` is one of:

| Type | Extra fields |
|---|---|
| `press`, `pressIn`, `pressOut`, `longPress` | `x`, `y` |
| `layout` | `x`, `y`, `width`, `height`, after layout, only when the value changed |
| `scroll` | `x`, `y`, `contentWidth`, `contentHeight`, coalesced per frame |
| `focus`, `blur` | |
| `changeText` | `text`, `eventCount` |
| `submitEditing` | |
| `selectionChange` | `start`, `end` |
| `key` | `key`, `modifiers` |
| `imageLoad`, `imageError` | `image_id` |

Subscribe via LISTEN using these mask bits: 1 press family, 2 layout, 4 scroll, 8 focus, 16 text, 32 key, 64 image.

### Host requirements (`backends/gpui`)

1. **A host executable, `laufey_gpui`, modelled on the winit host.**
   - Parse `--runtime` / `LAUFEY_RUNTIME_PATH`, find the co-located runtime, load it, and call `prestart(3)`, `init(&vtable)` and `start`.
   - GPUI owns the main thread, and `dispatch_ui_task` runs closures on it.
2. **The vtable subset needed for a Denext app with no web view:**
   - window create, close, title, size and position, show/hide, focus;
   - quit and quit-request;
   - the window event callbacks;
   - capabilities;
   - native tree.
   - Every other entry (webview, menus, tray, dialogs, clipboard and so on) returns "unsupported" cleanly. It must never crash.
   - The full vtable surface is listed in `docs/c-abi.md`; record per-entry support in a table there.
3. **The tree renderer:** a retained `HashMap<u32, Node>`, with each node mapped to a GPUI element on every frame.
   - The B0 spike's `ui.rs` is the starting point.
   - Text uses GPUI's text system. Image uses `RenderImage`. ScrollView uses GPUI scroll handles. TextInput uses GPUI input handling, including IME.
4. **The self-test:** `--self-test` with a tiny bundled test runtime, i.e. a stub `libdenort` written in Rust that implements `laufey::main!`. It drives batches through the real vtable path and prints `b1-result …`. CI runs it in the laufey train.

### Fork requirements (Brainwires/deno)

- **Ops and API:** `op_desktop_native_tree_commit`, `DesktopEvent::NativeTree`, `nativeTree.commit` plus the event in `lib.deno.desktop.d.ts`, and the `windowCapabilities().nativeTree` flag.
- **The `gpui` backend value.** It needs adding in these places:
  - `cli_parser` choices and the schema enum;
  - `laufey_archive_name` and the binary/bundle locate functions (`desktop.rs` 2538, 2715, 2749, 2845, 2881);
  - `macos_runtime_dylib_name`;
  - `linux_runtime_deps`;
  - `kernel_sandbox::Engine::from_backend`;
  - `host_engine` (3 = gpui).
- **The laufey pin:** bump it to a Brainwires/laufey rev that has the NativeTree API. A Brainwires/laufey release source is needed for the `laufey-gpui-*` archive. Until then, dev runs use `LAUFEY_DEV_DIR`.

### denext requirements

- **The renderer:** `denext/gpui`, a react-reconciler host config on top of `src/compat/react-internals.ts`.
  - It accumulates operations during a commit and calls `nativeTree.commit` once in `resetAfterCommit`.
- **Encoding:** the batch encoder uses a growable ArrayBuffer and the shared props table.
- **Components:** View, Text, Image, ScrollView, TextInput and Pressable, re-using the prop types from `src/react-native/`. Event dispatch runs through RN's pressability.
- **Platform:** `Platform.constants.denextShell = "gpui"`.

## Out of scope for B1

- VirtualList and navigation (B2), animation (B2), accessibility beyond labels and roles (B2).
- Transforms and shadows (v2), WebRTC (B3).

## Parallel work plan

1. laufey capi NativeTree API, then the gpui host (one Fable agent; this is large and subtle work across the vtable and threading).
2. Fork changes: the denext session, or a draft for it.
3. denext renderer: the denext session, or a draft for it.

Steps 2 and 3 depend only on this spec and the props table, so all three run at once. Integration happens in one train across the three repos: a laufey train build plus a fork dev build with `LAUFEY_DEV_DIR`, running a sample denext app.
