//! CSS Engine with lightningcss integration
//!
//! Provides CSS parsing, style computation, and minification using the
//! lightningcss library for high-performance CSS processing.

use anyhow::{Result, Context};
use lightningcss::stylesheet::{StyleSheet, ParserOptions, MinifyOptions, PrinterOptions};
use lightningcss::rules::CssRule;
use lightningcss::properties::Property;
use lightningcss::declaration::DeclarationBlock;
use lightningcss::selector::{Selector, Component};
use lightningcss::traits::ToCss;
use lightningcss::targets::{Browsers, Targets};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Computed CSS styles for an element
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComputedStyles {
    /// Display property (block, flex, grid, inline, etc.)
    pub display: Option<String>,
    /// Position property (static, relative, absolute, fixed, sticky)
    pub position: Option<String>,
    /// Width (px, %, auto, etc.)
    pub width: Option<String>,
    /// Height
    pub height: Option<String>,
    /// Margin (top, right, bottom, left)
    pub margin: Option<BoxModel>,
    /// Padding
    pub padding: Option<BoxModel>,
    /// Border
    pub border: Option<BorderStyles>,
    /// Background color
    pub background_color: Option<String>,
    /// Text color
    pub color: Option<String>,
    /// Font size
    pub font_size: Option<String>,
    /// Font family
    pub font_family: Option<String>,
    /// Font weight
    pub font_weight: Option<String>,
    /// Flex direction
    pub flex_direction: Option<String>,
    /// Justify content
    pub justify_content: Option<String>,
    /// Align items
    pub align_items: Option<String>,
    /// Gap
    pub gap: Option<String>,
    /// Overflow
    pub overflow: Option<String>,
    /// Z-index
    pub z_index: Option<i32>,
    /// Opacity
    pub opacity: Option<f32>,
    /// Visibility
    pub visibility: Option<String>,
    /// All other properties as key-value pairs
    #[serde(flatten)]
    pub other: HashMap<String, String>,
}

/// Box model values (margin, padding)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BoxModel {
    pub top: String,
    pub right: String,
    pub bottom: String,
    pub left: String,
}

/// Border styles
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BorderStyles {
    pub width: String,
    pub style: String,
    pub color: String,
}

/// A parsed CSS rule with selector and declarations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedRule {
    /// CSS selector string
    pub selector: String,
    /// Specificity (a, b, c) tuple
    pub specificity: (u32, u32, u32),
    /// Declarations (property: value)
    pub declarations: HashMap<String, String>,
}

/// CSS processor for handling CSS parsing, computation, and optimization
pub struct CssProcessor {
    /// Parsed rules from all stylesheets
    rules: Vec<ParsedRule>,
    /// Raw stylesheet sources for reference
    sources: Vec<String>,
}

impl CssProcessor {
    /// Create a new CSS processor
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            sources: Vec::new(),
        }
    }

    /// Parse a CSS string and add its rules to the processor
    pub fn parse(&mut self, css: &str) -> Result<()> {
        self.sources.push(css.to_string());

        let stylesheet = StyleSheet::parse(css, ParserOptions::default())
            .map_err(|e| anyhow::anyhow!("CSS parse error: {:?}", e))?;

        // Extract rules from the stylesheet
        for rule in stylesheet.rules.0.iter() {
            if let CssRule::Style(style_rule) = rule {
                // Get selector string
                let selector_str = style_rule.selectors.to_css_string(PrinterOptions::default())
                    .unwrap_or_default();

                // Calculate specificity (simplified)
                let specificity = self.calculate_specificity(&selector_str);

                // Extract declarations
                let mut declarations = HashMap::new();
                for decl in style_rule.declarations.declarations.iter() {
                    let prop_name = decl.property_id().to_css_string(PrinterOptions::default())
                        .unwrap_or_default();
                    let prop_value = decl.value_to_css_string(PrinterOptions::default())
                        .unwrap_or_default();
                    declarations.insert(prop_name, prop_value);
                }

                // Also handle important declarations
                for decl in style_rule.declarations.important_declarations.iter() {
                    let prop_name = decl.property_id().to_css_string(PrinterOptions::default())
                        .unwrap_or_default();
                    let prop_value = format!("{} !important",
                        decl.value_to_css_string(PrinterOptions::default()).unwrap_or_default());
                    declarations.insert(prop_name, prop_value);
                }

                self.rules.push(ParsedRule {
                    selector: selector_str,
                    specificity,
                    declarations,
                });
            }
        }

        Ok(())
    }

    /// Parse CSS and add rules from inline style attribute
    pub fn parse_inline_style(&mut self, style: &str, element_id: &str) -> Result<()> {
        // Wrap inline style in a rule targeting the element
        let css = format!("#{} {{ {} }}", element_id, style);
        self.parse(&css)
    }

    /// Get all rules that match a given selector
    pub fn get_matching_rules(&self, selector: &str) -> Vec<&ParsedRule> {
        self.rules.iter()
            .filter(|rule| self.selectors_match(&rule.selector, selector))
            .collect()
    }

    /// Compute styles for an element given its selector chain
    /// selector_chain is a list of selectors from root to the element (e.g., ["html", "body", "div.container", "p"])
    pub fn compute_style(&self, selector: &str) -> ComputedStyles {
        let mut styles = ComputedStyles::default();

        // Collect all matching rules sorted by specificity
        let mut matching_rules: Vec<&ParsedRule> = self.rules.iter()
            .filter(|rule| self.selector_applies(&rule.selector, selector))
            .collect();

        // Sort by specificity (lower first, so higher specificity overrides)
        matching_rules.sort_by(|a, b| a.specificity.cmp(&b.specificity));

        // Apply declarations in order
        for rule in matching_rules {
            self.apply_declarations(&mut styles, &rule.declarations);
        }

        styles
    }

    /// Get computed style for a specific property
    pub fn get_property(&self, selector: &str, property: &str) -> Option<String> {
        let styles = self.compute_style(selector);

        // Check known properties first
        match property {
            "display" => styles.display,
            "position" => styles.position,
            "width" => styles.width,
            "height" => styles.height,
            "background-color" => styles.background_color,
            "color" => styles.color,
            "font-size" => styles.font_size,
            "font-family" => styles.font_family,
            "font-weight" => styles.font_weight,
            "flex-direction" => styles.flex_direction,
            "justify-content" => styles.justify_content,
            "align-items" => styles.align_items,
            "gap" => styles.gap,
            "overflow" => styles.overflow,
            "visibility" => styles.visibility,
            "opacity" => styles.opacity.map(|o| o.to_string()),
            "z-index" => styles.z_index.map(|z| z.to_string()),
            _ => styles.other.get(property).cloned(),
        }
    }

    /// Minify CSS using lightningcss
    pub fn minify(&self, css: &str) -> Result<String> {
        let stylesheet = StyleSheet::parse(css, ParserOptions::default())
            .map_err(|e| anyhow::anyhow!("CSS parse error: {:?}", e))?;

        let minified = stylesheet.to_css(PrinterOptions {
            minify: true,
            ..Default::default()
        }).map_err(|e| anyhow::anyhow!("CSS minify error: {:?}", e))?;

        Ok(minified.code)
    }

    /// Process CSS with vendor prefixes and minification
    pub fn process_css(&self, css: &str) -> Result<String> {
        let stylesheet = StyleSheet::parse(css, ParserOptions::default())
            .map_err(|e| anyhow::anyhow!("CSS parse error: {:?}", e))?;

        // Use default browser targets - browserslist integration requires additional feature
        let targets = Targets::default();

        let result = stylesheet.to_css(PrinterOptions {
            minify: false,
            targets,
            ..Default::default()
        }).map_err(|e| anyhow::anyhow!("CSS processing error: {:?}", e))?;

        Ok(result.code)
    }

    /// Process CSS with specific browser targets
    pub fn process_css_with_targets(&self, css: &str, browsers: Browsers) -> Result<String> {
        let stylesheet = StyleSheet::parse(css, ParserOptions::default())
            .map_err(|e| anyhow::anyhow!("CSS parse error: {:?}", e))?;

        let result = stylesheet.to_css(PrinterOptions {
            minify: false,
            targets: Targets::from(browsers),
            ..Default::default()
        }).map_err(|e| anyhow::anyhow!("CSS processing error: {:?}", e))?;

        Ok(result.code)
    }

    /// Get all parsed rules
    pub fn get_rules(&self) -> &[ParsedRule] {
        &self.rules
    }

    /// Clear all parsed rules
    pub fn clear(&mut self) {
        self.rules.clear();
        self.sources.clear();
    }

    /// Calculate selector specificity (simplified)
    fn calculate_specificity(&self, selector: &str) -> (u32, u32, u32) {
        let mut elements = 0u32;

        // Count IDs (#)
        let ids = selector.matches('#').count() as u32;

        // Count classes (.), attributes ([]), and pseudo-classes (:not ::)
        let mut classes = selector.matches('.').count() as u32;
        classes += selector.matches('[').count() as u32;
        // Count single colons not followed by another colon (pseudo-classes)
        for (i, c) in selector.char_indices() {
            if c == ':' {
                let next = selector.chars().nth(i + 1);
                if next != Some(':') {
                    classes += 1;
                }
            }
        }

        // Count element selectors (rough approximation)
        let parts: Vec<&str> = selector.split(|c: char| c.is_whitespace() || c == '>' || c == '+' || c == '~')
            .filter(|s| !s.is_empty())
            .collect();

        for part in parts {
            // Skip if starts with # or .
            if !part.starts_with('#') && !part.starts_with('.') && !part.starts_with('[') && !part.starts_with(':') {
                // It's an element selector
                let elem_part = part.split(|c| c == '#' || c == '.' || c == '[' || c == ':').next().unwrap_or("");
                if !elem_part.is_empty() && elem_part != "*" {
                    elements += 1;
                }
            }
        }

        (ids, classes, elements)
    }

    /// Check if two selectors match (simplified)
    fn selectors_match(&self, rule_selector: &str, target_selector: &str) -> bool {
        // Simple exact match for now
        rule_selector == target_selector
    }

    /// Strip pseudo-classes and pseudo-elements from a selector for matching.
    /// e.g. "a:link" → "a", "a:hover::after" → "a"
    fn strip_pseudos(selector: &str) -> &str {
        // Find the first ':' that isn't escaped
        if let Some(idx) = selector.find(':') {
            &selector[..idx]
        } else {
            selector
        }
    }

    /// Check if a rule selector applies to a target element
    fn selector_applies(&self, rule_selector: &str, target: &str) -> bool {
        // Split rule selector by comma for multiple selectors
        for raw_selector in rule_selector.split(',').map(|s| s.trim()) {
            // Strip pseudo-classes (:link, :visited, :hover, ::before, etc.)
            let selector = Self::strip_pseudos(raw_selector);
            if selector.is_empty() {
                continue;
            }

            // Check element type match
            if selector == target {
                return true;
            }

            // Check class match (e.g., ".container" matches "div.container")
            if selector.starts_with('.') && target.contains(selector) {
                return true;
            }

            // Check ID match
            if selector.starts_with('#') && target.contains(selector) {
                return true;
            }

            // Check element match (e.g., "div" matches "div.container")
            let elem = target.split(|c| c == '.' || c == '#' || c == '[').next().unwrap_or("");
            if selector == elem {
                return true;
            }
        }

        false
    }

    /// Apply declarations to computed styles
    fn apply_declarations(&self, styles: &mut ComputedStyles, declarations: &HashMap<String, String>) {
        for (prop, value) in declarations {
            // Remove !important suffix for storage
            let clean_value = value.trim_end_matches(" !important").to_string();

            match prop.as_str() {
                "display" => styles.display = Some(clean_value),
                "position" => styles.position = Some(clean_value),
                "width" => styles.width = Some(clean_value),
                "height" => styles.height = Some(clean_value),
                "background-color" => styles.background_color = Some(clean_value),
                "background" => {
                    // The background shorthand can include color, image, position, etc.
                    // Extract just the color if it's a simple color value.
                    // lightningcss often splits background into component properties,
                    // but when it outputs the shorthand, the color is usually the first value.
                    let v = clean_value.trim();
                    // If it looks like a color value (hex, rgb, named color), use it
                    if v.starts_with('#') || v.starts_with("rgb") || v.starts_with("hsl")
                        || v == "transparent" || v == "inherit"
                        || (!v.contains(' ') && !v.starts_with("url"))
                    {
                        styles.background_color = Some(clean_value);
                    } else {
                        // Store full background in other for potential future use
                        styles.other.insert("background".to_string(), clean_value);
                    }
                },
                "color" => styles.color = Some(clean_value),
                "font-size" => styles.font_size = Some(clean_value),
                "font-family" => styles.font_family = Some(clean_value),
                "font-weight" => styles.font_weight = Some(clean_value),
                "flex-direction" => styles.flex_direction = Some(clean_value),
                "justify-content" => styles.justify_content = Some(clean_value),
                "align-items" => styles.align_items = Some(clean_value),
                "gap" => styles.gap = Some(clean_value),
                "overflow" => styles.overflow = Some(clean_value),
                "visibility" => styles.visibility = Some(clean_value),
                "opacity" => {
                    if let Ok(val) = clean_value.parse::<f32>() {
                        styles.opacity = Some(val);
                    }
                },
                "z-index" => {
                    if let Ok(val) = clean_value.parse::<i32>() {
                        styles.z_index = Some(val);
                    }
                },
                "margin" => {
                    styles.margin = Some(self.parse_box_model(&clean_value));
                },
                "margin-top" => {
                    let mut margin = styles.margin.clone().unwrap_or_default();
                    margin.top = clean_value;
                    styles.margin = Some(margin);
                },
                "margin-right" => {
                    let mut margin = styles.margin.clone().unwrap_or_default();
                    margin.right = clean_value;
                    styles.margin = Some(margin);
                },
                "margin-bottom" => {
                    let mut margin = styles.margin.clone().unwrap_or_default();
                    margin.bottom = clean_value;
                    styles.margin = Some(margin);
                },
                "margin-left" => {
                    let mut margin = styles.margin.clone().unwrap_or_default();
                    margin.left = clean_value;
                    styles.margin = Some(margin);
                },
                "padding" => {
                    styles.padding = Some(self.parse_box_model(&clean_value));
                },
                "padding-top" => {
                    let mut padding = styles.padding.clone().unwrap_or_default();
                    padding.top = clean_value;
                    styles.padding = Some(padding);
                },
                "padding-right" => {
                    let mut padding = styles.padding.clone().unwrap_or_default();
                    padding.right = clean_value;
                    styles.padding = Some(padding);
                },
                "padding-bottom" => {
                    let mut padding = styles.padding.clone().unwrap_or_default();
                    padding.bottom = clean_value;
                    styles.padding = Some(padding);
                },
                "padding-left" => {
                    let mut padding = styles.padding.clone().unwrap_or_default();
                    padding.left = clean_value;
                    styles.padding = Some(padding);
                },
                _ => {
                    styles.other.insert(prop.clone(), clean_value);
                }
            }
        }
    }

    /// Parse shorthand box model values (margin/padding)
    fn parse_box_model(&self, value: &str) -> BoxModel {
        let parts: Vec<&str> = value.split_whitespace().collect();
        match parts.len() {
            1 => BoxModel {
                top: parts[0].to_string(),
                right: parts[0].to_string(),
                bottom: parts[0].to_string(),
                left: parts[0].to_string(),
            },
            2 => BoxModel {
                top: parts[0].to_string(),
                right: parts[1].to_string(),
                bottom: parts[0].to_string(),
                left: parts[1].to_string(),
            },
            3 => BoxModel {
                top: parts[0].to_string(),
                right: parts[1].to_string(),
                bottom: parts[2].to_string(),
                left: parts[1].to_string(),
            },
            4 => BoxModel {
                top: parts[0].to_string(),
                right: parts[1].to_string(),
                bottom: parts[2].to_string(),
                left: parts[3].to_string(),
            },
            _ => BoxModel::default(),
        }
    }
}

impl Default for CssProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_css() {
        let mut processor = CssProcessor::new();
        let css = r#"
            .container {
                display: flex;
                width: 100%;
                padding: 20px;
            }

            #header {
                background-color: #333;
                color: white;
            }

            p {
                font-size: 16px;
                line-height: 1.5;
            }
        "#;

        processor.parse(css).unwrap();
        assert_eq!(processor.get_rules().len(), 3);
    }

    #[test]
    fn test_compute_style() {
        let mut processor = CssProcessor::new();
        processor.parse(".container { display: flex; width: 100%; }").unwrap();

        let styles = processor.compute_style(".container");
        assert_eq!(styles.display, Some("flex".to_string()));
        assert_eq!(styles.width, Some("100%".to_string()));
    }

    #[test]
    fn test_specificity() {
        let processor = CssProcessor::new();

        // Element only
        assert_eq!(processor.calculate_specificity("div"), (0, 0, 1));

        // Class
        assert_eq!(processor.calculate_specificity(".container"), (0, 1, 0));

        // ID
        assert_eq!(processor.calculate_specificity("#header"), (1, 0, 0));

        // Combined
        assert_eq!(processor.calculate_specificity("div.container#main"), (1, 1, 1));
    }

    #[test]
    fn test_minify() {
        let processor = CssProcessor::new();
        let css = r#"
            .container {
                display: flex;
                width: 100%;
            }
        "#;

        let minified = processor.minify(css).unwrap();
        assert!(!minified.contains('\n') || minified.len() < css.len());
    }

    #[test]
    fn test_box_model_parsing() {
        let processor = CssProcessor::new();

        // Single value
        let box1 = processor.parse_box_model("10px");
        assert_eq!(box1.top, "10px");
        assert_eq!(box1.right, "10px");
        assert_eq!(box1.bottom, "10px");
        assert_eq!(box1.left, "10px");

        // Two values (vertical horizontal)
        let box2 = processor.parse_box_model("10px 20px");
        assert_eq!(box2.top, "10px");
        assert_eq!(box2.right, "20px");
        assert_eq!(box2.bottom, "10px");
        assert_eq!(box2.left, "20px");

        // Four values
        let box4 = processor.parse_box_model("10px 20px 30px 40px");
        assert_eq!(box4.top, "10px");
        assert_eq!(box4.right, "20px");
        assert_eq!(box4.bottom, "30px");
        assert_eq!(box4.left, "40px");
    }
}
