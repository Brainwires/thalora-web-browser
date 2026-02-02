//! Global Layout Registry
//!
//! Provides a thread-local cache of computed element layouts that can be
//! populated by the layout engine and queried by DOM elements for
//! getBoundingClientRect().
//!
//! This bridges the layout computation (in the main crate) with the DOM
//! implementation (in thalora-browser-apis).

use std::cell::RefCell;
use std::collections::HashMap;

/// Computed layout information for an element
#[derive(Debug, Clone, Default)]
pub struct ComputedLayout {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

impl ComputedLayout {
    /// Create a new computed layout
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
            top: y,
            right: x + width,
            bottom: y + height,
            left: x,
        }
    }

    /// Create a default layout based on element type
    /// Returns realistic dimensions for common HTML elements
    pub fn default_for_element(tag: &str, viewport_width: f64) -> Self {
        let (width, height) = match tag.to_lowercase().as_str() {
            // Document root
            "html" | "body" => (viewport_width, 768.0),
            // Block elements - full width
            "div" | "section" | "article" | "main" | "aside" | "nav" | "header" | "footer" => {
                (viewport_width, 24.0)
            }
            // Paragraphs
            "p" => (viewport_width, 20.0),
            // Headings (realistic rendered heights)
            "h1" => (viewport_width, 42.0),
            "h2" => (viewport_width, 36.0),
            "h3" => (viewport_width, 30.0),
            "h4" => (viewport_width, 26.0),
            "h5" | "h6" => (viewport_width, 22.0),
            // List items
            "li" => (viewport_width - 40.0, 24.0),
            "ul" | "ol" => (viewport_width, 24.0),
            // Form elements (common default sizes)
            "input" => (173.0, 21.0), // Chrome default input size
            "button" => (54.0, 22.0),
            "textarea" => (300.0, 66.0),
            "select" => (152.0, 21.0),
            // Table elements
            "table" => (viewport_width * 0.9, 100.0),
            "tr" => (viewport_width * 0.9, 36.0),
            "td" | "th" => (100.0, 36.0),
            // Media elements
            "img" => (300.0, 150.0),
            "video" => (300.0, 150.0), // HTML5 default
            "audio" => (300.0, 32.0),
            "canvas" => (300.0, 150.0), // Default canvas size
            // Inline elements
            "span" | "a" | "strong" | "em" | "b" | "i" | "code" => (50.0, 18.0),
            // SVG
            "svg" => (300.0, 150.0),
            // Default for unknown elements
            _ => (viewport_width, 20.0),
        };

        Self::new(0.0, 0.0, width, height)
    }
}

thread_local! {
    /// Thread-local layout cache mapping element identifiers to their computed layouts
    static LAYOUT_CACHE: RefCell<HashMap<String, ComputedLayout>> = RefCell::new(HashMap::new());

    /// Default viewport width for calculations
    static VIEWPORT_WIDTH: RefCell<f64> = const { RefCell::new(1366.0) };

    /// Default viewport height for calculations
    static VIEWPORT_HEIGHT: RefCell<f64> = const { RefCell::new(768.0) };
}

/// Set the viewport dimensions for default layout calculations
pub fn set_viewport(width: f64, height: f64) {
    VIEWPORT_WIDTH.with(|w| *w.borrow_mut() = width);
    VIEWPORT_HEIGHT.with(|h| *h.borrow_mut() = height);
}

/// Get the current viewport width
pub fn get_viewport_width() -> f64 {
    VIEWPORT_WIDTH.with(|w| *w.borrow())
}

/// Get the current viewport height
pub fn get_viewport_height() -> f64 {
    VIEWPORT_HEIGHT.with(|h| *h.borrow())
}

/// Clear all cached layouts
pub fn clear_layouts() {
    LAYOUT_CACHE.with(|cache| cache.borrow_mut().clear());
}

/// Set the layout for a specific element
pub fn set_element_layout(element_id: &str, layout: ComputedLayout) {
    LAYOUT_CACHE.with(|cache| {
        cache.borrow_mut().insert(element_id.to_string(), layout);
    });
}

/// Set multiple element layouts at once (more efficient for bulk updates)
pub fn set_layouts(layouts: HashMap<String, ComputedLayout>) {
    LAYOUT_CACHE.with(|cache| {
        let mut c = cache.borrow_mut();
        c.clear();
        c.extend(layouts);
    });
}

/// Get the layout for a specific element
/// Returns the cached layout if available, otherwise returns a default based on element tag
pub fn get_element_layout(element_id: &str, tag: &str) -> ComputedLayout {
    LAYOUT_CACHE.with(|cache| {
        cache.borrow().get(element_id).cloned().unwrap_or_else(|| {
            let viewport_width = get_viewport_width();
            ComputedLayout::default_for_element(tag, viewport_width)
        })
    })
}

/// Get the layout for a specific element, returning None if not cached
pub fn get_element_layout_opt(element_id: &str) -> Option<ComputedLayout> {
    LAYOUT_CACHE.with(|cache| cache.borrow().get(element_id).cloned())
}

/// Check if there's a cached layout for an element
pub fn has_element_layout(element_id: &str) -> bool {
    LAYOUT_CACHE.with(|cache| cache.borrow().contains_key(element_id))
}

/// Get the number of cached layouts
pub fn layout_cache_size() -> usize {
    LAYOUT_CACHE.with(|cache| cache.borrow().len())
}

/// Get a DOMRect-like structure for an element
/// This is what getBoundingClientRect() should return
pub fn get_bounding_client_rect(element_id: &str, tag: &str) -> ComputedLayout {
    get_element_layout(element_id, tag)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_and_get_layout() {
        clear_layouts();

        let layout = ComputedLayout::new(10.0, 20.0, 100.0, 50.0);
        set_element_layout("test-element", layout);

        let retrieved = get_element_layout("test-element", "div");
        assert_eq!(retrieved.x, 10.0);
        assert_eq!(retrieved.y, 20.0);
        assert_eq!(retrieved.width, 100.0);
        assert_eq!(retrieved.height, 50.0);
    }

    #[test]
    fn test_default_layout() {
        clear_layouts();

        // Should get default layout for unknown element
        let layout = get_element_layout("nonexistent", "div");
        assert!(layout.width > 0.0);
        assert!(layout.height > 0.0);
    }

    #[test]
    fn test_element_specific_defaults() {
        let input_layout = ComputedLayout::default_for_element("input", 1366.0);
        assert_eq!(input_layout.width, 173.0); // Chrome default

        let canvas_layout = ComputedLayout::default_for_element("canvas", 1366.0);
        assert_eq!(canvas_layout.width, 300.0);
        assert_eq!(canvas_layout.height, 150.0);
    }

    #[test]
    fn test_viewport() {
        set_viewport(1920.0, 1080.0);
        assert_eq!(get_viewport_width(), 1920.0);
        assert_eq!(get_viewport_height(), 1080.0);

        // Reset for other tests
        set_viewport(1366.0, 768.0);
    }
}
