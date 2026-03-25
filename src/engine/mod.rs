// Core browser engine components

// Browser module - requires core networking (uses reqwest HTTP client)
#[cfg(feature = "core")]
pub mod browser;

// WASM browser stub - placeholder types for API compatibility
#[cfg(feature = "wasm")]
pub mod browser_wasm;
#[cfg(feature = "wasm")]
pub use browser_wasm as browser;

pub mod renderer;

// Engine module - requires core (uses tokio::sync::Mutex)
#[cfg(feature = "core")]
pub mod engine;

// WASM engine stub
#[cfg(feature = "wasm")]
pub mod engine_wasm;
#[cfg(feature = "wasm")]
pub use engine_wasm as engine;

pub mod engine_trait;
pub mod security;
pub mod test_helpers;
// DOM module removed - now natively implemented in Boa engine

// Re-exports for clean API
#[cfg(feature = "core")]
pub use browser::{
    AuthContext, BrowserStorage, Form, FormField, HeadlessWebBrowser, Image, InteractionResponse,
    Link, ScrapedData,
};
#[cfg(any(feature = "core", feature = "wasm"))]
pub use engine::JavaScriptEngine;
pub use engine_trait::{
    BoaEngineWrapper, EngineConfig, EngineFactory, EngineType, ThaloraBrowserEngine,
};
pub use renderer::{CssProcessor, LayoutEngine, LayoutResult, RustRenderer};
// V8EngineWrapper removed - V8 engine was removed
pub use test_helpers::{create_test_engine, get_test_engine_type, is_using_boa, is_using_v8};
// DOM is now natively implemented in Boa engine
// EventListener is now natively implemented in Boa engine
