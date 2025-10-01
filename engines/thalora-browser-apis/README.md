# Thalora Browser APIs

Browser Web APIs extracted from the Boa JavaScript engine fork for standalone use.

## Purpose

This crate will contain all browser Web APIs (Fetch, WebSocket, DOM, Storage, etc.) extracted from the Boa engine into a separate, reusable module. This allows:

1. **Engine Independence**: APIs can potentially be bound to different JavaScript engines
2. **Modularity**: Browser APIs are separate from the core JavaScript engine
3. **Reusability**: APIs can be used in different contexts (testing, headless browsing, GUI)
4. **Reference**: Chromium source code can be referenced for API implementations

## Current Status

**🚧 PLACEHOLDER - APIs Still in Boa Engine**

The APIs are currently still integrated within the Boa engine fork at `engines/boa/core/engine/src/builtins/`.

## Extraction Plan

### Phase 1: Identify API Modules

Web API modules in Boa (located in `engines/boa/core/engine/src/builtins/`):

**DOM APIs:**
- `document/` - Document interface
- `element/` - Element interface
- `node/` - Node interface
- `nodelist/` - NodeList interface
- `domtokenlist/` - DOMTokenList interface
- `attr/` - Attr interface
- `document_fragment/` - DocumentFragment interface
- `css/` - CSS interfaces

**Fetch & Networking:**
- `fetch/` - Fetch API
- `websocket/` - WebSocket API
- `websocket_stream/` - WebSocketStream API
- `event_source.rs` - EventSource (Server-Sent Events)

**Storage:**
- `storage/` - localStorage/sessionStorage
- `storage_event/` - StorageEvent
- `storage_manager/` - StorageManager
- `cookie_store/` - CookieStore API

**Workers:**
- `worker/` - Web Workers
- `worker_global_scope.rs` - WorkerGlobalScope
- `worker_navigator.rs` - WorkerNavigator
- `worker_script_loader.rs` - Worker script loading
- `worker_events.rs` - Worker-specific events
- `worker_error/` - Worker error handling

**Files:**
- `file/` - File API
- `file_reader/` - FileReader API
- `file_system/` - FileSystem API
- `blob/` - Blob API

**Events:**
- `event/` - Event interface
- `event_target/` - EventTarget interface
- `message_event/` - MessageEvent interface

**Browser Objects:**
- `window.rs` - Window interface
- `navigator/` - Navigator interface
- `history.rs` - History API
- `performance/` - Performance API
- `performance_old.rs` - Legacy performance APIs

**Crypto:**
- `crypto.rs` - Web Crypto API

**Console:**
- `console/` - Console API

**Timers:**
- `timers/` - setTimeout, setInterval, etc.

### Phase 2: Create Module Structure

```
src/
├── lib.rs              # Main exports and initialization
├── dom/                # DOM APIs
│   ├── document.rs
│   ├── element.rs
│   ├── node.rs
│   └── ...
├── fetch/              # Fetch & Networking
│   ├── fetch.rs
│   ├── websocket.rs
│   └── ...
├── storage/            # Storage APIs
│   ├── local_storage.rs
│   ├── cookie_store.rs
│   └── ...
├── worker/             # Web Workers
│   ├── worker.rs
│   ├── worker_global_scope.rs
│   └── ...
├── file/               # File APIs
│   ├── file.rs
│   ├── blob.rs
│   └── ...
├── events/             # Event system
│   ├── event.rs
│   ├── event_target.rs
│   └── ...
├── browser/            # Browser objects
│   ├── window.rs
│   ├── navigator.rs
│   ├── history.rs
│   └── ...
├── crypto/             # Crypto APIs
│   └── crypto.rs
├── console/            # Console API
│   └── console.rs
└── timers/             # Timer APIs
    └── timers.rs
```

### Phase 3: Extract Individual APIs

For each API module:

1. **Copy** from `engines/boa/core/engine/src/builtins/` to `engines/thalora-browser-apis/src/`
2. **Update imports** to use `boa_engine::` prefix
3. **Remove engine-specific bindings** (make them generic)
4. **Add tests** for standalone usage
5. **Update Boa** to use the extracted module (import from `thalora-browser-apis`)

### Phase 4: Testing

After extraction, ensure:

1. All tests in Boa still pass (using the extracted APIs)
2. APIs can be initialized independently
3. Thalora browser uses the extracted APIs
4. No circular dependencies

## Dependencies

- **boa_engine**: Required for JavaScript integration and type definitions
- **reqwest**: For Fetch API
- **tokio-tungstenite**: For WebSocket
- **tokio**: Async runtime

## Future: Engine Abstraction

Eventually, APIs could be made engine-agnostic by:

1. Creating trait-based engine abstraction
2. Implementing bindings for multiple engines (Boa, potentially others)
3. Runtime engine selection based on use case

## Contributing

When extracting APIs:

1. Maintain exact behavior from Boa
2. Keep tests alongside extracted code
3. Reference Chromium implementation for correctness
4. Document any deviations from specs
