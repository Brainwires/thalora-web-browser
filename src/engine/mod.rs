// Core browser engine components

// Browser module - native only (uses reqwest HTTP client)
#[cfg(any(feature = "native", feature = "web-search", feature = "mcp-server"))]
pub mod browser;

// WASM browser stub - placeholder types for API compatibility
#[cfg(feature = "wasm")]
pub mod browser_wasm;
#[cfg(feature = "wasm")]
pub use browser_wasm as browser;

pub mod renderer;

// Engine module - native only (uses tokio::sync::Mutex)
#[cfg(any(feature = "native", feature = "web-search", feature = "mcp-server"))]
pub mod engine;

// WASM engine stub
#[cfg(feature = "wasm")]
pub mod engine_wasm;
#[cfg(feature = "wasm")]
pub use engine_wasm as engine;

pub mod engine_trait;
pub mod test_helpers;
pub mod security;
// DOM module removed - now natively implemented in Boa engine

// Re-exports for clean API
#[cfg(any(feature = "native", feature = "web-search", feature = "mcp-server"))]
pub use browser::{HeadlessWebBrowser, ScrapedData, Link, Image, Form, FormField, InteractionResponse, BrowserStorage, AuthContext};
pub use renderer::{RustRenderer, CssProcessor, LayoutEngine, LayoutResult};
#[cfg(any(feature = "native", feature = "web-search", feature = "mcp-server", feature = "wasm"))]
pub use engine::JavaScriptEngine;
pub use engine_trait::{ThaloraBrowserEngine, EngineType, EngineFactory, BoaEngineWrapper, EngineConfig};
pub use test_helpers::{create_test_engine, get_test_engine_type, is_using_boa};
// DOM is now natively implemented in Boa engine
// EventListener is now natively implemented in Boa engine