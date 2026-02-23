//! Layout Engine with taffy integration
//!
//! Provides CSS-compliant layout computation using the taffy layout library.
//! Supports flexbox, grid, and block layout modes.

use anyhow::{Result, Context as AnyhowContext};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use taffy::prelude::*;
use taffy::style::{
    AlignContent, AlignItems, AlignSelf, Display, FlexDirection, FlexWrap,
    JustifyContent, JustifyItems, JustifySelf, Position,
};

use super::css::{ComputedStyles, CssProcessor};

/// Layout computation engine for calculating element positions and sizes
pub struct LayoutEngine {
    /// Taffy tree for layout computation
    tree: TaffyTree<LayoutNodeData>,
    /// Map from element IDs to taffy node IDs
    node_map: HashMap<String, NodeId>,
    /// Viewport width
    viewport_width: f32,
    /// Viewport height
    viewport_height: f32,
}

/// Additional data stored with each layout node
#[derive(Debug, Clone, Default)]
pub struct LayoutNodeData {
    /// Element ID
    pub id: String,
    /// HTML tag name
    pub tag: String,
    /// Computed CSS styles
    pub styles: ComputedStyles,
}

/// Result of layout computation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutResult {
    /// Total width of the layout
    pub width: f64,
    /// Total height of the layout
    pub height: f64,
    /// Laid out elements
    pub elements: Vec<ElementLayout>,
}

/// Layout information for a single element, including visual properties for painting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementLayout {
    /// Element ID
    pub id: String,
    /// HTML tag name
    pub tag: String,
    /// X position relative to viewport
    pub x: f64,
    /// Y position relative to viewport
    pub y: f64,
    /// Computed width
    pub width: f64,
    /// Computed height
    pub height: f64,
    /// Content box (excluding padding/border)
    pub content_box: Option<ContentBox>,
    /// Children elements
    pub children: Vec<ElementLayout>,

    // --- Visual properties for painting ---

    /// Text content of this element (direct text nodes combined)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_content: Option<String>,
    /// Link href (for <a> elements)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link_href: Option<String>,
    /// Image source URL (for <img> elements)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub img_src: Option<String>,
    /// Background color (CSS color string)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_color: Option<String>,
    /// Text color (CSS color string)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// Font size in px
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_size: Option<f64>,
    /// Font family name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_family: Option<String>,
    /// Font weight (normal, bold, 100-900)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_weight: Option<String>,
    /// Font style (normal, italic, oblique)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_style: Option<String>,
    /// Text alignment (left, center, right, justify)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_align: Option<String>,
    /// Text decoration (none, underline, line-through)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_decoration: Option<String>,
    /// Line height multiplier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_height: Option<f64>,
    /// White-space mode (normal, nowrap, pre, pre-wrap, pre-line)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub white_space: Option<String>,
    /// Border radius in px
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_radius: Option<f64>,
    /// Border width in px (uniform)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_width: Option<f64>,
    /// Border color (CSS color string)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_color: Option<String>,
    /// Opacity (0.0 - 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opacity: Option<f32>,
    /// Overflow mode (visible, hidden, scroll, auto)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overflow: Option<String>,
    /// List style type (disc, decimal, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_style_type: Option<String>,
    /// Whether margin-left is auto (for centering)
    #[serde(default, skip_serializing_if = "is_false")]
    pub margin_left_auto: bool,
    /// Whether margin-right is auto (for centering)
    #[serde(default, skip_serializing_if = "is_false")]
    pub margin_right_auto: bool,
    /// Padding box model (top, right, bottom, left as CSS strings)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding: Option<BoxModelSides>,
    /// Margin box model (top, right, bottom, left as CSS strings)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin: Option<BoxModelSides>,
    /// Border width per side
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_sides: Option<BoxModelSides>,
    /// CSS display value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<String>,
    /// Visibility
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub is_visible: bool,
}

/// Helper for serde skip
fn is_false(v: &bool) -> bool { !v }
fn is_true(v: &bool) -> bool { *v }
fn default_true() -> bool { true }

/// Box model sides (padding/margin/border) with values in px
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoxModelSides {
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

/// Content box dimensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Input element for layout calculation
#[derive(Debug, Clone)]
pub struct LayoutElement {
    /// Unique element ID
    pub id: String,
    /// HTML tag name
    pub tag: String,
    /// Computed CSS styles
    pub styles: ComputedStyles,
    /// Child elements
    pub children: Vec<LayoutElement>,
}

impl LayoutEngine {
    /// Create a new layout engine with default viewport
    pub fn new() -> Self {
        Self::with_viewport(1024.0, 768.0)
    }

    /// Create a new layout engine with specific viewport dimensions
    pub fn with_viewport(width: f32, height: f32) -> Self {
        Self {
            tree: TaffyTree::new(),
            node_map: HashMap::new(),
            viewport_width: width,
            viewport_height: height,
        }
    }

    /// Set viewport dimensions
    pub fn set_viewport(&mut self, width: f32, height: f32) {
        self.viewport_width = width;
        self.viewport_height = height;
    }

    /// Calculate layout for a tree of elements
    pub fn calculate_layout_from_elements(&mut self, root: &LayoutElement) -> Result<LayoutResult> {
        // Clear previous layout
        self.tree = TaffyTree::new();
        self.node_map.clear();

        // Build taffy tree from elements
        let root_node = self.build_node(root)?;

        // Compute layout
        let available_space = Size {
            width: AvailableSpace::Definite(self.viewport_width),
            height: AvailableSpace::Definite(self.viewport_height),
        };

        self.tree.compute_layout(root_node, available_space)
            .context("Failed to compute layout")?;

        // Extract layout results
        let elements = self.extract_layout(root_node, 0.0, 0.0)?;

        Ok(LayoutResult {
            width: self.viewport_width as f64,
            height: self.viewport_height as f64,
            elements,
        })
    }

    /// Build a taffy node from a LayoutElement
    fn build_node(&mut self, element: &LayoutElement) -> Result<NodeId> {
        let style = self.styles_to_taffy(&element.styles);

        // Build children first
        let children: Vec<NodeId> = element.children.iter()
            .map(|child| self.build_node(child))
            .collect::<Result<Vec<_>>>()?;

        // Create node with children
        let node = self.tree.new_with_children(
            style,
            &children,
        ).context("Failed to create taffy node")?;

        // Store node data
        self.tree.set_node_context(node, Some(LayoutNodeData {
            id: element.id.clone(),
            tag: element.tag.clone(),
            styles: element.styles.clone(),
        })).context("Failed to set node context")?;

        self.node_map.insert(element.id.clone(), node);

        Ok(node)
    }

    /// Convert computed CSS styles to taffy Style
    fn styles_to_taffy(&self, styles: &ComputedStyles) -> Style {
        let mut taffy_style = Style::default();
        let vw = self.viewport_width;
        let vh = self.viewport_height;

        // Extract font-size first so em units can be resolved in other properties
        let font_size_px = styles.font_size.as_ref()
            .and_then(|s| resolve_css_length_vp(s, ROOT_FONT_SIZE, vw, vh))
            .unwrap_or(ROOT_FONT_SIZE) as f32;

        // Display
        if let Some(ref display) = styles.display {
            taffy_style.display = match display.as_str() {
                "none" => Display::None,
                "flex" => Display::Flex,
                "grid" => Display::Grid,
                "block" => Display::Block,
                _ => Display::Block,
            };
        }

        // Position
        if let Some(ref position) = styles.position {
            taffy_style.position = match position.as_str() {
                "relative" => Position::Relative,
                "absolute" => Position::Absolute,
                _ => Position::Relative,
            };
        }

        // Width — CSS default is content-box, but taffy 0.5 only supports border-box.
        // Inflate explicit pixel widths by padding + border so taffy's border-box
        // matches the intended CSS content-box semantics.
        if let Some(ref width) = styles.width {
            let mut dim = parse_dimension(width, font_size_px, vw, vh);
            if let Dimension::Length(ref mut px) = dim {
                if let Some(ref p) = styles.padding {
                    let pl = resolve_css_length_vp(&p.left, font_size_px as f64, vw, vh).unwrap_or(0.0) as f32;
                    let pr = resolve_css_length_vp(&p.right, font_size_px as f64, vw, vh).unwrap_or(0.0) as f32;
                    *px += pl + pr;
                }
                if let Some(ref b) = styles.border {
                    let bw = resolve_css_length_vp(&b.width, font_size_px as f64, vw, vh).unwrap_or(0.0) as f32;
                    *px += 2.0 * bw;
                }
            }
            taffy_style.size.width = dim;
        }

        // Height — same content-box → border-box inflation as width above.
        if let Some(ref height) = styles.height {
            let mut dim = parse_dimension(height, font_size_px, vw, vh);
            if let Dimension::Length(ref mut px) = dim {
                if let Some(ref p) = styles.padding {
                    let pt = resolve_css_length_vp(&p.top, font_size_px as f64, vw, vh).unwrap_or(0.0) as f32;
                    let pb = resolve_css_length_vp(&p.bottom, font_size_px as f64, vw, vh).unwrap_or(0.0) as f32;
                    *px += pt + pb;
                }
                if let Some(ref b) = styles.border {
                    let bw = resolve_css_length_vp(&b.width, font_size_px as f64, vw, vh).unwrap_or(0.0) as f32;
                    *px += 2.0 * bw;
                }
            }
            taffy_style.size.height = dim;
        }

        // Flex direction
        if let Some(ref flex_dir) = styles.flex_direction {
            taffy_style.flex_direction = match flex_dir.as_str() {
                "row" => FlexDirection::Row,
                "row-reverse" => FlexDirection::RowReverse,
                "column" => FlexDirection::Column,
                "column-reverse" => FlexDirection::ColumnReverse,
                _ => FlexDirection::Row,
            };
        }

        // Justify content
        if let Some(ref justify) = styles.justify_content {
            taffy_style.justify_content = Some(match justify.as_str() {
                "flex-start" | "start" => JustifyContent::FlexStart,
                "flex-end" | "end" => JustifyContent::FlexEnd,
                "center" => JustifyContent::Center,
                "space-between" => JustifyContent::SpaceBetween,
                "space-around" => JustifyContent::SpaceAround,
                "space-evenly" => JustifyContent::SpaceEvenly,
                _ => JustifyContent::FlexStart,
            });
        }

        // Align items
        if let Some(ref align) = styles.align_items {
            taffy_style.align_items = Some(match align.as_str() {
                "flex-start" | "start" => AlignItems::FlexStart,
                "flex-end" | "end" => AlignItems::FlexEnd,
                "center" => AlignItems::Center,
                "stretch" => AlignItems::Stretch,
                "baseline" => AlignItems::Baseline,
                _ => AlignItems::Stretch,
            });
        }

        // Gap
        if let Some(ref gap) = styles.gap {
            let gap_value = parse_length_percentage(gap, font_size_px, vw, vh);
            taffy_style.gap = Size { width: gap_value, height: gap_value };
        }

        // Margin
        if let Some(ref margin) = styles.margin {
            taffy_style.margin = Rect {
                left: parse_length_percentage_auto(&margin.left, font_size_px, vw, vh),
                right: parse_length_percentage_auto(&margin.right, font_size_px, vw, vh),
                top: parse_length_percentage_auto(&margin.top, font_size_px, vw, vh),
                bottom: parse_length_percentage_auto(&margin.bottom, font_size_px, vw, vh),
            };
        }

        // Padding
        if let Some(ref padding) = styles.padding {
            taffy_style.padding = Rect {
                left: parse_length_percentage(&padding.left, font_size_px, vw, vh),
                right: parse_length_percentage(&padding.right, font_size_px, vw, vh),
                top: parse_length_percentage(&padding.top, font_size_px, vw, vh),
                bottom: parse_length_percentage(&padding.bottom, font_size_px, vw, vh),
            };
        }

        // Border
        if let Some(ref border) = styles.border {
            if let Some(w) = resolve_css_length_vp(&border.width, font_size_px as f64, vw, vh) {
                let w = w as f32;
                taffy_style.border = Rect {
                    left: LengthPercentage::Length(w),
                    right: LengthPercentage::Length(w),
                    top: LengthPercentage::Length(w),
                    bottom: LengthPercentage::Length(w),
                };
            }
        }

        // Check for additional flex properties
        if let Some(ref flex_grow) = styles.flex_grow {
            if let Ok(val) = flex_grow.parse::<f32>() {
                taffy_style.flex_grow = val;
            }
        }

        if let Some(ref flex_shrink) = styles.flex_shrink {
            if let Ok(val) = flex_shrink.parse::<f32>() {
                taffy_style.flex_shrink = val;
            }
        }

        if let Some(ref flex_basis) = styles.flex_basis {
            taffy_style.flex_basis = parse_dimension(flex_basis, font_size_px, vw, vh);
        }

        // Min/max dimensions
        if let Some(ref min_width) = styles.min_width {
            taffy_style.min_size.width = parse_dimension(min_width, font_size_px, vw, vh);
        }
        if let Some(ref max_width) = styles.max_width {
            taffy_style.max_size.width = parse_dimension(max_width, font_size_px, vw, vh);
        }
        if let Some(ref min_height) = styles.min_height {
            taffy_style.min_size.height = parse_dimension(min_height, font_size_px, vw, vh);
        }
        if let Some(ref max_height) = styles.max_height {
            taffy_style.max_size.height = parse_dimension(max_height, font_size_px, vw, vh);
        }

        taffy_style
    }

}

/// Parse a CSS dimension (px, em, rem, %, vw, vh, auto, etc.) to taffy Dimension
fn parse_dimension(value: &str, font_size_px: f32, vw: f32, vh: f32) -> Dimension {
    let value = value.trim();

    if value == "auto" || value.is_empty() {
        return Dimension::Auto;
    }

    if value.ends_with('%') {
        if let Ok(pct) = value.trim_end_matches('%').parse::<f32>() {
            return Dimension::Percent(pct / 100.0);
        }
    }

    if let Some(px) = resolve_css_length_vp(value, font_size_px as f64, vw, vh) {
        return Dimension::Length(px as f32);
    }

    Dimension::Auto
}

/// Parse a CSS length/percentage value (px, em, rem, %, vw, vh)
fn parse_length_percentage(value: &str, font_size_px: f32, vw: f32, vh: f32) -> LengthPercentage {
    let value = value.trim();

    if value.ends_with('%') {
        if let Ok(pct) = value.trim_end_matches('%').parse::<f32>() {
            return LengthPercentage::Percent(pct / 100.0);
        }
    }

    if let Some(px) = resolve_css_length_vp(value, font_size_px as f64, vw, vh) {
        return LengthPercentage::Length(px as f32);
    }

    LengthPercentage::Length(0.0)
}

/// Parse a CSS length/percentage/auto value (px, em, rem, %, vw, vh, auto)
fn parse_length_percentage_auto(value: &str, font_size_px: f32, vw: f32, vh: f32) -> LengthPercentageAuto {
    let value = value.trim();

    if value == "auto" || value.is_empty() {
        return LengthPercentageAuto::Auto;
    }

    if value.ends_with('%') {
        if let Ok(pct) = value.trim_end_matches('%').parse::<f32>() {
            return LengthPercentageAuto::Percent(pct / 100.0);
        }
    }

    if let Some(px) = resolve_css_length_vp(value, font_size_px as f64, vw, vh) {
        return LengthPercentageAuto::Length(px as f32);
    }

    LengthPercentageAuto::Length(0.0)
}

impl LayoutEngine {
    /// Extract layout results from the computed taffy tree
    fn extract_layout(&self, node: NodeId, parent_x: f64, parent_y: f64) -> Result<Vec<ElementLayout>> {
        let layout = self.tree.layout(node)
            .context("Failed to get layout for node")?;

        let node_data = self.tree.get_node_context(node)
            .context("Failed to get node context")?;

        let x = parent_x + layout.location.x as f64;
        let y = parent_y + layout.location.y as f64;
        let width = layout.size.width as f64;
        let height = layout.size.height as f64;
        let vw = self.viewport_width;
        let vh = self.viewport_height;

        // Get children layouts
        let children_ids = self.tree.children(node)
            .context("Failed to get children")?;

        let children: Vec<ElementLayout> = children_ids.iter()
            .flat_map(|&child_id| self.extract_layout(child_id, x, y).unwrap_or_default())
            .collect();

        // Calculate content box by subtracting both border and padding from border-box
        let content_box = Some(ContentBox {
            x: x + layout.border.left as f64 + layout.padding.left as f64,
            y: y + layout.border.top as f64 + layout.padding.top as f64,
            width: width - (layout.border.left + layout.padding.left + layout.padding.right + layout.border.right) as f64,
            height: height - (layout.border.top + layout.padding.top + layout.padding.bottom + layout.border.bottom) as f64,
        });

        // Extract visual properties from computed styles
        let styles = &node_data.styles;

        // Resolve font-size for em unit conversion in output properties
        let font_size_px = styles.font_size.as_ref()
            .and_then(|s| resolve_css_length_vp(s, ROOT_FONT_SIZE, vw, vh))
            .unwrap_or(ROOT_FONT_SIZE);

        Ok(vec![ElementLayout {
            id: node_data.id.clone(),
            tag: node_data.tag.clone(),
            x,
            y,
            width,
            height,
            content_box,
            children,
            // Visual properties — populated from ComputedStyles
            text_content: None, // filled by page_layout
            link_href: None,
            img_src: None,
            background_color: styles.background_color.clone(),
            color: styles.color.clone(),
            font_size: Some(font_size_px),
            font_family: styles.font_family.clone(),
            font_weight: styles.font_weight.clone(),
            font_style: styles.font_style.clone(),
            text_align: styles.text_align.clone(),
            text_decoration: styles.text_decoration.clone(),
            line_height: styles.line_height.as_ref().and_then(|s| resolve_css_length_vp(s, font_size_px, vw, vh).or_else(|| s.parse::<f64>().ok())),
            white_space: styles.white_space.clone(),
            border_radius: styles.border_radius.as_ref().and_then(|s| resolve_css_length_vp(s, font_size_px, vw, vh)),
            border_width: styles.border.as_ref().and_then(|b| resolve_css_length_vp(&b.width, font_size_px, vw, vh)),
            border_color: styles.border.as_ref().map(|b| b.color.clone()),
            opacity: styles.opacity,
            overflow: styles.overflow.clone(),
            list_style_type: styles.list_style_type.clone(),
            margin_left_auto: styles.margin.as_ref().map(|m| m.left == "auto").unwrap_or(false),
            margin_right_auto: styles.margin.as_ref().map(|m| m.right == "auto").unwrap_or(false),
            padding: styles.padding.as_ref().map(|p| BoxModelSides {
                top: resolve_css_length_vp(&p.top, font_size_px, vw, vh).unwrap_or(0.0),
                right: resolve_css_length_vp(&p.right, font_size_px, vw, vh).unwrap_or(0.0),
                bottom: resolve_css_length_vp(&p.bottom, font_size_px, vw, vh).unwrap_or(0.0),
                left: resolve_css_length_vp(&p.left, font_size_px, vw, vh).unwrap_or(0.0),
            }),
            margin: styles.margin.as_ref().map(|m| BoxModelSides {
                top: resolve_css_length_vp(&m.top, font_size_px, vw, vh).unwrap_or(0.0),
                right: resolve_css_length_vp(&m.right, font_size_px, vw, vh).unwrap_or(0.0),
                bottom: resolve_css_length_vp(&m.bottom, font_size_px, vw, vh).unwrap_or(0.0),
                left: resolve_css_length_vp(&m.left, font_size_px, vw, vh).unwrap_or(0.0),
            }),
            border_sides: styles.border.as_ref().and_then(|b| {
                resolve_css_length_vp(&b.width, font_size_px, vw, vh).map(|w| BoxModelSides {
                    top: w, right: w, bottom: w, left: w,
                })
            }),
            display: styles.display.clone(),
            is_visible: styles.visibility.as_ref().map(|v| v != "hidden" && v != "collapse").unwrap_or(true)
                && styles.display.as_ref().map(|d| d != "none").unwrap_or(true),
        }])
    }

    /// Legacy method for backward compatibility - parse HTML/CSS and compute layout
    pub fn calculate_layout(&self, _html: &str, _css: &str) -> Result<LayoutResult> {
        // This is a stub for backward compatibility
        // Real implementation would need to:
        // 1. Parse HTML to get DOM tree
        // 2. Parse CSS and apply to DOM
        // 3. Convert to LayoutElement tree
        // 4. Call calculate_layout_from_elements

        let mut root = ElementLayout::new_empty("root".to_string(), "html".to_string());
        root.width = self.viewport_width as f64;
        root.height = self.viewport_height as f64;

        let mut body = ElementLayout::new_empty("body".to_string(), "body".to_string());
        body.width = self.viewport_width as f64;
        body.height = self.viewport_height as f64;
        root.children.push(body);

        Ok(LayoutResult {
            width: self.viewport_width as f64,
            height: self.viewport_height as f64,
            elements: vec![root],
        })
    }

    /// Get layout for a specific element by ID
    pub fn get_element_layout(&self, element_id: &str) -> Option<&Layout> {
        self.node_map.get(element_id)
            .and_then(|node_id| self.tree.layout(*node_id).ok())
    }
}

impl Default for LayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Default root font size in px (used for rem units)
const ROOT_FONT_SIZE: f64 = 16.0;

/// Resolve a CSS length value to pixels, handling px, em, rem, pt, vw, vh, vmin, vmax,
/// and bare numbers.
/// - `em` is relative to `font_size_px` (the element's computed font-size)
/// - `rem` is relative to the root font-size (16px)
/// - `pt` is converted at 1pt = 1.333px
/// - `vw` is 1% of viewport width, `vh` is 1% of viewport height
/// - `vmin` is min(vw, vh), `vmax` is max(vw, vh)
/// Returns None for "auto", "none", empty strings, or unparseable values.
///
/// When viewport dimensions are not available (0.0), viewport units return None.
pub fn resolve_css_length(value: &str, font_size_px: f64) -> Option<f64> {
    resolve_css_length_vp(value, font_size_px, 0.0, 0.0)
}

/// Like `resolve_css_length` but with viewport dimensions for vw/vh/vmin/vmax resolution.
pub fn resolve_css_length_vp(value: &str, font_size_px: f64, viewport_w: f32, viewport_h: f32) -> Option<f64> {
    let v = value.trim();
    if v.is_empty() || v == "auto" || v == "none" || v == "initial" || v == "inherit" {
        return None;
    }

    if v.ends_with("px") {
        return v.trim_end_matches("px").parse::<f64>().ok();
    }

    if v.ends_with("rem") {
        return v.trim_end_matches("rem").parse::<f64>().ok()
            .map(|n| n * ROOT_FONT_SIZE);
    }

    if v.ends_with("em") {
        return v.trim_end_matches("em").parse::<f64>().ok()
            .map(|n| n * font_size_px);
    }

    if v.ends_with("pt") {
        return v.trim_end_matches("pt").parse::<f64>().ok()
            .map(|n| n * 1.333);
    }

    if v.ends_with("vmin") {
        if viewport_w <= 0.0 && viewport_h <= 0.0 { return None; }
        let vmin = (viewport_w as f64).min(viewport_h as f64);
        return v.trim_end_matches("vmin").parse::<f64>().ok()
            .map(|n| n * vmin / 100.0);
    }

    if v.ends_with("vmax") {
        if viewport_w <= 0.0 && viewport_h <= 0.0 { return None; }
        let vmax = (viewport_w as f64).max(viewport_h as f64);
        return v.trim_end_matches("vmax").parse::<f64>().ok()
            .map(|n| n * vmax / 100.0);
    }

    if v.ends_with("vw") {
        if viewport_w <= 0.0 { return None; }
        return v.trim_end_matches("vw").parse::<f64>().ok()
            .map(|n| n * viewport_w as f64 / 100.0);
    }

    if v.ends_with("vh") {
        if viewport_h <= 0.0 { return None; }
        return v.trim_end_matches("vh").parse::<f64>().ok()
            .map(|n| n * viewport_h as f64 / 100.0);
    }

    // Try bare number (assumed px)
    v.parse::<f64>().ok()
}

/// Parse a CSS value to a pixel number.
/// Handles px, em, rem, pt, bare numbers. Returns None for non-numeric or "auto".
pub fn parse_px_value(value: &str) -> Option<f64> {
    resolve_css_length(value, ROOT_FONT_SIZE)
}

impl ElementLayout {
    /// Create a default/empty ElementLayout for constructing trees
    pub fn new_empty(id: String, tag: String) -> Self {
        Self {
            id,
            tag,
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
            content_box: None,
            children: Vec::new(),
            text_content: None,
            link_href: None,
            img_src: None,
            background_color: None,
            color: None,
            font_size: None,
            font_family: None,
            font_weight: None,
            font_style: None,
            text_align: None,
            text_decoration: None,
            line_height: None,
            white_space: None,
            border_radius: None,
            border_width: None,
            border_color: None,
            opacity: None,
            overflow: None,
            list_style_type: None,
            margin_left_auto: false,
            margin_right_auto: false,
            padding: None,
            margin: None,
            border_sides: None,
            display: None,
            is_visible: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_layout() {
        let mut engine = LayoutEngine::with_viewport(800.0, 600.0);

        let root = LayoutElement {
            id: "root".to_string(),
            tag: "div".to_string(),
            styles: ComputedStyles {
                display: Some("flex".to_string()),
                width: Some("100%".to_string()),
                height: Some("100%".to_string()),
                flex_direction: Some("column".to_string()),
                ..Default::default()
            },
            children: vec![
                LayoutElement {
                    id: "header".to_string(),
                    tag: "header".to_string(),
                    styles: ComputedStyles {
                        height: Some("100px".to_string()),
                        ..Default::default()
                    },
                    children: vec![],
                },
                LayoutElement {
                    id: "content".to_string(),
                    tag: "main".to_string(),
                    styles: ComputedStyles {
                        other: [("flex-grow".to_string(), "1".to_string())].into_iter().collect(),
                        ..Default::default()
                    },
                    children: vec![],
                },
            ],
        };

        let result = engine.calculate_layout_from_elements(&root).unwrap();

        assert_eq!(result.width, 800.0);
        assert_eq!(result.height, 600.0);
        assert!(!result.elements.is_empty());
    }

    #[test]
    fn test_flexbox_row() {
        let mut engine = LayoutEngine::with_viewport(300.0, 100.0);

        let root = LayoutElement {
            id: "row".to_string(),
            tag: "div".to_string(),
            styles: ComputedStyles {
                display: Some("flex".to_string()),
                width: Some("300px".to_string()),
                height: Some("100px".to_string()),
                flex_direction: Some("row".to_string()),
                ..Default::default()
            },
            children: vec![
                LayoutElement {
                    id: "item1".to_string(),
                    tag: "div".to_string(),
                    styles: ComputedStyles {
                        width: Some("100px".to_string()),
                        ..Default::default()
                    },
                    children: vec![],
                },
                LayoutElement {
                    id: "item2".to_string(),
                    tag: "div".to_string(),
                    styles: ComputedStyles {
                        width: Some("100px".to_string()),
                        ..Default::default()
                    },
                    children: vec![],
                },
                LayoutElement {
                    id: "item3".to_string(),
                    tag: "div".to_string(),
                    styles: ComputedStyles {
                        width: Some("100px".to_string()),
                        ..Default::default()
                    },
                    children: vec![],
                },
            ],
        };

        let result = engine.calculate_layout_from_elements(&root).unwrap();
        assert_eq!(result.elements.len(), 1);
        assert_eq!(result.elements[0].children.len(), 3);
    }

    #[test]
    fn test_parse_dimensions() {
        // Test px
        assert!(matches!(super::parse_dimension("100px", 16.0, 1024.0, 768.0), Dimension::Length(v) if (v - 100.0).abs() < 0.01));

        // Test percent
        assert!(matches!(super::parse_dimension("50%", 16.0, 1024.0, 768.0), Dimension::Percent(v) if (v - 0.5).abs() < 0.01));

        // Test auto
        assert!(matches!(super::parse_dimension("auto", 16.0, 1024.0, 768.0), Dimension::Auto));

        // Test em
        assert!(matches!(super::parse_dimension("2em", 16.0, 1024.0, 768.0), Dimension::Length(v) if (v - 32.0).abs() < 0.01));

        // Test rem
        assert!(matches!(super::parse_dimension("1.5rem", 24.0, 1024.0, 768.0), Dimension::Length(v) if (v - 24.0).abs() < 0.01));

        // Test vw
        assert!(matches!(super::parse_dimension("60vw", 16.0, 1024.0, 768.0), Dimension::Length(v) if (v - 614.4).abs() < 0.1));

        // Test vh
        assert!(matches!(super::parse_dimension("15vh", 16.0, 1024.0, 768.0), Dimension::Length(v) if (v - 115.2).abs() < 0.1));
    }
}
