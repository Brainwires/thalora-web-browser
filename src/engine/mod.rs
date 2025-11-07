// Core browser engine components
pub mod browser;
pub mod renderer;
pub mod engine;
pub mod engine_trait;
pub mod test_helpers;
pub mod security;
// DOM module removed - now natively implemented in Boa engine

// Re-exports for clean API
pub use browser::{HeadlessWebBrowser, ScrapedData, Link, Image, Form, FormField, InteractionResponse, BrowserStorage, AuthContext};
pub use renderer::{RustRenderer, CssProcessor, LayoutEngine, LayoutResult};
pub use engine::JavaScriptEngine;
pub use engine_trait::{ThaloraBrowserEngine, EngineType, EngineFactory, BoaEngineWrapper, EngineConfig};
// V8EngineWrapper removed - V8 engine was removed
pub use test_helpers::{create_test_engine, get_test_engine_type, is_using_v8, is_using_boa};
// DOM is now natively implemented in Boa engine
// EventListener is now natively implemented in Boa engine