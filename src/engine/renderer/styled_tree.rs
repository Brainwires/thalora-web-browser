//! Styled Element Tree
//!
//! Defines the output format for the new rendering pipeline where Rust computes
//! CSS styles (via lightningcss) and C# handles layout + rendering (via Avalonia).
//!
//! Unlike `ElementLayout` which contains pixel positions computed by taffy,
//! `StyledElement` contains only resolved CSS property values as strings.
//! The C# side converts these to Avalonia controls and lets Avalonia handle
//! measurement, layout, and painting natively.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Result of styled tree computation — the root element with all CSS resolved.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyledTreeResult {
    /// The root element (typically <html>)
    pub root: StyledElement,
    /// Viewport width used during CSS resolution
    pub viewport_width: f32,
    /// Viewport height used during CSS resolution
    pub viewport_height: f32,
    /// Mapping from element ID to a unique CSS selector string.
    /// Used by the GUI to dispatch DOM events back to the JS engine.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub element_selectors: Option<HashMap<String, String>>,
}

/// A styled DOM element with resolved CSS properties but no layout positions.
///
/// This is the bridge between Rust (CSS resolution) and C# (Avalonia rendering).
/// All style values are CSS strings that C# will parse into Avalonia types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyledElement {
    /// Unique element ID (e.g., "e1", "e2", ...)
    pub id: String,
    /// HTML tag name (e.g., "div", "p", "#text", "img")
    pub tag: String,

    // --- Content ---

    /// Text content for text nodes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_content: Option<String>,
    /// Image source URL for <img> elements
    #[serde(skip_serializing_if = "Option::is_none")]
    pub img_src: Option<String>,
    /// Image alt text for <img> elements
    #[serde(skip_serializing_if = "Option::is_none")]
    pub img_alt: Option<String>,
    /// Link href for <a> elements
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link_href: Option<String>,

    /// Resolved CSS styles
    pub styles: ResolvedStyles,

    /// Hover-specific CSS style overrides (from :hover rules).
    /// Only present when :hover rules match this element.
    /// The GUI applies these locally on PointerEntered/PointerExited — no Rust round-trip needed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hover_styles: Option<ResolvedStyles>,

    /// Child elements
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<StyledElement>,
}

/// Resolved CSS styles as strings.
///
/// All values are CSS strings (e.g., "16px", "#ff0000", "bold", "auto", "50%").
/// The C# side is responsible for parsing these into Avalonia-specific types.
/// This keeps the Rust side simple — it only resolves specificity and inheritance,
/// not the actual numeric/color values.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResolvedStyles {
    // --- Display & Layout ---

    /// CSS display value: block, flex, inline, inline-block, none, grid, table, list-item, etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<String>,
    /// CSS position: static, relative, absolute, fixed, sticky
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<String>,
    /// Flex direction: row, column, row-reverse, column-reverse
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flex_direction: Option<String>,
    /// Flex wrap: nowrap, wrap, wrap-reverse
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flex_wrap: Option<String>,
    /// Justify content: flex-start, center, flex-end, space-between, space-around, space-evenly
    #[serde(skip_serializing_if = "Option::is_none")]
    pub justify_content: Option<String>,
    /// Align items: stretch, flex-start, center, flex-end, baseline
    #[serde(skip_serializing_if = "Option::is_none")]
    pub align_items: Option<String>,
    /// Align self: auto, stretch, flex-start, center, flex-end, baseline
    #[serde(skip_serializing_if = "Option::is_none")]
    pub align_self: Option<String>,
    /// Gap between flex/grid children (e.g., "8px", "1em")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gap: Option<String>,
    /// Flex grow factor
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flex_grow: Option<String>,
    /// Flex shrink factor
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flex_shrink: Option<String>,
    /// Flex basis (e.g., "auto", "0", "200px")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flex_basis: Option<String>,

    // --- Box Model (CSS strings) ---

    /// Width (e.g., "100px", "50%", "auto")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<String>,
    /// Height
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<String>,
    /// Min width
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_width: Option<String>,
    /// Min height
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_height: Option<String>,
    /// Max width
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_width: Option<String>,
    /// Max height
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_height: Option<String>,

    /// Margin per side (CSS strings, e.g., "8px", "auto", "1em")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin: Option<StyleBoxSides>,
    /// Padding per side
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding: Option<StyleBoxSides>,

    // --- Typography ---

    /// Font size (e.g., "16px", "1.2em", "larger")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_size: Option<String>,
    /// Font family (e.g., "Fira Mono, monospace", "system-ui, sans-serif")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_family: Option<String>,
    /// Font weight (e.g., "normal", "bold", "700")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_weight: Option<String>,
    /// Font style (e.g., "normal", "italic", "oblique")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_style: Option<String>,
    /// Line height (e.g., "1.5", "24px", "normal")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_height: Option<String>,
    /// Text alignment (e.g., "left", "center", "right", "justify")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_align: Option<String>,
    /// Text decoration (e.g., "none", "underline", "line-through")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_decoration: Option<String>,
    /// Text transform (e.g., "none", "uppercase", "lowercase", "capitalize")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_transform: Option<String>,
    /// White space handling (e.g., "normal", "nowrap", "pre", "pre-wrap", "pre-line")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub white_space: Option<String>,
    /// Letter spacing (e.g., "normal", "2px", "0.1em")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub letter_spacing: Option<String>,
    /// Word spacing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub word_spacing: Option<String>,

    // --- Colors & Visual ---

    /// Text color (CSS color string, e.g., "#333", "rgb(0,0,0)", "red")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// Background color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_color: Option<String>,

    // --- Borders ---

    /// Border width per side (in CSS strings)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_width: Option<StyleBoxSides>,
    /// Border color (CSS color string)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_color: Option<String>,
    /// Border style (e.g., "solid", "dashed", "none")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_style: Option<String>,
    /// Border radius (e.g., "4px", "50%")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_radius: Option<String>,

    // --- Miscellaneous ---

    /// Opacity (0.0 to 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opacity: Option<f32>,
    /// Overflow handling (e.g., "visible", "hidden", "scroll", "auto")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overflow: Option<String>,
    /// Visibility (e.g., "visible", "hidden", "collapse")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<String>,
    /// Z-index
    #[serde(skip_serializing_if = "Option::is_none")]
    pub z_index: Option<i32>,
    /// List style type (e.g., "disc", "decimal", "none")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_style_type: Option<String>,
    /// Cursor style
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

/// Box model sides with CSS string values (not yet parsed to pixels).
/// Supports "auto", percentages, em/rem, etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleBoxSides {
    pub top: String,
    pub right: String,
    pub bottom: String,
    pub left: String,
}
