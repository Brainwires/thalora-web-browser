// Core browser engine components
pub mod browser;
pub mod renderer;
pub mod engine;
pub mod dom;

// Re-exports for clean API
pub use browser::{HeadlessWebBrowser, ScrapedData, Link, Image, Form, FormField, InteractionResponse, BrowserStorage, AuthContext};
pub use renderer::{RustRenderer, CssProcessor, LayoutEngine, LayoutResult};
pub use engine::JavaScriptEngine;
pub use dom::{DomElement, EnhancedDom, DomMutation};
pub use crate::apis::events::EventListener;