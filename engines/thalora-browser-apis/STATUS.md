# Browser APIs Extraction Status

## Current State: ⚠️ PARTIAL EXTRACTION

API files have been **copied** from Boa but are **NOT yet functional** as a standalone crate.

### What Was Done

1. ✅ Created `thalora-browser-apis` crate structure
2. ✅ Copied all API files from `engines/boa/core/engine/src/builtins/`
3. ✅ Created module structure (dom, fetch, storage, worker, file, events, browser, crypto, console, timers)
4. ✅ Added to main Cargo.toml as dependency

### Current Issues

The extracted APIs have **deep integration with Boa internals** and cannot compile standalone:

1. **Missing module declarations**: Subdirectories need their own mod.rs files
2. **Import path errors**: Files import from `crate::` expecting to be inside Boa
3. **Internal dependencies**: APIs depend on Boa's internal Context, intrinsics, etc.
4. **Tight coupling**: APIs are tightly coupled to Boa's builtin registration system

### Build Errors

```
error[E0583]: file not found for module `css`
error[E0583]: file not found for module `document_fragment`
error[E0583]: file not found for module `message_event`
error[E0583]: file not found for module `timers`
error[E0428]: the name `PERFORMANCE_STATE` is defined multiple times
```

## Why This Is Hard

The Boa engine APIs were **never designed to be extracted**. They are deeply integrated:

- APIs use `crate::builtins::` imports extensively
- APIs depend on Boa's `Context` and `Realm` internals
- APIs use Boa's intrinsic object registration system
- No clear abstraction layer between engine and APIs

## Recommended Approach

### Option 1: Keep APIs in Boa (RECOMMENDED)

**Leave the APIs where they are** in the Boa fork. This is what we've been doing successfully:

- ✅ Boa fork already has all browser APIs
- ✅ APIs work perfectly integrated with engine
- ✅ No refactoring needed
- ✅ Can reference Chromium for new APIs and add them to Boa

**Pros:**
- Zero work, everything already works
- APIs are battle-tested in Boa
- Can continue adding new APIs to Boa fork

**Cons:**
- APIs tied to Boa engine only
- Can't easily swap engines

### Option 2: Gradual Extraction (LONG TERM)

Extract APIs gradually over many months:

1. **Phase 1**: Create abstraction layer in Boa for one API (e.g., Console)
2. **Phase 2**: Move that one API to thalora-browser-apis
3. **Phase 3**: Make Boa import from thalora-browser-apis
4. **Phase 4**: Repeat for each API (50+ APIs = many months of work)

**Pros:**
- Eventually gets engine-independent APIs
- Could theoretically support multiple engines

**Cons:**
- Massive engineering effort (6-12 months)
- High risk of breaking existing functionality
- Unclear benefit since Boa works great

### Option 3: Hybrid Approach (PRAGMATIC)

Keep most APIs in Boa, extract only **truly engine-independent** ones:

- **Keep in Boa**: DOM, Events, Workers (tightly coupled to JS engine)
- **Extract**: HTTP client utilities, cookie management, etc. (pure Rust)

## Current Recommendation

**Use Option 1: Keep APIs in Boa**

The Boa fork is a "beast" (user's words) - a complete browser engine with APIs. This is actually a **strength**, not a weakness:

1. **Boa has the APIs**: Document, Element, Fetch, WebSocket, Workers, Storage, Crypto, etc.
2. **V8 has nothing**: Just JavaScript execution, no browser APIs
3. **APIs ARE the browser**: DOM, Fetch, Events, Storage are what make it a browser

The separation isn't between "engine" and "APIs" - it's:
- **Boa**: Complete browser JavaScript runtime (engine + all Web APIs)
- **Thalora**: HTTP client, navigation, session management, MCP server

## Next Steps

For now, **STOP the extraction**. The copied files remain as a reference but won't be used.

To add new browser APIs in the future:

1. Reference Chromium implementation
2. Implement in `engines/boa/core/engine/src/builtins/`
3. Register in Boa's intrinsics
4. Document in `engines/boa/ADDED-FEATURES.md`
5. Use in Thalora through Boa engine

## Files Status

### Extracted (but not usable)
- `engines/thalora-browser-apis/src/` - Contains copied files
- These files remain as reference but won't compile standalone

### Production (use these)
- `engines/boa/core/engine/src/builtins/` - **ACTUAL working browser APIs**
- This is where all browser APIs actually live and work

### Chromium Reference
- `engines/chromium-reference/` - Documentation for referencing Chromium source code
