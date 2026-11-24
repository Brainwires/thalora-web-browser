// Core browser engine components
//
// Page JavaScript runs on V8, through the page runtime of the Brainwires deno
// fork (`deno_web_page`, see `browser::page_js` and docs/A3_V8_DESIGN.md):
// one isolate per page, on the browser's thread, with web APIs only. A page
// is static (fetched, parsed and laid out from its HTML/CSS, no script run)
// when JavaScript is disabled or the page runtime could not start.

// Browser module - requires core networking (uses reqwest HTTP client)
#[cfg(feature = "core")]
pub mod browser;

pub mod renderer;
pub mod security;

// Re-exports for clean API
#[cfg(feature = "core")]
pub use browser::{
    AuthContext, BrowserStorage, Form, FormField, HeadlessWebBrowser, Image, InteractionResponse,
    Link, ScrapedData,
};
pub use renderer::{CssProcessor, LayoutEngine, LayoutResult};

/// Message of the error returned where an operation needs page JavaScript
/// and the page is static: JavaScript is disabled, the page runtime failed
/// to start, or the page was loaded outside a browser thread. The reason,
/// when known, follows in parentheses.
pub const JS_UNAVAILABLE: &str =
    "JavaScript execution is unavailable: this page was loaded as static HTML";

/// The error returned by operations that need page JavaScript on a static
/// page.
pub fn js_unavailable() -> anyhow::Error {
    anyhow::anyhow!(JS_UNAVAILABLE)
}
