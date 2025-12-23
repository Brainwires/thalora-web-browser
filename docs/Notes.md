Here are some of the canonical test‐suites, compatibility databases, and feature spec trackers that you can use or reference when building a headless browser, so you can define “compliance” in measurable terms. I also include some suggestions on how to extract the exact list of features Chrome supports.

---

Websocket test timing issue## Test Suites & Compatibility Databases

These are sources of tests or data you can run (or use as baseline) to verify feature support.

| Name                                        | What it is / What it covers                                                                                                                                    | Useful for integration                                                                                                                       |
| ------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------- |
| **Web-Platform-Tests (WPT)**                | A broad, cross-browser open test suite maintained by W3C et al. It includes tests for HTML, DOM, CSS, Media, many web standards. ([web-platform-tests.org][1]) | You can pull in WPT, run its tests in your headless browser; any failures tell you what you're missing. Also good for defining test harness. |
| **MDN browser-compat-data (BCD)**           | Machine-readable data for Web APIs, CSS features, HTML elements, etc.; shows which browser versions support which features. ([GitHub][2])                      | Useful for building a spec of “should support these APIs in this minimal version of Chrome”; also can auto-generate feature matrices.        |
| **“Can I use”**                             | Tabulated support data per feature / API across browsers. ([caniuse.com][3])                                                                                   | Good for UI-level overview; when you want to allow fallback or detect missing support.                                                       |
| **Chrome Platform Status (“Chromestatus”)** | Shows features that Chrome is working on, origin trials, new/experimental API features, etc. ([Chrome Status][4])                                              | Useful to see what features are shipped stable vs in trial, helps decide what you want to require.                                           |
| **RuntimeEnabledFeatures in Blink**         | List of Chrome/Blink features that are enabled/disabled (or behind flags) on different platforms. ([Chromium Git Repositories][5])                             | Useful for seeing what features might be platform or build specific; good for ensuring your headless browser matches the target platform.    |

---

## Chrome / Web Features you’ll want to include in your “full support” spec

Here are categories (with examples) of APIs / behaviors you should test/implement to match Chrome-ish “full support”. Each of these can be expanded into dozens of specific tests. Use WPT + BCD + Chromestatus to populate the finer grained list.

1. **HTML & DOM**

   * Document parsing (HTML5 parsing, tags, void elements, custom elements)
   * DOM mutation methods, shadow DOM
   * HTML forms, inputs, validation
   * HTML media elements (`<video>`, `<audio>`) and codec support
   * HTML5 Canvas (2D, WebGL, WebGL2)
   * Web Components (Templates, slots, custom elements)

2. **CSS / Styling**

   * CSS selectors, specificity, combinators, pseudo-classes/elements
   * CSS layout models: flexbox, grid, block, inline, tables
   * CSS animations, transitions
   * CSS media queries, feature queries
   * CSS variables (custom properties)
   * CSS geometric transforms

3. **JavaScript / ECMAScript**

   * Latest language features up to the version Chrome supports (async/await, modules, class properties, etc.)
   * ES modules, dynamic imports
   * Promises, async iteration, etc.

4. **Web APIs**

   * DOM API (events, attributes, nodes, etc.)
   * Fetch / XHR
   * WebSockets
   * Web Storage, IndexedDB
   * Service Workers / Cache API
   * Notifications, Push API
   * Geolocation, Permissions API
   * Web Workers
   * File / Blob / Streams / URL / URLSearchParams
   * WebRTC
   * WebXR
   * Web Audio API

5. **Security, CORS, Mixed Content, Origins**

   * Cross-origin resource sharing (CORS) behavior
   * Same-origin policy, CSP (Content Security Policy)
   * Redirects, referrer policy
   * TLS / certificate validation behavior

6. **Media, Graphics, Layout**

   * Image formats (JPEG, PNG, WebP, AVIF, etc.)
   * Font handling (webfonts, font features)
   * Video codecs (H.264, VP9, AV1 etc.), container formats
   * Audio codecs
   * OffscreenCanvas, GPU contexts

7. **Browser engine internals / Platform integration**

   * Flag / feature controlled APIs (behind flags)
   * Origin trials (for experimental APIs)
   * DevTools protocol support (if relevant)
   * Printing, PDF generation (if relevant)

8. **Accessibility & UI Behaviors (maybe)**

   * ARIA support
   * Layout reflow, painting, hit testing
   * Keyboard navigation, focus management
   * Zoom / DPIs / pixel density behavior

---

## How to Extract Chrome’s Current Supported Features

To build a document “Chrome supported features” (which can then be compared vs your implementation), you can:

* **Use Chrome Platform Status** to see what’s marked “stable / shipped” + what’s in origin trials. That gives you features you *may not* require for 100% but can mark “experimental.” ([Chrome Status][4])
* **Use MDN BCD data**, filtering for Chrome, and only those features with compat support = “yes” (or certain Chrome version). That allows you to generate a tree (feature → Chrome version) automatically.
* **Run WPT** test results on Chrome to see current failures/passes (wpt.fyi has Chrome vs feature results).
* **Inspect Blink’s `RuntimeEnabledFeatures`** JSON metadata to see what features are stable, experimental, or platform-flagged. ([Chromium Git Repositories][5])

---

If you want, I can generate a *starting list* (in Typescript / JSON) of “must-have web APIs & behaviors” modeled after Chrome’s latest stable version that you can use as your spec. Do you want me to build that for you?

[1]: https://web-platform-tests.org/?utm_source=chatgpt.com "Web Platform Tests"
[2]: https://github.com/mdn/browser-compat-data?utm_source=chatgpt.com "mdn/browser-compat-data"
[3]: https://caniuse.com/?utm_source=chatgpt.com "Can I use... Support tables for HTML5, CSS3, etc"
[4]: https://chromestatus.com/?utm_source=chatgpt.com "Chrome Platform Status"
[5]: https://chromium.googlesource.com/chromium/src/%2B/main/third_party/blink/renderer/platform/RuntimeEnabledFeatures.md?utm_source=chatgpt.com "Runtime Enabled Features"


Here’s a JSON/TypeScript-structured *starting spec* for features your headless browser should aim to support, modeled on Chrome Stable’s current behavior. You can use this as a checklist or test plan. If you like, I can also generate a more exhaustive version directly extracted from MDN BCD or WPT.

---

## Feature Spec (Modeled After Chrome Stable)

```ts
interface FeatureSupport {
  name: string;
  stable: boolean;        // Is this feature considered stable/shipped in Chrome
  experimental?: boolean;  // If still behind flag / origin trial
  deprecated?: boolean;    // If feature is deprecated or being removed
}

const ChromeFeatureSpec: FeatureSupport[] = [
  // Core Web / Platform APIs
  { name: "DOM (Document Object Model)", stable: true },
  { name: "HTML5 parsing (void elements, custom elements)", stable: true },
  { name: "Shadow DOM", stable: true },
  { name: "Web Components (Templates, slots, custom elements)", stable: true },
  { name: "Fetch API", stable: true },
  { name: "XMLHttpRequest", stable: true },
  { name: "WebSocket", stable: true },
  { name: "Streaming / Readable / Writable / Transform streams", stable: true },
  { name: "Service Workers & Cache API", stable: true },
  { name: "Web Storage (localStorage, sessionStorage)", stable: true },
  { name: "IndexedDB", stable: true },
  { name: "File API / Blob / FileReader", stable: true },
  { name: "URL / URLSearchParams", stable: true },
  { name: "Web Crypto API", stable: true },
  { name: "Permissions API", stable: true },
  { name: "Geolocation API", stable: true },

  // Media / Graphics / Rendering / Layout
  { name: "Canvas 2D", stable: true },
  { name: "WebGL / WebGL2", stable: true },
  { name: "Video & Audio elements", stable: true },
  { name: "CSS Flexbox", stable: true },
  { name: "CSS Grid", stable: true },
  { name: "CSS Variables (Custom Properties)", stable: true },
  { name: "CSS Animations & Transitions", stable: true },
  { name: "CSS Media Queries", stable: true },
  { name: "CSS Feature Queries (@supports)", stable: true },
  { name: "CSS Pseudo-classes/elements", stable: true },
  { name: "Fonts / Web Fonts / Font Face", stable: true },
  { name: "Image formats (JPEG / PNG / WebP / AVIF / GIF etc.)", stable: true },

  // JS / ECMAScript
  { name: "ECMAScript Modules (static import / export)", stable: true },
  { name: "Dynamic import()", stable: true },
  { name: "Async / await", stable: true },
  { name: "Promises", stable: true },
  { name: "Generator functions / iterators", stable: true },
  { name: "Optional chaining / nullish coalescing", stable: true },
  { name: "BigInt", stable: true },
  { name: "Proxy / Reflect APIs", stable: true },

  // Security / Networking / Protocols
  { name: "HTTPS / TLS (modern versions)", stable: true },
  { name: "CORS (Cross-Origin Resource Sharing)", stable: true },
  { name: "Same-Origin Policy", stable: true },
  { name: "Content Security Policy (CSP)", stable: true },
  { name: "Mixed Content blocking", stable: true },
  { name: "Referrer Policy", stable: true },
  { name: "Redirect handling", stable: true },
  { name: "Cookies", stable: true },

  // Workers / Multithreading / Off-main-thread APIs
  { name: "Web Workers", stable: true },
  { name: "Service Worker", stable: true },
  { name: "Shared Workers", stable: true },
  { name: "Worklets (e.g. CSS Paint / Audio Worklet)", stable: true },

  // UX / UI / Input
  { name: "Pointer / Touch Events", stable: true },
  { name: "Keyboard Events & focus management", stable: true },
  { name: "Drag & Drop API", stable: true },
  { name: "Dialog element / <dialog>", stable: true },
  { name: "Fullscreen API", stable: true },
  { name: "Screen Capture / getDisplayMedia", stable: true },

  // Accessibility
  { name: "ARIA attributes & roles", stable: true },
  { name: "Focus navigation, keyboard focus", stable: true },

  // Deprecated or Removed
  { name: "Web SQL Database", stable: false, deprecated: true },
];
```

---

## Notes and Data Sources

* The **MDN Browser Compatibility Data (BCD)** package can be used to find whether a given API or CSS property is supported in a particular Chrome version. ([npm][1])
* Deprecated / removed features like *Web SQL Database* have been phased out. For example, Chrome has removed WebSQL in many contexts and deprecated it almost completely. ([Wikipedia][2])
* To find if something is “experimental / behind flags”, you’ll often see them marked in BCD or on Chrome Platform Status (“origin trials”) or in WPT results.
* “What’s New in Chrome” docs provide an ongoing list of recent features added or experimental APIs. ([Chrome for Developers][3])

---

If you want, I can generate a *machine-readable* (JSON) full manifest of Chrome Stable’s supported features as of the latest version (e.g. Chrome 140), pulling directly from BCD / WPT. Do you want that?

[1]: https://www.npmjs.com/package/%40mdn/browser-compat-data?utm_source=chatgpt.com "mdn/browser-compat-data"
[2]: https://en.wikipedia.org/wiki/Web_SQL_Database?utm_source=chatgpt.com "Web SQL Database"
[3]: https://developer.chrome.com/new?utm_source=chatgpt.com "What's New in Chrome | Docs"
