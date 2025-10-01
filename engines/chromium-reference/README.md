# Chromium Reference

This directory serves as a reference point for Chromium browser API implementations.

## Chromium Source Code

The full Chromium source code is too large to include as a submodule (~35GB).

### Online Reference

Browse Chromium source code online:
- **Official source**: https://source.chromium.org/chromium
- **GitHub mirror**: https://github.com/chromium/chromium

### Key Directories for Browser API Reference

When implementing browser APIs in Thalora, reference these Chromium directories:

- **Blink (Rendering Engine)**: `third_party/blink/renderer/`
  - DOM APIs: `third_party/blink/renderer/core/dom/`
  - Web APIs: `third_party/blink/renderer/modules/`
  - Bindings: `third_party/blink/renderer/bindings/`

- **Content API**: `content/`
  - Browser process: `content/browser/`
  - Renderer process: `content/renderer/`

- **V8 Bindings**: `third_party/blink/renderer/bindings/core/v8/`

### Local Clone (Optional)

To clone locally for offline reference (WARNING: ~35GB):

```bash
# Shallow clone to save space
git clone --depth=1 https://github.com/chromium/chromium.git chromium-src
```

## API Implementation Strategy

1. Reference Chromium's API implementations
2. Extract core logic and patterns
3. Implement in pure Rust in `thalora-browser-apis` crate
4. Bind to Boa JavaScript engine via engine trait
