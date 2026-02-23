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
    /// Source order index for tie-breaking specificity
    #[serde(default)]
    pub source_order: usize,
}

/// CSS processor for handling CSS parsing, computation, and optimization
pub struct CssProcessor {
    /// Parsed rules from all stylesheets
    rules: Vec<ParsedRule>,
    /// Raw stylesheet sources for reference
    sources: Vec<String>,
    /// CSS custom properties (--name: value) collected from :root and other selectors
    custom_properties: HashMap<String, String>,
    /// Viewport width for media query evaluation
    viewport_width: f32,
    /// Source order counter
    source_order_counter: usize,
}

impl CssProcessor {
    /// Create a new CSS processor
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            sources: Vec::new(),
            custom_properties: HashMap::new(),
            viewport_width: 1024.0,
            source_order_counter: 0,
        }
    }

    /// Create a new CSS processor with a specific viewport width for media query evaluation
    pub fn new_with_viewport(viewport_width: f32) -> Self {
        Self {
            rules: Vec::new(),
            sources: Vec::new(),
            custom_properties: HashMap::new(),
            viewport_width,
            source_order_counter: 0,
        }
    }

    /// Parse a CSS string and add its rules to the processor
    pub fn parse(&mut self, css: &str) -> Result<()> {
        self.sources.push(css.to_string());

        let stylesheet = StyleSheet::parse(css, ParserOptions::default())
            .map_err(|e| anyhow::anyhow!("CSS parse error: {:?}", e))?;

        // Extract rules from the stylesheet
        self.process_rules(&stylesheet.rules.0);

        Ok(())
    }

    /// Process CSS rules recursively (handles @media, @supports, etc.)
    fn process_rules(&mut self, rules: &[CssRule]) {
        for rule in rules {
            match rule {
                CssRule::Style(style_rule) => {
                    // Get selector string
                    let selector_str = style_rule.selectors.to_css_string(PrinterOptions::default())
                        .unwrap_or_default();

                    // Calculate specificity
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

                    // Collect custom properties from :root rules (or any rule)
                    let is_root_selector = selector_str.split(',')
                        .any(|s| {
                            let s = s.trim();
                            s == ":root" || s == "html" || s == ":root, html" || s == "html, :root"
                        });

                    for (prop, value) in &declarations {
                        if prop.starts_with("--") {
                            let clean_value = value.trim_end_matches(" !important").to_string();
                            // :root custom properties get stored globally
                            // Non-root ones too (simplified — real browsers scope them)
                            self.custom_properties.insert(prop.clone(), clean_value);
                        }
                    }

                    let source_order = self.source_order_counter;
                    self.source_order_counter += 1;

                    self.rules.push(ParsedRule {
                        selector: selector_str,
                        specificity,
                        declarations,
                        source_order,
                    });
                }
                CssRule::Media(media_rule) => {
                    // Evaluate media query against viewport
                    let media_str = media_rule.query.to_css_string(PrinterOptions::default())
                        .unwrap_or_default();

                    if self.evaluate_media_query(&media_str) {
                        // Media query matches — process inner rules
                        self.process_rules(&media_rule.rules.0);
                    }
                }
                _ => {
                    // Skip @font-face, @keyframes, @import, etc.
                }
            }
        }
    }

    /// Evaluate a media query string against the current viewport
    fn evaluate_media_query(&self, query: &str) -> bool {
        // Handle multiple queries separated by comma (OR logic)
        for single_query in query.split(',') {
            if self.evaluate_single_media_query(single_query.trim()) {
                return true;
            }
        }
        false
    }

    /// Evaluate a single media query
    fn evaluate_single_media_query(&self, query: &str) -> bool {
        let query = query.trim();

        // Empty query matches everything
        if query.is_empty() {
            return true;
        }

        // Handle "not" prefix
        let (negated, query) = if query.starts_with("not ") {
            (true, &query[4..])
        } else {
            (false, query)
        };

        let result = self.evaluate_media_query_inner(query);

        if negated { !result } else { result }
    }

    /// Inner media query evaluation
    fn evaluate_media_query_inner(&self, query: &str) -> bool {
        let query = query.trim();

        // "all" always matches
        if query == "all" {
            return true;
        }

        // "screen" matches (we're a screen renderer)
        if query == "screen" {
            return true;
        }

        // "print" never matches
        if query == "print" {
            return false;
        }

        // Handle "screen and (...)" or just "(...)"
        let conditions_str = if query.starts_with("screen and ") {
            &query[11..]
        } else if query.starts_with("all and ") {
            &query[8..]
        } else {
            query
        };

        // Parse individual conditions: "(max-width: 700px)" etc.
        // Handle "and" joined conditions
        let mut all_match = true;
        for part in conditions_str.split(" and ") {
            let part = part.trim().trim_start_matches('(').trim_end_matches(')');
            if !self.evaluate_media_feature(part) {
                all_match = false;
                break;
            }
        }

        all_match
    }

    /// Evaluate a single media feature like "max-width: 700px" or "width <= 700px"
    fn evaluate_media_feature(&self, feature: &str) -> bool {
        // Handle modern range syntax: "width <= 700px", "width >= 768px", "700px <= width"
        if let Some(result) = self.evaluate_range_media_feature(feature) {
            return result;
        }

        // Handle legacy syntax: "max-width: 700px"
        let parts: Vec<&str> = feature.splitn(2, ':').collect();
        if parts.len() != 2 {
            // Features without values (e.g., "color") — assume true
            return true;
        }

        let name = parts[0].trim();
        let value_str = parts[1].trim();

        match name {
            "max-width" => {
                if let Some(px) = Self::parse_media_length(value_str) {
                    self.viewport_width <= px
                } else {
                    true
                }
            }
            "min-width" => {
                if let Some(px) = Self::parse_media_length(value_str) {
                    self.viewport_width >= px
                } else {
                    true
                }
            }
            "max-height" | "min-height" => {
                // We don't track viewport height precisely — assume true
                true
            }
            "prefers-color-scheme" => {
                // Default to light mode
                value_str == "light"
            }
            "prefers-reduced-motion" => {
                value_str == "no-preference"
            }
            _ => true, // Unknown features — assume match to be permissive
        }
    }

    /// Evaluate CSS Media Queries Level 4 range syntax
    /// e.g., "width <= 700px", "width >= 768px", "width < 1200px", "700px <= width"
    fn evaluate_range_media_feature(&self, feature: &str) -> Option<bool> {
        let feature = feature.trim();

        // Try patterns: "prop <= val", "prop >= val", "prop < val", "prop > val"
        for (op, op_str) in &[("<=", "<="), (">=", ">="), ("<", "<"), (">", ">")] {
            if let Some(pos) = feature.find(op_str) {
                let left = feature[..pos].trim();
                let right = feature[pos + op.len()..].trim();

                // Determine which side is the property name
                let (prop, value, reversed) = if left.chars().next().map(|c| c.is_alphabetic()).unwrap_or(false) {
                    (left, right, false)
                } else {
                    (right, left, true)
                };

                let viewport_val = match prop {
                    "width" => Some(self.viewport_width),
                    "height" => return Some(true), // Not tracked precisely
                    _ => None,
                };

                if let (Some(vp), Some(px)) = (viewport_val, Self::parse_media_length(value)) {
                    let result = if reversed {
                        // "700px <= width" means width >= 700px
                        match *op_str {
                            "<=" => vp >= px,
                            ">=" => vp <= px,
                            "<" => vp > px,
                            ">" => vp < px,
                            _ => true,
                        }
                    } else {
                        // "width <= 700px"
                        match *op_str {
                            "<=" => vp <= px,
                            ">=" => vp >= px,
                            "<" => vp < px,
                            ">" => vp > px,
                            _ => true,
                        }
                    };
                    return Some(result);
                }
            }
        }

        None // Not a range expression
    }

    /// Parse a media query length value like "700px" to f32
    fn parse_media_length(value: &str) -> Option<f32> {
        let v = value.trim();
        if v.ends_with("px") {
            v.trim_end_matches("px").parse::<f32>().ok()
        } else if v.ends_with("em") || v.ends_with("rem") {
            // 1em/rem = 16px for media queries (always relative to initial value)
            v.trim_end_matches("em").trim_end_matches("r").parse::<f32>().ok()
                .map(|n| n * 16.0)
        } else {
            v.parse::<f32>().ok()
        }
    }

    /// Resolve CSS `var()` references in a property value.
    /// Handles `var(--name)` and `var(--name, fallback)` with up to 10 levels of nesting.
    pub fn resolve_var(&self, value: &str) -> String {
        let mut result = value.to_string();

        // Iterate to resolve nested var() references (max 10 levels)
        for _ in 0..10 {
            if !result.contains("var(") {
                break;
            }
            result = self.resolve_var_once(&result);
        }

        result
    }

    /// Single pass of var() resolution
    fn resolve_var_once(&self, value: &str) -> String {
        let mut result = String::with_capacity(value.len());
        let mut chars = value.char_indices().peekable();

        while let Some((i, c)) = chars.next() {
            // Look for "var("
            if c == 'v' && value[i..].starts_with("var(") {
                // Find the matching closing paren, accounting for nesting
                let start = i + 4; // after "var("
                let mut depth = 1;
                let mut end = start;
                for (j, ch) in value[start..].char_indices() {
                    match ch {
                        '(' => depth += 1,
                        ')' => {
                            depth -= 1;
                            if depth == 0 {
                                end = start + j;
                                break;
                            }
                        }
                        _ => {}
                    }
                }

                if depth != 0 {
                    // Malformed var() — keep as-is
                    result.push(c);
                    continue;
                }

                let inner = &value[start..end];

                // Split on first comma for fallback: "var(--name, fallback)"
                let (var_name, fallback) = if let Some(comma_pos) = inner.find(',') {
                    let name = inner[..comma_pos].trim();
                    let fb = inner[comma_pos + 1..].trim();
                    (name, Some(fb))
                } else {
                    (inner.trim(), None)
                };

                // Look up the custom property
                let resolved = if let Some(val) = self.custom_properties.get(var_name) {
                    val.clone()
                } else if let Some(fb) = fallback {
                    fb.to_string()
                } else {
                    // Unresolved — keep original for debugging
                    format!("var({})", inner)
                };

                result.push_str(&resolved);

                // Skip past the closing paren
                // Advance the char iterator past the end of var(...)
                let skip_to = end + 1; // +1 for the ')'
                while let Some(&(j, _)) = chars.peek() {
                    if j >= skip_to {
                        break;
                    }
                    chars.next();
                }
            } else {
                result.push(c);
            }
        }

        result
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

    /// Compute styles for an element given its selector chain (legacy string-based matching)
    /// selector_chain is a list of selectors from root to the element (e.g., ["html", "body", "div.container", "p"])
    pub fn compute_style(&self, selector: &str) -> ComputedStyles {
        let mut styles = ComputedStyles::default();

        // Collect all matching rules sorted by specificity
        let mut matching_rules: Vec<&ParsedRule> = self.rules.iter()
            .filter(|rule| self.selector_applies(&rule.selector, selector))
            .collect();

        // Sort by specificity then source order (lower first, so higher specificity overrides)
        matching_rules.sort_by(|a, b| {
            a.specificity.cmp(&b.specificity)
                .then(a.source_order.cmp(&b.source_order))
        });

        // Apply declarations in order
        for rule in matching_rules {
            self.apply_declarations(&mut styles, &rule.declarations);
        }

        styles
    }

    /// Compute styles for an element using scraper's built-in CSS selector matching.
    /// This properly handles descendant selectors, child selectors, compound selectors, etc.
    pub fn compute_style_for_element(&self, element: &scraper::ElementRef) -> ComputedStyles {
        let mut styles = ComputedStyles::default();

        // Collect all rules whose selectors match this element
        let mut matching_rules: Vec<(&ParsedRule, bool)> = Vec::new(); // (rule, is_important)

        for rule in &self.rules {
            // Split selector by comma for multiple selectors (e.g., "h1, h2, h3")
            for raw_selector in rule.selector.split(',').map(|s| s.trim()) {
                if raw_selector.is_empty() {
                    continue;
                }

                // Try to parse the selector with scraper
                if let Ok(parsed_selector) = scraper::Selector::parse(raw_selector) {
                    if parsed_selector.matches(element) {
                        matching_rules.push((rule, false));
                        break; // One match is enough for this rule
                    }
                }
            }
        }

        // Sort by specificity then source order
        matching_rules.sort_by(|a, b| {
            a.0.specificity.cmp(&b.0.specificity)
                .then(a.0.source_order.cmp(&b.0.source_order))
        });

        // Apply declarations in order (non-important first)
        for (rule, _) in &matching_rules {
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

    /// Get custom properties map
    pub fn get_custom_properties(&self) -> &HashMap<String, String> {
        &self.custom_properties
    }

    /// Clear all parsed rules
    pub fn clear(&mut self) {
        self.rules.clear();
        self.sources.clear();
        self.custom_properties.clear();
        self.source_order_counter = 0;
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

    /// Check if a rule selector applies to a target element (legacy string-based matching)
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

    /// Apply declarations to computed styles, resolving var() references
    fn apply_declarations(&self, styles: &mut ComputedStyles, declarations: &HashMap<String, String>) {
        for (prop, value) in declarations {
            // Skip custom properties (--*) — they're already collected
            if prop.starts_with("--") {
                continue;
            }

            // Remove !important suffix for storage
            let raw_value = value.trim_end_matches(" !important").to_string();
            // Resolve var() references
            let clean_value = self.resolve_var(&raw_value);

            match prop.as_str() {
                "display" => styles.display = Some(clean_value),
                "position" => styles.position = Some(clean_value),
                "width" => styles.width = Some(clean_value),
                "height" => styles.height = Some(clean_value),
                "background-color" => styles.background_color = Some(clean_value),
                "background" => {
                    // The background shorthand can include color, image, position, etc.
                    // Extract just the color if it's a simple color value.
                    let v = clean_value.trim();
                    if v.starts_with('#') || v.starts_with("rgb") || v.starts_with("hsl")
                        || v == "transparent" || v == "inherit"
                        || (!v.contains(' ') && !v.starts_with("url"))
                    {
                        styles.background_color = Some(clean_value);
                    } else {
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

    #[test]
    fn test_media_query_evaluation() {
        let processor = CssProcessor::new_with_viewport(1024.0);

        // max-width greater than viewport — should match
        assert!(processor.evaluate_media_query("(max-width: 1200px)"));
        // max-width less than viewport — should not match
        assert!(!processor.evaluate_media_query("(max-width: 700px)"));
        // min-width less than viewport — should match
        assert!(processor.evaluate_media_query("(min-width: 768px)"));
        // min-width greater than viewport — should not match
        assert!(!processor.evaluate_media_query("(min-width: 1200px)"));
        // screen — should match
        assert!(processor.evaluate_media_query("screen"));
        // print — should not match
        assert!(!processor.evaluate_media_query("print"));
        // screen and condition
        assert!(processor.evaluate_media_query("screen and (min-width: 768px)"));
    }

    #[test]
    fn test_media_query_in_css() {
        let mut processor = CssProcessor::new_with_viewport(1024.0);
        let css = r#"
            div { width: 600px; }
            @media (max-width: 700px) {
                div { width: auto; }
            }
        "#;

        processor.parse(css).unwrap();
        // The media query (max-width: 700px) should NOT match at 1024px viewport
        // So only the first rule should be present
        let styles = processor.compute_style("div");
        assert_eq!(styles.width, Some("600px".to_string()));
    }

    #[test]
    fn test_media_query_matches() {
        let mut processor = CssProcessor::new_with_viewport(500.0);
        let css = r#"
            div { width: 600px; }
            @media (max-width: 700px) {
                div { width: auto; }
            }
        "#;

        processor.parse(css).unwrap();
        // At 500px viewport, max-width: 700px matches, so div should get width: auto
        let styles = processor.compute_style("div");
        assert_eq!(styles.width, Some("auto".to_string()));
    }

    #[test]
    fn test_css_custom_properties() {
        let mut processor = CssProcessor::new();
        let css = r#"
            :root {
                --primary-color: #3366ff;
                --font-size: 16px;
            }
            .button {
                color: var(--primary-color);
                font-size: var(--font-size);
            }
        "#;

        processor.parse(css).unwrap();
        let styles = processor.compute_style(".button");
        // lightningcss normalizes #3366ff → #36f
        assert_eq!(styles.color, Some("#36f".to_string()));
        assert_eq!(styles.font_size, Some("16px".to_string()));
    }

    #[test]
    fn test_css_var_with_fallback() {
        let mut processor = CssProcessor::new();
        let css = r#"
            .box {
                color: var(--undefined-var, red);
                background-color: var(--also-undefined, #ffffff);
            }
        "#;

        processor.parse(css).unwrap();
        let styles = processor.compute_style(".box");
        assert_eq!(styles.color, Some("red".to_string()));
        // lightningcss normalizes #ffffff → #fff
        assert_eq!(styles.background_color, Some("#fff".to_string()));
    }

    #[test]
    fn test_element_based_matching() {
        let html = r#"<html><body><div class="container"><p class="text">Hello</p></div></body></html>"#;
        let document = scraper::Html::parse_document(html);

        let mut processor = CssProcessor::new();
        // lightningcss normalizes named colors: blue → #00f
        processor.parse(".container .text { color: blue; }").unwrap();
        processor.parse("p { font-size: 14px; }").unwrap();

        let p_selector = scraper::Selector::parse("p.text").unwrap();
        if let Some(p_element) = document.select(&p_selector).next() {
            let styles = processor.compute_style_for_element(&p_element);
            assert_eq!(styles.color, Some("#00f".to_string()));
            assert_eq!(styles.font_size, Some("14px".to_string()));
        }
    }
}
