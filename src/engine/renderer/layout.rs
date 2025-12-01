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

/// Layout information for a single element
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
        *self.tree.get_node_context_mut(node).unwrap() = LayoutNodeData {
            id: element.id.clone(),
            tag: element.tag.clone(),
            styles: element.styles.clone(),
        };

        self.node_map.insert(element.id.clone(), node);

        Ok(node)
    }

    /// Convert computed CSS styles to taffy Style
    fn styles_to_taffy(&self, styles: &ComputedStyles) -> Style {
        let mut taffy_style = Style::default();

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

        // Width
        if let Some(ref width) = styles.width {
            taffy_style.size.width = self.parse_dimension(width);
        }

        // Height
        if let Some(ref height) = styles.height {
            taffy_style.size.height = self.parse_dimension(height);
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
            let gap_value = self.parse_length_percentage(gap);
            taffy_style.gap = Size { width: gap_value, height: gap_value };
        }

        // Margin
        if let Some(ref margin) = styles.margin {
            taffy_style.margin = Rect {
                left: self.parse_length_percentage_auto(&margin.left),
                right: self.parse_length_percentage_auto(&margin.right),
                top: self.parse_length_percentage_auto(&margin.top),
                bottom: self.parse_length_percentage_auto(&margin.bottom),
            };
        }

        // Padding
        if let Some(ref padding) = styles.padding {
            taffy_style.padding = Rect {
                left: self.parse_length_percentage(&padding.left),
                right: self.parse_length_percentage(&padding.right),
                top: self.parse_length_percentage(&padding.top),
                bottom: self.parse_length_percentage(&padding.bottom),
            };
        }

        // Check for additional flex properties in 'other'
        if let Some(flex_grow) = styles.other.get("flex-grow") {
            if let Ok(val) = flex_grow.parse::<f32>() {
                taffy_style.flex_grow = val;
            }
        }

        if let Some(flex_shrink) = styles.other.get("flex-shrink") {
            if let Ok(val) = flex_shrink.parse::<f32>() {
                taffy_style.flex_shrink = val;
            }
        }

        if let Some(flex_basis) = styles.other.get("flex-basis") {
            taffy_style.flex_basis = self.parse_dimension(flex_basis);
        }

        // Min/max dimensions
        if let Some(min_width) = styles.other.get("min-width") {
            taffy_style.min_size.width = self.parse_dimension(min_width);
        }
        if let Some(max_width) = styles.other.get("max-width") {
            taffy_style.max_size.width = self.parse_dimension(max_width);
        }
        if let Some(min_height) = styles.other.get("min-height") {
            taffy_style.min_size.height = self.parse_dimension(min_height);
        }
        if let Some(max_height) = styles.other.get("max-height") {
            taffy_style.max_size.height = self.parse_dimension(max_height);
        }

        taffy_style
    }

    /// Parse a CSS dimension (px, %, auto, etc.) to taffy Dimension
    fn parse_dimension(&self, value: &str) -> Dimension {
        let value = value.trim();

        if value == "auto" || value.is_empty() {
            return Dimension::Auto;
        }

        if value.ends_with('%') {
            if let Ok(pct) = value.trim_end_matches('%').parse::<f32>() {
                return Dimension::Percent(pct / 100.0);
            }
        }

        if value.ends_with("px") {
            if let Ok(px) = value.trim_end_matches("px").parse::<f32>() {
                return Dimension::Length(px);
            }
        }

        // Try parsing as bare number (assumed px)
        if let Ok(px) = value.parse::<f32>() {
            return Dimension::Length(px);
        }

        Dimension::Auto
    }

    /// Parse a CSS length/percentage value
    fn parse_length_percentage(&self, value: &str) -> LengthPercentage {
        let value = value.trim();

        if value.ends_with('%') {
            if let Ok(pct) = value.trim_end_matches('%').parse::<f32>() {
                return LengthPercentage::Percent(pct / 100.0);
            }
        }

        if value.ends_with("px") {
            if let Ok(px) = value.trim_end_matches("px").parse::<f32>() {
                return LengthPercentage::Length(px);
            }
        }

        // Try parsing as bare number
        if let Ok(px) = value.parse::<f32>() {
            return LengthPercentage::Length(px);
        }

        LengthPercentage::Length(0.0)
    }

    /// Parse a CSS length/percentage/auto value
    fn parse_length_percentage_auto(&self, value: &str) -> LengthPercentageAuto {
        let value = value.trim();

        if value == "auto" || value.is_empty() {
            return LengthPercentageAuto::Auto;
        }

        if value.ends_with('%') {
            if let Ok(pct) = value.trim_end_matches('%').parse::<f32>() {
                return LengthPercentageAuto::Percent(pct / 100.0);
            }
        }

        if value.ends_with("px") {
            if let Ok(px) = value.trim_end_matches("px").parse::<f32>() {
                return LengthPercentageAuto::Length(px);
            }
        }

        if let Ok(px) = value.parse::<f32>() {
            return LengthPercentageAuto::Length(px);
        }

        LengthPercentageAuto::Length(0.0)
    }

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

        // Get children layouts
        let children_ids = self.tree.children(node)
            .context("Failed to get children")?;

        let children: Vec<ElementLayout> = children_ids.iter()
            .flat_map(|&child_id| self.extract_layout(child_id, x, y).unwrap_or_default())
            .collect();

        // Calculate content box (layout.content_box is available in taffy)
        let content_box = Some(ContentBox {
            x: x + layout.padding.left as f64,
            y: y + layout.padding.top as f64,
            width: width - (layout.padding.left + layout.padding.right) as f64,
            height: height - (layout.padding.top + layout.padding.bottom) as f64,
        });

        Ok(vec![ElementLayout {
            id: node_data.id.clone(),
            tag: node_data.tag.clone(),
            x,
            y,
            width,
            height,
            content_box,
            children,
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

        Ok(LayoutResult {
            width: self.viewport_width as f64,
            height: self.viewport_height as f64,
            elements: vec![
                ElementLayout {
                    id: "root".to_string(),
                    tag: "html".to_string(),
                    x: 0.0,
                    y: 0.0,
                    width: self.viewport_width as f64,
                    height: self.viewport_height as f64,
                    content_box: None,
                    children: vec![
                        ElementLayout {
                            id: "body".to_string(),
                            tag: "body".to_string(),
                            x: 0.0,
                            y: 0.0,
                            width: self.viewport_width as f64,
                            height: self.viewport_height as f64,
                            content_box: None,
                            children: vec![],
                        }
                    ],
                }
            ],
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
        let engine = LayoutEngine::new();

        // Test px
        assert!(matches!(engine.parse_dimension("100px"), Dimension::Length(v) if (v - 100.0).abs() < 0.01));

        // Test percent
        assert!(matches!(engine.parse_dimension("50%"), Dimension::Percent(v) if (v - 0.5).abs() < 0.01));

        // Test auto
        assert!(matches!(engine.parse_dimension("auto"), Dimension::Auto));
    }
}
