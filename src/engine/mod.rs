// Core browser engine components
//
// There is currently no JavaScript engine: Boa was removed and V8 (via the
// Brainwires deno fork) is not wired in yet. Pages are fetched, parsed and
// laid out from their static HTML/CSS; scripts are not executed.

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

/// Message of every error returned where JavaScript execution would be needed.
pub const JS_UNAVAILABLE: &str =
    "JavaScript execution is unavailable: the Boa engine was removed; V8 support is in progress";

/// The error returned by operations that need a JavaScript engine.
pub fn js_unavailable() -> anyhow::Error {
    anyhow::anyhow!(JS_UNAVAILABLE)
}
