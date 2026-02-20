pub mod core;
pub mod security;
pub mod js_security;
pub mod execution;
pub mod css;
pub mod layout;
pub mod page_layout;
pub mod polyfills;

// Re-export the main types
pub use core::RustRenderer;
pub use css::CssProcessor;
pub use layout::{LayoutEngine, LayoutResult, ElementLayout, LayoutElement, BoxModelSides, parse_px_value};
pub use page_layout::compute_page_layout;