pub mod core;
pub mod css;
pub mod execution;
pub mod js_security;
pub mod layout;
pub mod layout_bridge;
pub mod page_layout;
pub mod polyfills;
pub mod security;
pub mod styled_tree;
pub mod text_measure;

// Re-export the main types
pub use core::RustRenderer;
pub use css::CssProcessor;
pub use layout::{
    BoxModelSides, ElementLayout, LayoutElement, LayoutEngine, LayoutResult, parse_px_value,
};
pub use layout_bridge::{css_path_for_element, flatten_layout_to_rects};
pub use page_layout::{
    compute_page_layout, compute_page_layout_with_css, compute_styled_tree,
    compute_styled_tree_with_css,
};
pub use styled_tree::{ResolvedStyles, StyledElement, StyledTreeResult};
