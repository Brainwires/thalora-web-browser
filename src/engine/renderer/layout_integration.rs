//! Layout Integration Module
//!
//! Connects the CSS processor, layout engine, and DOM to provide
//! real computed positions for elements via getBoundingClientRect().
//!
//! This module bridges:
//! - HTML parsing (scraper) -> DOM tree structure
//! - CSS parsing (CssProcessor) -> computed styles
//! - Layout computation (LayoutEngine/taffy) -> element positions
//! - DOM updates (ElementData) -> bounding_rect values

use anyhow::{Result, Context as AnyhowContext};
use scraper::{Html, Selector, ElementRef};
use std::collections::HashMap;

use super::css::{CssProcessor, ComputedStyles};
use super::layout::{LayoutEngine, LayoutElement, LayoutResult, ElementLayout};

/// Manages layout computation for HTML documents
pub struct LayoutIntegration {
    /// CSS processor for style computation
    css_processor: CssProcessor,
    /// Layout engine for position computation
    layout_engine: LayoutEngine,
    /// Cached layout results by element ID
    layout_cache: HashMap<String, ElementLayoutInfo>,
    /// Viewport dimensions
    viewport_width: f32,
    viewport_height: f32,
}

/// Cached layout information for an element
#[derive(Debug, Clone)]
pub struct ElementLayoutInfo {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

impl ElementLayoutInfo {
    /// Create from computed layout
    pub fn from_layout(x: f64, y: f64, width: f64, height: f64) -> Self {
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

    /// Create default layout for elements without explicit sizing
    /// Uses realistic defaults based on element type
    pub fn default_for_element(tag: &str, parent_width: f64, y_offset: f64) -> Self {
        let (width, height) = match tag.to_lowercase().as_str() {
            // Block elements - full width, auto height
            "div" | "section" | "article" | "main" | "aside" | "nav" | "header" | "footer" => {
                (parent_width, 20.0) // Default content height
            }
            // Paragraphs
            "p" => (parent_width, 24.0), // One line of text
            // Headings
            "h1" => (parent_width, 40.0),
            "h2" => (parent_width, 32.0),
            "h3" => (parent_width, 28.0),
            "h4" | "h5" | "h6" => (parent_width, 24.0),
            // List items
            "li" => (parent_width - 40.0, 24.0),
            "ul" | "ol" => (parent_width, 24.0),
            // Form elements
            "input" => (200.0, 32.0),
            "button" => (100.0, 36.0),
            "textarea" => (300.0, 100.0),
            "select" => (200.0, 32.0),
            // Table elements
            "table" => (parent_width, 100.0),
            "tr" => (parent_width, 40.0),
            "td" | "th" => (100.0, 40.0),
            // Images - placeholder size
            "img" => (300.0, 200.0),
            // Inline elements
            "span" | "a" | "strong" | "em" | "b" | "i" => (100.0, 20.0),
            // Canvas - common default
            "canvas" => (300.0, 150.0),
            // Video/Audio
            "video" => (640.0, 360.0),
            "audio" => (300.0, 54.0),
            // Default
            _ => (parent_width, 20.0),
        };

        Self::from_layout(0.0, y_offset, width, height)
    }
}

impl LayoutIntegration {
    /// Create a new layout integration with default viewport (1366x768)
    pub fn new() -> Self {
        Self::with_viewport(1366.0, 768.0)
    }

    /// Create with specific viewport dimensions
    pub fn with_viewport(width: f32, height: f32) -> Self {
        Self {
            css_processor: CssProcessor::new(),
            layout_engine: LayoutEngine::with_viewport(width, height),
            layout_cache: HashMap::new(),
            viewport_width: width,
            viewport_height: height,
        }
    }

    /// Set viewport dimensions
    pub fn set_viewport(&mut self, width: f32, height: f32) {
        self.viewport_width = width;
        self.viewport_height = height;
        self.layout_engine.set_viewport(width, height);
    }

    /// Process HTML and compute layout for all elements
    /// Returns a map of element selectors/ids to their computed layouts
    pub fn compute_layout_for_html(&mut self, html: &str) -> Result<HashMap<String, ElementLayoutInfo>> {
        // Clear previous state
        self.css_processor.clear();
        self.layout_cache.clear();

        // Parse HTML
        let document = Html::parse_document(html);

        // Extract and parse all <style> tags
        self.extract_and_parse_styles(&document)?;

        // Build layout element tree from DOM
        let root_element = self.build_layout_tree(&document)?;

        // Compute layout using taffy
        let layout_result = self.layout_engine.calculate_layout_from_elements(&root_element)?;

        // Flatten layout results into cache
        self.cache_layout_results(&layout_result.elements, 0.0, 0.0);

        Ok(self.layout_cache.clone())
    }

    /// Extract CSS from <style> tags and parse it
    fn extract_and_parse_styles(&mut self, document: &Html) -> Result<()> {
        // Select all <style> tags
        let style_selector = Selector::parse("style").unwrap();

        for style_element in document.select(&style_selector) {
            let css_text = style_element.text().collect::<String>();
            if !css_text.trim().is_empty() {
                if let Err(e) = self.css_processor.parse(&css_text) {
                    // Log but don't fail - CSS errors are common
                    eprintln!("CSS parse warning: {}", e);
                }
            }
        }

        // Also handle inline styles in style attributes
        let all_selector = Selector::parse("*").unwrap();
        for element in document.select(&all_selector) {
            if let Some(style_attr) = element.value().attr("style") {
                // Generate a unique ID for this element
                let element_id = self.generate_element_id(&element);
                if let Err(e) = self.css_processor.parse_inline_style(style_attr, &element_id) {
                    eprintln!("Inline style parse warning: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Build a LayoutElement tree from the HTML document
    fn build_layout_tree(&self, document: &Html) -> Result<LayoutElement> {
        // Find the root element (usually <html>)
        let html_selector = Selector::parse("html").unwrap();

        if let Some(html_element) = document.select(&html_selector).next() {
            self.element_to_layout(&html_element, 0)
        } else {
            // Fallback: create a default root
            Ok(LayoutElement {
                id: "root".to_string(),
                tag: "html".to_string(),
                styles: ComputedStyles {
                    display: Some("block".to_string()),
                    width: Some("100%".to_string()),
                    height: Some("100%".to_string()),
                    ..Default::default()
                },
                children: vec![],
            })
        }
    }

    /// Convert a scraper ElementRef to a LayoutElement
    fn element_to_layout(&self, element: &ElementRef, depth: usize) -> Result<LayoutElement> {
        let tag = element.value().name().to_string();
        let element_id = self.generate_element_id(element);

        // Get computed styles for this element
        let selector = self.element_to_selector(element);
        let mut styles = self.css_processor.compute_style(&selector);

        // Apply default styles for common elements if not specified
        self.apply_default_styles(&mut styles, &tag);

        // Apply inline style attribute
        if let Some(style_attr) = element.value().attr("style") {
            self.apply_inline_style(&mut styles, style_attr);
        }

        // Recursively process children (limit depth to avoid infinite recursion)
        let children = if depth < 50 {
            element
                .children()
                .filter_map(|child| {
                    if let Some(child_element) = ElementRef::wrap(child) {
                        self.element_to_layout(&child_element, depth + 1).ok()
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            vec![]
        };

        Ok(LayoutElement {
            id: element_id,
            tag,
            styles,
            children,
        })
    }

    /// Generate a unique identifier for an element
    fn generate_element_id(&self, element: &ElementRef) -> String {
        // Prefer id attribute
        if let Some(id) = element.value().attr("id") {
            return format!("#{}", id);
        }

        // Build a path-based ID
        let tag = element.value().name();
        let classes = element.value().attr("class").unwrap_or("");

        if !classes.is_empty() {
            let first_class = classes.split_whitespace().next().unwrap_or("");
            format!("{}.{}", tag, first_class)
        } else {
            tag.to_string()
        }
    }

    /// Convert element to a CSS selector string
    fn element_to_selector(&self, element: &ElementRef) -> String {
        let tag = element.value().name();
        let mut selector = tag.to_string();

        // Add ID if present
        if let Some(id) = element.value().attr("id") {
            selector.push_str(&format!("#{}", id));
        }

        // Add classes
        if let Some(classes) = element.value().attr("class") {
            for class in classes.split_whitespace() {
                selector.push_str(&format!(".{}", class));
            }
        }

        selector
    }

    /// Apply default CSS styles for common HTML elements
    fn apply_default_styles(&self, styles: &mut ComputedStyles, tag: &str) {
        // Apply user-agent defaults if not already set
        match tag.to_lowercase().as_str() {
            "html" => {
                if styles.display.is_none() {
                    styles.display = Some("block".to_string());
                }
                if styles.width.is_none() {
                    styles.width = Some("100%".to_string());
                }
                if styles.height.is_none() {
                    styles.height = Some("100%".to_string());
                }
            }
            "body" => {
                if styles.display.is_none() {
                    styles.display = Some("block".to_string());
                }
                if styles.margin.is_none() {
                    styles.margin = Some(super::css::BoxModel {
                        top: "8px".to_string(),
                        right: "8px".to_string(),
                        bottom: "8px".to_string(),
                        left: "8px".to_string(),
                    });
                }
            }
            "div" | "section" | "article" | "main" | "aside" | "nav" | "header" | "footer"
            | "p" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "ul" | "ol" | "li"
            | "form" | "table" | "tr" | "blockquote" | "pre" | "figure" | "figcaption" => {
                if styles.display.is_none() {
                    styles.display = Some("block".to_string());
                }
            }
            "span" | "a" | "strong" | "em" | "b" | "i" | "u" | "small" | "sub" | "sup"
            | "code" | "kbd" | "var" | "samp" | "abbr" | "cite" | "q" | "label" => {
                if styles.display.is_none() {
                    styles.display = Some("inline".to_string());
                }
            }
            "img" | "input" | "button" | "select" | "textarea" | "canvas" | "video" | "audio" => {
                if styles.display.is_none() {
                    styles.display = Some("inline-block".to_string());
                }
            }
            "script" | "style" | "link" | "meta" | "head" | "title" | "noscript" => {
                styles.display = Some("none".to_string());
            }
            _ => {
                if styles.display.is_none() {
                    styles.display = Some("block".to_string());
                }
            }
        }
    }

    /// Parse and apply inline style attribute
    fn apply_inline_style(&self, styles: &mut ComputedStyles, style_attr: &str) {
        for declaration in style_attr.split(';') {
            let declaration = declaration.trim();
            if declaration.is_empty() {
                continue;
            }

            if let Some((property, value)) = declaration.split_once(':') {
                let property = property.trim();
                let value = value.trim();

                match property {
                    "display" => styles.display = Some(value.to_string()),
                    "position" => styles.position = Some(value.to_string()),
                    "width" => styles.width = Some(value.to_string()),
                    "height" => styles.height = Some(value.to_string()),
                    "background-color" | "background" => styles.background_color = Some(value.to_string()),
                    "color" => styles.color = Some(value.to_string()),
                    "font-size" => styles.font_size = Some(value.to_string()),
                    "flex-direction" => styles.flex_direction = Some(value.to_string()),
                    "justify-content" => styles.justify_content = Some(value.to_string()),
                    "align-items" => styles.align_items = Some(value.to_string()),
                    "gap" => styles.gap = Some(value.to_string()),
                    _ => {
                        styles.other.insert(property.to_string(), value.to_string());
                    }
                }
            }
        }
    }

    /// Cache layout results recursively
    fn cache_layout_results(&mut self, elements: &[ElementLayout], parent_x: f64, parent_y: f64) {
        for element in elements {
            let info = ElementLayoutInfo::from_layout(
                element.x,
                element.y,
                element.width,
                element.height,
            );

            self.layout_cache.insert(element.id.clone(), info);

            // Recursively cache children
            self.cache_layout_results(&element.children, element.x, element.y);
        }
    }

    /// Get cached layout for an element by ID or selector
    pub fn get_element_layout(&self, id_or_selector: &str) -> Option<&ElementLayoutInfo> {
        self.layout_cache.get(id_or_selector)
    }

    /// Get all cached layouts
    pub fn get_all_layouts(&self) -> &HashMap<String, ElementLayoutInfo> {
        &self.layout_cache
    }

    /// Get layout info suitable for getBoundingClientRect()
    /// Returns (x, y, width, height, top, right, bottom, left)
    pub fn get_bounding_client_rect(&self, element_id: &str) -> Option<(f64, f64, f64, f64, f64, f64, f64, f64)> {
        self.layout_cache.get(element_id).map(|info| {
            (info.x, info.y, info.width, info.height, info.top, info.right, info.bottom, info.left)
        })
    }

    /// Create a DOMRect-like structure for JavaScript
    pub fn create_dom_rect(&self, element_id: &str) -> HashMap<String, f64> {
        let mut rect = HashMap::new();

        if let Some(info) = self.layout_cache.get(element_id) {
            rect.insert("x".to_string(), info.x);
            rect.insert("y".to_string(), info.y);
            rect.insert("width".to_string(), info.width);
            rect.insert("height".to_string(), info.height);
            rect.insert("top".to_string(), info.top);
            rect.insert("right".to_string(), info.right);
            rect.insert("bottom".to_string(), info.bottom);
            rect.insert("left".to_string(), info.left);
        } else {
            // Return reasonable defaults rather than zeros
            // This is more realistic for a real browser
            rect.insert("x".to_string(), 0.0);
            rect.insert("y".to_string(), 0.0);
            rect.insert("width".to_string(), self.viewport_width as f64);
            rect.insert("height".to_string(), 20.0); // Reasonable default line height
            rect.insert("top".to_string(), 0.0);
            rect.insert("right".to_string(), self.viewport_width as f64);
            rect.insert("bottom".to_string(), 20.0);
            rect.insert("left".to_string(), 0.0);
        }

        rect
    }
}

impl Default for LayoutIntegration {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_html_layout() {
        let mut integration = LayoutIntegration::new();

        let html = r#"
            <!DOCTYPE html>
            <html>
            <head>
                <style>
                    .container { display: flex; width: 100%; height: 200px; }
                    .item { width: 100px; height: 100px; }
                </style>
            </head>
            <body>
                <div class="container">
                    <div class="item" id="item1"></div>
                    <div class="item" id="item2"></div>
                </div>
            </body>
            </html>
        "#;

        let layouts = integration.compute_layout_for_html(html).unwrap();

        // Should have computed layouts for elements
        assert!(!layouts.is_empty());

        // Check that we have some layout data
        for (id, layout) in &layouts {
            println!("Element {}: {:?}", id, layout);
        }
    }

    #[test]
    fn test_inline_styles() {
        let mut integration = LayoutIntegration::new();

        let html = r#"
            <html>
            <body>
                <div id="styled" style="width: 300px; height: 150px;"></div>
            </body>
            </html>
        "#;

        let layouts = integration.compute_layout_for_html(html).unwrap();

        if let Some(layout) = layouts.get("#styled") {
            assert_eq!(layout.width, 300.0);
            assert_eq!(layout.height, 150.0);
        }
    }

    #[test]
    fn test_create_dom_rect() {
        let mut integration = LayoutIntegration::new();

        let html = r#"
            <html>
            <body>
                <div id="test" style="width: 200px; height: 100px;"></div>
            </body>
            </html>
        "#;

        integration.compute_layout_for_html(html).unwrap();

        let rect = integration.create_dom_rect("#test");

        // Should have all DOMRect properties
        assert!(rect.contains_key("x"));
        assert!(rect.contains_key("y"));
        assert!(rect.contains_key("width"));
        assert!(rect.contains_key("height"));
        assert!(rect.contains_key("top"));
        assert!(rect.contains_key("right"));
        assert!(rect.contains_key("bottom"));
        assert!(rect.contains_key("left"));
    }

    #[test]
    fn test_default_element_layout() {
        let layout = ElementLayoutInfo::default_for_element("div", 1000.0, 0.0);
        assert!(layout.width > 0.0);
        assert!(layout.height > 0.0);

        let input_layout = ElementLayoutInfo::default_for_element("input", 1000.0, 0.0);
        assert_eq!(input_layout.width, 200.0);
        assert_eq!(input_layout.height, 32.0);
    }
}
