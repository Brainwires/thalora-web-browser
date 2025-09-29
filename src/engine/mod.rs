// Core browser engine components
pub mod browser;
pub mod renderer;
pub mod engine;
pub mod engine_trait;
// DOM module removed - now natively implemented in Boa engine

// Re-exports for clean API
pub use browser::{HeadlessWebBrowser, ScrapedData, Link, Image, Form, FormField, InteractionResponse, BrowserStorage, AuthContext};
pub use renderer::{RustRenderer, CssProcessor, LayoutEngine, LayoutResult};
pub use engine::JavaScriptEngine;
pub use engine_trait::{ThaloraBrowserEngine, EngineType, EngineFactory, BoaEngineWrapper, EngineConfig};
#[cfg(feature = "v8-engine")]
pub use engine_trait::V8EngineWrapper;
// DOM is now natively implemented in Boa engine
// EventListener is now natively implemented in Boa engine