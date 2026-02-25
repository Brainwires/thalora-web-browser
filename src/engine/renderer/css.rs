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

    // --- Promoted from `other` HashMap for performance (direct field access, no hashing) ---

    /// Flex wrap: nowrap, wrap, wrap-reverse
    pub flex_wrap: Option<String>,
    /// Align self
    pub align_self: Option<String>,
    /// Flex grow factor
    pub flex_grow: Option<String>,
    /// Flex shrink factor
    pub flex_shrink: Option<String>,
    /// Flex basis
    pub flex_basis: Option<String>,
    /// Min width
    pub min_width: Option<String>,
    /// Min height
    pub min_height: Option<String>,
    /// Max width
    pub max_width: Option<String>,
    /// Max height
    pub max_height: Option<String>,
    /// Font style (normal, italic, oblique)
    pub font_style: Option<String>,
    /// Line height
    pub line_height: Option<String>,
    /// Text alignment
    pub text_align: Option<String>,
    /// Text decoration
    pub text_decoration: Option<String>,
    /// Text transform
    pub text_transform: Option<String>,
    /// White space handling
    pub white_space: Option<String>,
    /// Letter spacing
    pub letter_spacing: Option<String>,
    /// Word spacing
    pub word_spacing: Option<String>,
    /// Border radius
    pub border_radius: Option<String>,
    /// List style type
    pub list_style_type: Option<String>,
    /// Cursor style
    pub cursor: Option<String>,
    /// Grid template columns (e.g., "1fr 3fr", "200px 1fr 200px", "12.25rem minmax(0,1fr)")
    pub grid_template_columns: Option<String>,
    /// Grid template rows (e.g., "min-content 1fr min-content")
    pub grid_template_rows: Option<String>,
    /// Grid template areas (e.g., "'header header' 'sidebar content' 'footer footer'")
    pub grid_template_areas: Option<String>,
    /// Grid area name for child placement (e.g., "pageContent", "sidebar")
    pub grid_area: Option<String>,

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

/// A pre-compiled CSS rule: selector already parsed into scraper::Selector for fast matching.
/// Rules whose selectors fail to parse are stored with compiled_selector = None and fall back
/// to the slow path (pseudo-class fallback matching).
pub struct CompiledRule {
    /// Index into CssProcessor::rules
    pub rule_index: usize,
    /// Pre-compiled selectors (one per comma-separated selector alternative).
    /// None entries mean scraper couldn't parse that selector alternative.
    pub compiled_selectors: Vec<Option<scraper::Selector>>,
    /// Whether any selector alternative contains :hover (for fast skip in non-hover path)
    pub has_hover: bool,
    /// For hover rules: pre-compiled base selectors with :hover stripped
    pub hover_base_selectors: Vec<Option<scraper::Selector>>,
    /// Key tag names extracted from each selector alternative's rightmost simple selector.
    /// Used for fast pre-filtering: if the element's tag doesn't match any key_tag, skip .matches().
    /// None means the selector has no tag constraint (class-only, universal, etc.) — must always check.
    pub key_tags: Vec<Option<String>>,
    /// Key class name extracted from the rightmost simple selector.
    /// Used with key_tags for indexed rule lookup.
    pub key_classes: Vec<Option<String>>,
    /// Key ID extracted from the rightmost simple selector.
    /// Used with key_tags/key_classes for indexed rule lookup.
    pub key_ids: Vec<Option<String>>,
}

/// Indexed rule lookup for fast CSS matching.
/// Instead of iterating all rules per element, look up candidate rules by the element's
/// tag name, class names, and ID, then only run .matches() on those candidates.
pub struct RuleIndex {
    /// compiled_rule index → for rules whose key selector targets a specific tag
    pub by_tag: HashMap<String, Vec<usize>>,
    /// compiled_rule index → for rules whose key selector targets a specific class
    pub by_class: HashMap<String, Vec<usize>>,
    /// compiled_rule index → for rules whose key selector targets a specific ID
    pub by_id: HashMap<String, Vec<usize>>,
    /// compiled_rule indices for rules that could match any element (universal, complex selectors)
    pub universal: Vec<usize>,
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
    /// Classes on the <html> element (for scoping custom property selectors)
    html_classes: Vec<String>,
    /// Source order counter
    source_order_counter: usize,
    /// Pre-compiled selectors for fast matching (populated by compile_selectors())
    compiled_rules: Vec<CompiledRule>,
    /// Indexed rule lookup for O(1) candidate selection per element
    rule_index: Option<RuleIndex>,
}

impl CssProcessor {
    /// Create a new CSS processor
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            sources: Vec::new(),
            custom_properties: HashMap::new(),
            viewport_width: 1024.0,
            html_classes: Vec::new(),
            source_order_counter: 0,
            compiled_rules: Vec::new(),
            rule_index: None,
        }
    }

    /// Create a new CSS processor with a specific viewport width for media query evaluation
    pub fn new_with_viewport(viewport_width: f32) -> Self {
        Self {
            rules: Vec::new(),
            sources: Vec::new(),
            custom_properties: HashMap::new(),
            viewport_width,
            html_classes: Vec::new(),
            source_order_counter: 0,
            compiled_rules: Vec::new(),
            rule_index: None,
        }
    }

    /// Set the classes present on the <html> element.
    /// Used to scope CSS custom property definitions: selectors like
    /// `html.skin-theme-clientpref-night` should only contribute custom properties
    /// when the <html> element actually has that class.
    pub fn set_html_classes(&mut self, classes: Vec<String>) {
        self.html_classes = classes;
    }

    /// Pre-compile all CSS selectors for fast matching.
    /// Call this after all stylesheets have been parsed, before computing styles.
    /// This avoids re-parsing selector strings on every element match.
    pub fn compile_selectors(&mut self) {
        let start = std::time::Instant::now();
        self.compiled_rules.clear();
        self.compiled_rules.reserve(self.rules.len());

        for (idx, rule) in self.rules.iter().enumerate() {
            let selector_alternatives: Vec<&str> = rule.selector.split(',').map(|s| s.trim()).collect();
            let mut compiled_selectors = Vec::with_capacity(selector_alternatives.len());
            let mut has_hover = false;
            let mut hover_base_selectors = Vec::new();
            let mut key_tags = Vec::with_capacity(selector_alternatives.len());
            let mut key_classes = Vec::with_capacity(selector_alternatives.len());
            let mut key_ids = Vec::with_capacity(selector_alternatives.len());

            for raw_selector in &selector_alternatives {
                if raw_selector.is_empty() {
                    compiled_selectors.push(None);
                    hover_base_selectors.push(None);
                    key_tags.push(None);
                    key_classes.push(None);
                    key_ids.push(None);
                    continue;
                }

                // Extract key tag, class, and ID from rightmost simple selector for indexed lookup
                key_tags.push(Self::extract_key_tag(raw_selector));
                key_classes.push(Self::extract_key_class(raw_selector));
                key_ids.push(Self::extract_key_id(raw_selector));

                // Check for :hover
                let is_hover = Self::contains_hover_pseudo(raw_selector);
                if is_hover {
                    has_hover = true;
                }

                // Strip `:not(:focus)`, `:not(:hover)`, `:not(:active)`, `:not(:focus-visible)`
                // from selectors before compilation. In static rendering, elements are never
                // focused/hovered/active, so these negations are always true.
                let preprocessed = raw_selector
                    .replace(":not(:focus-visible)", "")
                    .replace(":not(:focus-within)", "")
                    .replace(":not(:focus)", "")
                    .replace(":not(:hover)", "")
                    .replace(":not(:active)", "");
                let sel_to_compile = if preprocessed.trim().is_empty() {
                    raw_selector.to_string()
                } else {
                    preprocessed
                };

                // Compile the full selector
                let compiled = scraper::Selector::parse(&sel_to_compile).ok();
                compiled_selectors.push(compiled);

                // For hover selectors, also compile the base (with :hover stripped)
                if is_hover {
                    let base = Self::strip_hover_pseudo(raw_selector);
                    if base.is_empty() {
                        hover_base_selectors.push(None); // bare :hover matches anything
                    } else {
                        hover_base_selectors.push(scraper::Selector::parse(&base).ok());
                    }
                } else {
                    hover_base_selectors.push(None);
                }
            }

            self.compiled_rules.push(CompiledRule {
                rule_index: idx,
                compiled_selectors,
                has_hover,
                hover_base_selectors,
                key_tags,
                key_classes,
                key_ids,
            });
        }

        // Build the rule index: group compiled rules by key tag, key class, key id, or universal
        let mut by_tag: HashMap<String, Vec<usize>> = HashMap::new();
        let mut by_class: HashMap<String, Vec<usize>> = HashMap::new();
        let mut by_id: HashMap<String, Vec<usize>> = HashMap::new();
        let mut universal: Vec<usize> = Vec::new();

        for (ci, compiled) in self.compiled_rules.iter().enumerate() {
            // A rule is indexed by the FIRST selector alternative that has a key tag, class, or id.
            // If any alternative has no key tag/class/id, the rule must go in universal.
            let mut has_specific = false;
            let mut has_universal_alt = false;

            for i in 0..compiled.key_tags.len() {
                let kt = compiled.key_tags.get(i).and_then(|x| x.as_ref());
                let kc = compiled.key_classes.get(i).and_then(|x| x.as_ref());
                let ki = compiled.key_ids.get(i).and_then(|x| x.as_ref());

                if kt.is_some() || kc.is_some() || ki.is_some() {
                    has_specific = true;
                    if let Some(tag) = kt {
                        by_tag.entry(tag.clone()).or_default().push(ci);
                    }
                    if let Some(cls) = kc {
                        by_class.entry(cls.clone()).or_default().push(ci);
                    }
                    if let Some(id) = ki {
                        by_id.entry(id.clone()).or_default().push(ci);
                    }
                } else {
                    has_universal_alt = true;
                }
            }

            // If any selector alternative is universal (no tag/class/id), the rule must also
            // be in the universal bucket so it gets checked for all elements.
            if has_universal_alt || !has_specific {
                universal.push(ci);
            }
        }

        eprintln!("[TIMING] compile_selectors: {}ms ({} rules, {} universal, {} tag-indexed, {} class-indexed, {} id-indexed)",
            start.elapsed().as_millis(), self.rules.len(), universal.len(),
            by_tag.len(), by_class.len(), by_id.len());

        self.rule_index = Some(RuleIndex { by_tag, by_class, by_id, universal });
    }

    /// Extract the tag name from the rightmost simple selector of a CSS selector string.
    /// Returns None for class-only, ID-only, or universal selectors.
    /// Examples:
    ///   "nav > ul > li > a.link" → Some("a")
    ///   ".container"             → None
    ///   "div.flex"               → Some("div")
    ///   "#main"                  → None
    ///   "h1"                     → Some("h1")
    ///   "*"                      → None
    fn extract_key_tag(selector: &str) -> Option<String> {
        // Get the last simple selector (after the last combinator: space, >, +, ~)
        let last = selector
            .rsplit(|c: char| c == ' ' || c == '>' || c == '+' || c == '~')
            .next()
            .unwrap_or(selector)
            .trim();
        if last.is_empty() {
            return None;
        }
        // Extract tag name: everything before first '.', '#', ':', '['
        let tag_end = last
            .find(|c: char| c == '.' || c == '#' || c == ':' || c == '[')
            .unwrap_or(last.len());
        let tag = &last[..tag_end];
        if tag.is_empty() || tag == "*" {
            None
        } else {
            Some(tag.to_lowercase())
        }
    }

    /// Extract the ID from the rightmost simple selector of a CSS selector string.
    /// Returns None for tag-only, class-only, or universal selectors.
    /// Examples:
    ///   "#main"        → Some("main")
    ///   "div#content"  → Some("content")
    ///   ".container"   → None
    ///   "h1"           → None
    fn extract_key_id(selector: &str) -> Option<String> {
        let last = selector
            .rsplit(|c: char| c == ' ' || c == '>' || c == '+' || c == '~')
            .next()
            .unwrap_or(selector)
            .trim();
        if last.is_empty() {
            return None;
        }
        // Find first '#' that starts an ID
        if let Some(hash_pos) = last.find('#') {
            let after_hash = &last[hash_pos + 1..];
            // ID ends at next '.', '#', ':', '['
            let id_end = after_hash
                .find(|c: char| c == '.' || c == '#' || c == ':' || c == '[')
                .unwrap_or(after_hash.len());
            let id = &after_hash[..id_end];
            if !id.is_empty() {
                return Some(id.to_lowercase());
            }
        }
        None
    }

    /// Extract the first class name from the rightmost simple selector of a CSS selector string.
    /// Returns None for tag-only, ID-only, or universal selectors.
    /// Examples:
    ///   ".container"        → Some("container")
    ///   "div.flex"          → Some("flex")
    ///   "a.nav-link.active" → Some("nav-link")
    ///   "#main"             → None
    ///   "h1"                → None
    fn extract_key_class(selector: &str) -> Option<String> {
        let last = selector
            .rsplit(|c: char| c == ' ' || c == '>' || c == '+' || c == '~')
            .next()
            .unwrap_or(selector)
            .trim();
        if last.is_empty() {
            return None;
        }
        // Find first '.' that starts a class name
        if let Some(dot_pos) = last.find('.') {
            let after_dot = &last[dot_pos + 1..];
            // Class name ends at next '.', '#', ':', '['
            let class_end = after_dot
                .find(|c: char| c == '.' || c == '#' || c == ':' || c == '[')
                .unwrap_or(after_dot.len());
            let class = &after_dot[..class_end];
            if !class.is_empty() {
                return Some(class.to_lowercase());
            }
        }
        None
    }

    /// Parse inline style declarations directly into ComputedStyles without
    /// creating a full CssProcessor. Much faster for per-element inline styles.
    pub fn parse_inline_style_direct(style: &str) -> ComputedStyles {
        let mut styles = ComputedStyles::default();
        for declaration in style.split(';') {
            let declaration = declaration.trim();
            if declaration.is_empty() {
                continue;
            }
            if let Some((prop, value)) = declaration.split_once(':') {
                let prop = prop.trim().to_lowercase();
                let value = value.trim().to_string();
                if value.is_empty() {
                    continue;
                }
                // Map CSS properties to ComputedStyles fields
                match prop.as_str() {
                    "display" => styles.display = Some(value),
                    "position" => styles.position = Some(value),
                    "width" => styles.width = Some(value),
                    "height" => styles.height = Some(value),
                    "background-color" => styles.background_color = Some(value),
                    "background" => {
                        if let Some(color) = Self::extract_color_from_background(&value) {
                            styles.background_color = Some(color);
                        }
                    }
                    "color" => styles.color = Some(value),
                    "font-size" => styles.font_size = Some(value),
                    "font-family" => styles.font_family = Some(value),
                    "font-weight" => styles.font_weight = Some(value),
                    "flex-direction" => styles.flex_direction = Some(value),
                    "justify-content" => styles.justify_content = Some(value),
                    "align-items" => styles.align_items = Some(value),
                    "gap" => styles.gap = Some(value),
                    "overflow" => styles.overflow = Some(value),
                    "visibility" => styles.visibility = Some(value),
                    "opacity" => { if let Ok(v) = value.parse::<f32>() { styles.opacity = Some(v); } },
                    "z-index" => { if let Ok(v) = value.parse::<i32>() { styles.z_index = Some(v); } },
                    "margin" => {
                        let parts: Vec<&str> = value.split_whitespace().collect();
                        styles.margin = Some(Self::parse_shorthand_box(&parts));
                    }
                    "margin-top" | "margin-right" | "margin-bottom" | "margin-left" => {
                        let m = styles.margin.get_or_insert_with(|| BoxModel::default());
                        match prop.as_str() {
                            "margin-top" => m.top = value,
                            "margin-right" => m.right = value,
                            "margin-bottom" => m.bottom = value,
                            "margin-left" => m.left = value,
                            _ => {}
                        }
                    }
                    "padding" => {
                        let parts: Vec<&str> = value.split_whitespace().collect();
                        styles.padding = Some(Self::parse_shorthand_box(&parts));
                    }
                    "padding-top" | "padding-right" | "padding-bottom" | "padding-left" => {
                        let p = styles.padding.get_or_insert_with(|| BoxModel::default());
                        match prop.as_str() {
                            "padding-top" => p.top = value,
                            "padding-right" => p.right = value,
                            "padding-bottom" => p.bottom = value,
                            "padding-left" => p.left = value,
                            _ => {}
                        }
                    }
                    // Promoted properties
                    "flex-wrap" => styles.flex_wrap = Some(value),
                    "align-self" => styles.align_self = Some(value),
                    "flex-grow" => styles.flex_grow = Some(value),
                    "flex-shrink" => styles.flex_shrink = Some(value),
                    "flex-basis" => styles.flex_basis = Some(value),
                    "flex" => {
                        // flex shorthand: <grow> [<shrink>] [<basis>]
                        // Common forms: "1" → grow:1 shrink:1 basis:0%
                        //               "0 1 auto" → grow:0 shrink:1 basis:auto
                        //               "none" → grow:0 shrink:0 basis:auto
                        let parts: Vec<&str> = value.split_whitespace().collect();
                        match parts.len() {
                            1 if parts[0] == "none" => {
                                styles.flex_grow = Some("0".to_string());
                                styles.flex_shrink = Some("0".to_string());
                                styles.flex_basis = Some("auto".to_string());
                            }
                            1 if parts[0] == "auto" => {
                                styles.flex_grow = Some("1".to_string());
                                styles.flex_shrink = Some("1".to_string());
                                styles.flex_basis = Some("auto".to_string());
                            }
                            1 => {
                                styles.flex_grow = Some(parts[0].to_string());
                                styles.flex_shrink = Some("1".to_string());
                                styles.flex_basis = Some("0%".to_string());
                            }
                            2 => {
                                styles.flex_grow = Some(parts[0].to_string());
                                styles.flex_shrink = Some(parts[1].to_string());
                                styles.flex_basis = Some("0%".to_string());
                            }
                            _ => {
                                styles.flex_grow = Some(parts[0].to_string());
                                styles.flex_shrink = Some(parts[1].to_string());
                                styles.flex_basis = Some(parts[2].to_string());
                            }
                        }
                    },
                    "min-width" => styles.min_width = Some(value),
                    "min-height" => styles.min_height = Some(value),
                    "max-width" => styles.max_width = Some(value),
                    "max-height" => styles.max_height = Some(value),
                    "font-style" => styles.font_style = Some(value),
                    "line-height" => styles.line_height = Some(value),
                    "text-align" => styles.text_align = Some(value),
                    "text-decoration" => styles.text_decoration = Some(value),
                    "text-transform" => styles.text_transform = Some(value),
                    "white-space" => styles.white_space = Some(value),
                    "letter-spacing" => styles.letter_spacing = Some(value),
                    "word-spacing" => styles.word_spacing = Some(value),
                    "border-radius" => styles.border_radius = Some(value),
                    "list-style-type" => styles.list_style_type = Some(value),
                    "list-style" => {
                        // list-style shorthand: <type> || <position> || <image>
                        // Common usage: "none" sets list-style-type to none
                        let v = value.trim();
                        if v == "none" || v.starts_with("none ") || v.contains(" none") {
                            styles.list_style_type = Some("none".to_string());
                        } else {
                            // Extract the type keyword from the shorthand
                            for part in v.split_whitespace() {
                                match part {
                                    "disc" | "circle" | "square" | "decimal"
                                    | "decimal-leading-zero" | "lower-roman" | "upper-roman"
                                    | "lower-alpha" | "upper-alpha" | "lower-latin"
                                    | "upper-latin" | "lower-greek" => {
                                        styles.list_style_type = Some(part.to_string());
                                        break;
                                    }
                                    _ => {}
                                }
                            }
                        }
                    },
                    "cursor" => styles.cursor = Some(value),
                    "grid-template-columns" => styles.grid_template_columns = Some(value),
                    "grid-template-rows" => styles.grid_template_rows = Some(value),
                    "grid-template-areas" => styles.grid_template_areas = Some(value),
                    "grid-area" => styles.grid_area = Some(value),
                    "grid-template" => {
                        // grid-template shorthand: extract columns (after last '/') and rows/areas (before)
                        if let Some(slash_pos) = value.rfind('/') {
                            let columns = value[slash_pos + 1..].trim().to_string();
                            if !columns.is_empty() {
                                styles.grid_template_columns = Some(columns);
                            }
                            let rows_part = value[..slash_pos].trim().to_string();
                            if !rows_part.is_empty() {
                                let mut areas = Vec::new();
                                let mut rows = Vec::new();
                                let mut in_quote = false;
                                let mut quote_char = '\0';
                                let mut current_area = String::new();
                                let mut current_token = String::new();
                                for ch in rows_part.chars() {
                                    if (ch == '\'' || ch == '"') && (!in_quote || ch == quote_char) {
                                        if in_quote {
                                            areas.push(current_area.clone());
                                            current_area.clear();
                                            in_quote = false;
                                            quote_char = '\0';
                                        } else {
                                            let token = current_token.trim().to_string();
                                            if !token.is_empty() { rows.push(token); }
                                            current_token.clear();
                                            in_quote = true;
                                            quote_char = ch;
                                        }
                                    } else if in_quote {
                                        current_area.push(ch);
                                    } else {
                                        current_token.push(ch);
                                    }
                                }
                                let token = current_token.trim().to_string();
                                if !token.is_empty() { rows.push(token); }
                                if !areas.is_empty() {
                                    let areas_str = areas.iter()
                                        .map(|a| format!("'{}'", a))
                                        .collect::<Vec<_>>()
                                        .join(" ");
                                    styles.grid_template_areas = Some(areas_str);
                                }
                                if !rows.is_empty() {
                                    styles.grid_template_rows = Some(rows.join(" "));
                                }
                            }
                        }
                    }
                    "column-gap" => styles.gap = Some(value),
                    "row-gap" => {
                        if styles.gap.is_none() { styles.gap = Some(value); }
                    }
                    _ => {
                        // Store everything else in the 'other' map
                        styles.other.insert(prop, value);
                    }
                }
            }
        }
        styles
    }

    /// Parse a CSS shorthand box value (margin/padding) into a BoxModel.
    fn parse_shorthand_box(parts: &[&str]) -> BoxModel {
        match parts.len() {
            1 => BoxModel {
                top: parts[0].to_string(), right: parts[0].to_string(),
                bottom: parts[0].to_string(), left: parts[0].to_string(),
            },
            2 => BoxModel {
                top: parts[0].to_string(), right: parts[1].to_string(),
                bottom: parts[0].to_string(), left: parts[1].to_string(),
            },
            3 => BoxModel {
                top: parts[0].to_string(), right: parts[1].to_string(),
                bottom: parts[2].to_string(), left: parts[1].to_string(),
            },
            4 => BoxModel {
                top: parts[0].to_string(), right: parts[1].to_string(),
                bottom: parts[2].to_string(), left: parts[3].to_string(),
            },
            _ => BoxModel::default(),
        }
    }

    /// Check if a single CSS selector alternative would match the document root element.
    ///
    /// This is used for scoping CSS custom property definitions: selectors like
    /// `html.skin-theme-clientpref-night` should only contribute their custom properties
    /// when the `<html>` element actually has that class.
    ///
    /// Matches:
    /// - `:root` / `html` / `*` (always match root)
    /// - `html.some-class` (matches if html_classes contains "some-class")
    /// - `.some-class` (matches if html_classes contains "some-class", since :root has all classes)
    /// - Complex selectors (descendant/child combinators) are skipped (return false)
    fn selector_matches_root(&self, selector: &str) -> bool {
        let s = selector.trim();
        if s.is_empty() {
            return false;
        }

        // Universal selectors always match
        if s == ":root" || s == "html" || s == "*"
            || s == "*, :before, :after"
            || s == "*, ::before, ::after"
            || s == ":before" || s == ":after"
            || s == "::before" || s == "::after"
        {
            return true;
        }

        // If selector has spaces or combinators (>, +, ~), it's a descendant selector —
        // custom properties on descendant selectors aren't typically root-scoped.
        // But they might define vars used elsewhere. For now, be permissive and store them.
        if s.contains(' ') || s.contains('>') || s.contains('+') || s.contains('~') {
            // Descendant/complex selectors — store their custom properties
            // (they might define vars that other elements need)
            return true;
        }

        // Handle :root with pseudo-classes (e.g., ":root:lang(en)")
        if s.starts_with(":root") {
            return true;
        }

        // Handle "html.classname" — check if the html element has the required classes
        if s.starts_with("html.") || s.starts_with("html[") {
            // Extract class requirements from the selector
            return self.html_element_matches_selector(s);
        }

        // Handle ".classname" (without tag) — could match any element including :root
        // Be permissive: store the custom properties
        if s.starts_with('.') || s.starts_with('[') || s.starts_with('#') {
            return true;
        }

        // Other tag selectors (e.g., "body", "div") — these don't match root
        // but their custom properties might be needed. Be permissive.
        true
    }

    /// Check if the html element matches a simple selector like "html.class1.class2".
    /// Only handles class and tag requirements on the html element itself.
    fn html_element_matches_selector(&self, selector: &str) -> bool {
        // Parse "html.class1.class2" or "html[attr]" patterns
        let without_tag = if selector.starts_with("html") {
            &selector[4..]
        } else {
            selector
        };

        // Extract all class requirements (everything after each '.')
        for part in without_tag.split('.') {
            let class_name = part.split(|c: char| c == ':' || c == '[' || c == '#')
                .next()
                .unwrap_or("")
                .trim();
            if !class_name.is_empty() {
                if !self.html_classes.iter().any(|c| c == class_name) {
                    return false;
                }
            }
        }

        true
    }

    /// Pre-process CSS to remove constructs that lightningcss can't handle.
    /// This strips IE CSS hacks like `*zoom: 1` which cause parse failures.
    fn preprocess_css(css: &str) -> String {
        // Remove IE property hacks: `*property: value` inside declaration blocks.
        // These are `{...; *zoom: 1; ...}` patterns — the `*` prefix is an IE 6/7 hack.
        let mut result = String::with_capacity(css.len());
        let bytes = css.as_bytes();
        let len = bytes.len();
        let mut i = 0;
        while i < len {
            // Detect `*identifier:` inside a block (after `{` or `;`)
            if bytes[i] == b'*' && i + 1 < len && bytes[i + 1].is_ascii_alphabetic() {
                // Check if preceded by `{` or `;` (skipping whitespace)
                let mut j = i.wrapping_sub(1);
                while j < len && (bytes[j] == b' ' || bytes[j] == b'\n' || bytes[j] == b'\r' || bytes[j] == b'\t') {
                    j = j.wrapping_sub(1);
                }
                if j < len && (bytes[j] == b'{' || bytes[j] == b';') {
                    // This is an IE hack — skip until `;` or `}`
                    i += 1;
                    while i < len && bytes[i] != b';' && bytes[i] != b'}' {
                        i += 1;
                    }
                    if i < len && bytes[i] == b';' {
                        i += 1; // skip the semicolon too
                    }
                    // If we hit `}`, don't consume it — it closes the block
                    continue;
                }
            }
            result.push(bytes[i] as char);
            i += 1;
        }
        result
    }

    /// Parse a CSS string and add its rules to the processor
    pub fn parse(&mut self, css: &str) -> Result<()> {
        // Pre-process to remove IE hacks that lightningcss can't handle
        let cleaned = Self::preprocess_css(css);
        self.sources.push(cleaned.clone());

        let stylesheet = StyleSheet::parse(&cleaned, ParserOptions::default())
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

                    // Custom properties (CSS variables) need scoping:
                    // Only store them if at least one selector alternative would match
                    // the document root. Selectors like `html.skin-theme-clientpref-night`
                    // should only contribute their custom properties when the <html> element
                    // actually has that class.
                    let should_store_vars = selector_str.split(',')
                        .any(|alt| self.selector_matches_root(alt.trim()));

                    for (prop, value) in &declarations {
                        if prop.starts_with("--") {
                            let clean_value = value.trim_end_matches(" !important").to_string();

                            if should_store_vars {
                                self.custom_properties.insert(prop.clone(), clean_value);
                            }
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

                    let matches = self.evaluate_media_query(&media_str);
                    if matches {
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

        // Handle "only screen and (...)", "screen and (...)", or just "(...)"
        // The "only" keyword is for backwards compatibility and should be ignored.
        let conditions_str = if query.starts_with("only screen and ") {
            &query[16..]
        } else if query.starts_with("only screen") {
            return true; // "only screen" with no conditions
        } else if query.starts_with("screen and ") {
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
        // Handle calc() expressions: calc(640px - 1px) → 639
        if v.starts_with("calc(") && v.ends_with(")") {
            return Self::evaluate_calc_expr(&v[5..v.len()-1]);
        }
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

    /// Evaluate a simple calc() expression in media queries.
    /// Handles: calc(Npx +/- Mpx), calc(Nem +/- Mem), etc.
    fn evaluate_calc_expr(expr: &str) -> Option<f32> {
        let expr = expr.trim();

        // Try to find + or - operator (not inside a sub-expression)
        // Handle: "640px - 1px", "100vw - 2rem"
        for op in [" - ", " + "] {
            if let Some(pos) = expr.find(op) {
                let left = expr[..pos].trim();
                let right = expr[pos + op.len()..].trim();
                let left_val = Self::parse_media_length(left)?;
                let right_val = Self::parse_media_length(right)?;
                return Some(if op == " + " {
                    left_val + right_val
                } else {
                    left_val - right_val
                });
            }
        }

        // Fallback: try parsing the whole expression as a single value
        Self::parse_media_length(expr)
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
                    // Detect self-referential values: if the stored value references
                    // the same variable (e.g., --font-size-medium: var(--font-size-medium, 1rem)),
                    // use the fallback instead to break the cycle.
                    // Per CSS spec, a custom property that references itself is
                    // "invalid at computed-value time" and the fallback must be used.
                    if val.contains(&format!("var({}", var_name)) {
                        if let Some(fb) = fallback {
                            fb.to_string()
                        } else {
                            val.clone()
                        }
                    } else {
                        val.clone()
                    }
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

    /// Compute styles for an element using indexed rule lookup.
    /// Instead of checking all rules, only checks rules that could match this element
    /// based on its tag name and class names. This reduces matching from O(all_rules) to
    /// O(relevant_rules) per element, which is typically 5-20x fewer rules.
    pub fn compute_style_for_element(&self, element: &scraper::ElementRef) -> ComputedStyles {
        let mut styles = ComputedStyles::default();

        // Fast path: use indexed rule lookup
        if let Some(ref index) = self.rule_index {
            let mut matching_rules: Vec<(&ParsedRule, bool)> = Vec::new();
            let el = element.value();
            let tag_name = el.name().to_lowercase();

            // Collect candidate rule indices from the index
            let mut candidates = Vec::new();

            // Add rules indexed by this element's tag
            if let Some(tag_rules) = index.by_tag.get(&tag_name) {
                candidates.extend_from_slice(tag_rules);
            }

            // Add rules indexed by this element's ID
            if let Some(id_attr) = el.attr("id") {
                let id_lower = id_attr.to_lowercase();
                if let Some(id_rules) = index.by_id.get(&id_lower) {
                    candidates.extend_from_slice(id_rules);
                }
            }

            // Add rules indexed by this element's classes
            if let Some(class_attr) = el.attr("class") {
                for cls in class_attr.split_whitespace() {
                    let cls_lower = cls.to_lowercase();
                    if let Some(class_rules) = index.by_class.get(&cls_lower) {
                        candidates.extend_from_slice(class_rules);
                    }
                }
            }

            // Add universal rules (always checked)
            candidates.extend_from_slice(&index.universal);

            // Deduplicate candidate indices
            candidates.sort_unstable();
            candidates.dedup();

            // Only check candidate rules
            for &ci in &candidates {
                let compiled = &self.compiled_rules[ci];
                let rule = &self.rules[compiled.rule_index];

                let mut matched = false;
                for (i, compiled_sel) in compiled.compiled_selectors.iter().enumerate() {
                    if let Some(sel) = compiled_sel {
                        if sel.matches(element) {
                            matched = true;
                            break;
                        }
                    } else {
                        // Fallback: selector failed to compile, try pseudo-class fallback
                        let raw_selectors: Vec<&str> = rule.selector.split(',').map(|s| s.trim()).collect();
                        if let Some(raw_sel) = raw_selectors.get(i) {
                            if Self::matches_pseudo_class_fallback(raw_sel, &tag_name, el) {
                                matched = true;
                                break;
                            }
                        }
                    }
                }

                if matched {
                    matching_rules.push((rule, false));
                }
            }

            // Sort by specificity then source order
            matching_rules.sort_by(|a, b| {
                a.0.specificity.cmp(&b.0.specificity)
                    .then(a.0.source_order.cmp(&b.0.source_order))
            });

            for (rule, _) in &matching_rules {
                self.apply_declarations(&mut styles, &rule.declarations);
            }

            return styles;
        }

        // Slow path: no pre-compiled selectors, parse at runtime (legacy)
        let mut matching_rules: Vec<(&ParsedRule, bool)> = Vec::new();
        let el = element.value();
        let tag_name = el.name().to_lowercase();

        for rule in &self.rules {
            for raw_selector in rule.selector.split(',').map(|s| s.trim()) {
                if raw_selector.is_empty() {
                    continue;
                }
                // Strip always-true negations for static rendering
                let preprocessed_slow = raw_selector
                    .replace(":not(:focus-visible)", "")
                    .replace(":not(:focus-within)", "")
                    .replace(":not(:focus)", "")
                    .replace(":not(:hover)", "")
                    .replace(":not(:active)", "");
                let sel_slow = if preprocessed_slow.trim().is_empty() { raw_selector.to_string() } else { preprocessed_slow };
                let sel_slow = sel_slow.as_str();
                if let Ok(parsed_selector) = scraper::Selector::parse(sel_slow) {
                    if parsed_selector.matches(element) {
                        matching_rules.push((rule, false));
                        break;
                    }
                } else {
                    if Self::matches_pseudo_class_fallback(raw_selector, &tag_name, el) {
                        matching_rules.push((rule, false));
                        break;
                    }
                }
            }
        }

        matching_rules.sort_by(|a, b| {
            a.0.specificity.cmp(&b.0.specificity)
                .then(a.0.source_order.cmp(&b.0.source_order))
        });

        for (rule, _) in &matching_rules {
            self.apply_declarations(&mut styles, &rule.declarations);
        }

        styles
    }

    /// Compute hover-specific styles for an element using scraper's selector matching.
    ///
    /// This iterates all parsed rules looking for selectors that contain `:hover`.
    /// For each matching rule, we strip `:hover` from the selector and check if the
    /// base selector matches the element. Returns the accumulated hover-only declarations.
    pub fn compute_hover_style_for_element(&self, element: &scraper::ElementRef) -> ComputedStyles {
        let mut styles = ComputedStyles::default();

        // Fast path: use pre-compiled selectors
        if !self.compiled_rules.is_empty() {
            let mut matching_rules: Vec<&ParsedRule> = Vec::new();
            let el = element.value();
            let tag_name = el.name().to_lowercase();

            for compiled in &self.compiled_rules {
                // Skip rules that don't have :hover at all
                if !compiled.has_hover {
                    continue;
                }

                let rule = &self.rules[compiled.rule_index];
                let raw_selectors: Vec<&str> = rule.selector.split(',').map(|s| s.trim()).collect();
                let mut matched = false;

                for (i, raw_sel) in raw_selectors.iter().enumerate() {
                    if !Self::contains_hover_pseudo(raw_sel) {
                        continue;
                    }

                    // Use pre-compiled hover base selector
                    if let Some(Some(base_sel)) = compiled.hover_base_selectors.get(i) {
                        if base_sel.matches(element) {
                            matched = true;
                            break;
                        }
                    } else if let Some(None) = compiled.hover_base_selectors.get(i) {
                        // Base selector was empty (bare ":hover") or failed to compile
                        let base = Self::strip_hover_pseudo(raw_sel);
                        if base.is_empty() {
                            matched = true;
                            break;
                        }
                        // Fallback for failed compile
                        if Self::simple_selector_matches(&base, &tag_name, el) {
                            matched = true;
                            break;
                        }
                    }
                }

                if matched {
                    matching_rules.push(rule);
                }
            }

            matching_rules.sort_by(|a, b| {
                a.specificity.cmp(&b.specificity)
                    .then(a.source_order.cmp(&b.source_order))
            });

            for rule in &matching_rules {
                self.apply_declarations(&mut styles, &rule.declarations);
            }

            return styles;
        }

        // Slow path: no pre-compiled selectors (legacy)
        let mut matching_rules: Vec<&ParsedRule> = Vec::new();

        for rule in &self.rules {
            for raw_selector in rule.selector.split(',').map(|s| s.trim()) {
                if raw_selector.is_empty() {
                    continue;
                }
                if !Self::contains_hover_pseudo(raw_selector) {
                    continue;
                }
                let base_selector = Self::strip_hover_pseudo(raw_selector);
                if base_selector.is_empty() {
                    matching_rules.push(rule);
                    break;
                }
                if let Ok(parsed_selector) = scraper::Selector::parse(&base_selector) {
                    if parsed_selector.matches(element) {
                        matching_rules.push(rule);
                        break;
                    }
                } else {
                    let el = element.value();
                    let tag_name = el.name().to_lowercase();
                    if Self::simple_selector_matches(&base_selector, &tag_name, el) {
                        matching_rules.push(rule);
                        break;
                    }
                }
            }
        }

        matching_rules.sort_by(|a, b| {
            a.specificity.cmp(&b.specificity)
                .then(a.source_order.cmp(&b.source_order))
        });

        for rule in &matching_rules {
            self.apply_declarations(&mut styles, &rule.declarations);
        }

        styles
    }

    /// Check if a selector string contains the :hover pseudo-class.
    fn contains_hover_pseudo(selector: &str) -> bool {
        // Look for ":hover" that's either at the end or followed by non-alphanumeric
        let mut search_from = 0;
        while let Some(pos) = selector[search_from..].find(":hover") {
            let abs_pos = search_from + pos;
            let after = abs_pos + 6;
            // Make sure it's ":hover" and not ":hover-something"
            if after >= selector.len() || !selector.as_bytes()[after].is_ascii_alphanumeric() {
                // Also make sure it's not "::hover" (pseudo-element)
                if abs_pos == 0 || selector.as_bytes()[abs_pos - 1] != b':' {
                    return true;
                }
            }
            search_from = after;
        }
        false
    }

    /// Strip `:hover` from a selector, returning the base selector.
    /// e.g., "a:hover" → "a", ".nav-link:hover" → ".nav-link",
    /// "nav a:hover" → "nav a", ":hover" → ""
    fn strip_hover_pseudo(selector: &str) -> String {
        let mut result = selector.to_string();
        // Remove ":hover" occurrences (not "::hover")
        loop {
            if let Some(pos) = result.find(":hover") {
                // Ensure it's not "::hover"
                if pos > 0 && result.as_bytes()[pos - 1] == b':' {
                    break;
                }
                let after = pos + 6;
                // Make sure it's ":hover" and not ":hover-something"
                if after < result.len() && result.as_bytes()[after].is_ascii_alphanumeric() {
                    break;
                }
                result = format!("{}{}", &result[..pos], &result[after..]);
            } else {
                break;
            }
        }
        result.trim().to_string()
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

    /// Fallback matching for selectors containing pseudo-classes that scraper can't parse.
    /// Handles :link, :visited (match <a> with href), and strips other pseudo-classes
    /// to attempt base selector matching.
    fn matches_pseudo_class_fallback(
        raw_selector: &str,
        tag_name: &str,
        el: &scraper::node::Element,
    ) -> bool {
        // Extract the pseudo-class and base selector
        // Handle selectors like "a:link", ".class:hover", "div a:visited"
        // We find the last pseudo-class in the selector
        if let Some(colon_idx) = raw_selector.rfind(':') {
            let pseudo_part = &raw_selector[colon_idx + 1..];
            let base = raw_selector[..colon_idx].trim();

            // Extract just the pseudo-class name (before any parentheses)
            let pseudo_name = pseudo_part.split('(').next().unwrap_or("").trim();

            match pseudo_name {
                "link" | "visited" | "any-link" => {
                    // :link / :visited / :any-link apply to <a> elements with href
                    if tag_name != "a" || el.attr("href").is_none() {
                        return false;
                    }
                    // Match the base selector (e.g., "a" from "a:link")
                    if base.is_empty() {
                        return true;
                    }
                    // Try to parse and match the base selector
                    // For simple selectors like "a", check tag match
                    Self::simple_selector_matches(base, tag_name, el)
                }
                "hover" | "active" | "focus" | "focus-visible" | "focus-within" => {
                    // Interactive pseudo-classes don't apply during static rendering
                    false
                }
                // Pseudo-elements create virtual elements inside the target — their styles
                // must NOT apply to the element itself. Both CSS2 (single colon) and CSS3
                // (double colon) syntax end up here because rfind(':') strips the last colon.
                "before" | "after" | "first-letter" | "first-line" | "placeholder"
                | "selection" | "marker" | "backdrop" | "cue" | "grammar-error"
                | "spelling-error" | "target-text" | "file-selector-button"
                | ":before" | ":after" | ":first-letter" | ":first-line"
                | ":placeholder" | ":selection" | ":marker" => {
                    false
                }
                _ => {
                    // Unknown pseudo-class — try stripping it and matching base
                    if base.is_empty() {
                        return false;
                    }
                    Self::simple_selector_matches(base, tag_name, el)
                }
            }
        } else {
            false
        }
    }

    /// Simple selector matching for fallback pseudo-class handling.
    /// Handles tag selectors, class selectors, and ID selectors.
    fn simple_selector_matches(selector: &str, tag_name: &str, el: &scraper::node::Element) -> bool {
        // Try scraper first (it handles complex selectors)
        if let Ok(parsed) = scraper::Selector::parse(selector) {
            // We can't use parsed.matches() without an ElementRef, so fall back to manual
            // For now, do simple matching
        }

        let selector = selector.trim();

        // Simple tag match: "a", "div", etc.
        if selector == tag_name {
            return true;
        }

        // Class-based match: ".classname" or "tag.classname"
        if selector.contains('.') {
            let classes_attr = el.attr("class").unwrap_or("");
            let el_classes: Vec<&str> = classes_attr.split_whitespace().collect();

            // Split selector into tag and classes
            let parts: Vec<&str> = selector.split('.').collect();
            let sel_tag = parts[0]; // May be empty for ".classname"

            // Check tag match (empty means any tag)
            if !sel_tag.is_empty() && sel_tag != tag_name {
                return false;
            }

            // Check all required classes
            for cls in &parts[1..] {
                if !cls.is_empty() && !el_classes.contains(cls) {
                    return false;
                }
            }
            return true;
        }

        // ID-based match: "#id" or "tag#id"
        if selector.contains('#') {
            let parts: Vec<&str> = selector.splitn(2, '#').collect();
            let sel_tag = parts[0];
            let sel_id = parts.get(1).unwrap_or(&"");

            if !sel_tag.is_empty() && sel_tag != tag_name {
                return false;
            }
            if let Some(el_id) = el.attr("id") {
                return el_id == *sel_id;
            }
            return false;
        }

        false
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
                    // Extract the color component from the shorthand.
                    if let Some(color) = Self::extract_color_from_background(&clean_value) {
                        styles.background_color = Some(color);
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
                // Promoted properties (previously in `other` HashMap)
                "flex-wrap" => styles.flex_wrap = Some(clean_value),
                "align-self" => styles.align_self = Some(clean_value),
                "flex-grow" => styles.flex_grow = Some(clean_value),
                "flex-shrink" => styles.flex_shrink = Some(clean_value),
                "flex-basis" => styles.flex_basis = Some(clean_value),
                "flex" => {
                    // flex shorthand: <grow> [<shrink>] [<basis>]
                    let parts: Vec<&str> = clean_value.split_whitespace().collect();
                    match parts.len() {
                        1 if parts[0] == "none" => {
                            styles.flex_grow = Some("0".to_string());
                            styles.flex_shrink = Some("0".to_string());
                            styles.flex_basis = Some("auto".to_string());
                        }
                        1 if parts[0] == "auto" => {
                            styles.flex_grow = Some("1".to_string());
                            styles.flex_shrink = Some("1".to_string());
                            styles.flex_basis = Some("auto".to_string());
                        }
                        1 => {
                            styles.flex_grow = Some(parts[0].to_string());
                            styles.flex_shrink = Some("1".to_string());
                            styles.flex_basis = Some("0%".to_string());
                        }
                        2 => {
                            styles.flex_grow = Some(parts[0].to_string());
                            styles.flex_shrink = Some(parts[1].to_string());
                            styles.flex_basis = Some("0%".to_string());
                        }
                        _ => {
                            styles.flex_grow = Some(parts[0].to_string());
                            styles.flex_shrink = Some(parts[1].to_string());
                            styles.flex_basis = Some(parts[2].to_string());
                        }
                    }
                },
                "min-width" => styles.min_width = Some(clean_value),
                "min-height" => styles.min_height = Some(clean_value),
                "max-width" => styles.max_width = Some(clean_value),
                "max-height" => styles.max_height = Some(clean_value),
                "font-style" => styles.font_style = Some(clean_value),
                "line-height" => styles.line_height = Some(clean_value),
                "text-align" => styles.text_align = Some(clean_value),
                "text-decoration" => styles.text_decoration = Some(clean_value),
                "text-transform" => styles.text_transform = Some(clean_value),
                "white-space" => styles.white_space = Some(clean_value),
                "letter-spacing" => styles.letter_spacing = Some(clean_value),
                "word-spacing" => styles.word_spacing = Some(clean_value),
                "border-radius" => styles.border_radius = Some(clean_value),
                "list-style-type" => styles.list_style_type = Some(clean_value),
                "list-style" => {
                    // list-style shorthand
                    let v = clean_value.trim().to_string();
                    if v == "none" || v.starts_with("none ") || v.contains(" none") {
                        styles.list_style_type = Some("none".to_string());
                    } else {
                        for part in v.split_whitespace() {
                            match part {
                                "disc" | "circle" | "square" | "decimal"
                                | "decimal-leading-zero" | "lower-roman" | "upper-roman"
                                | "lower-alpha" | "upper-alpha" | "lower-latin"
                                | "upper-latin" | "lower-greek" => {
                                    styles.list_style_type = Some(part.to_string());
                                    break;
                                }
                                _ => {}
                            }
                        }
                    }
                },
                "cursor" => styles.cursor = Some(clean_value),
                "grid-template-columns" => styles.grid_template_columns = Some(clean_value),
                "grid-template-rows" => styles.grid_template_rows = Some(clean_value),
                "grid-template-areas" => styles.grid_template_areas = Some(clean_value),
                "grid-area" => styles.grid_area = Some(clean_value),
                "grid-template" => {
                    // grid-template shorthand: <rows> / <columns>
                    // May also include area names: 'name name' <row-size> / <columns>
                    // Extract the columns portion (everything after the last '/')
                    if let Some(slash_pos) = clean_value.rfind('/') {
                        let columns = clean_value[slash_pos + 1..].trim().to_string();
                        if !columns.is_empty() {
                            styles.grid_template_columns = Some(columns);
                        }
                        let rows_part = clean_value[..slash_pos].trim().to_string();
                        if !rows_part.is_empty() {
                            // The rows part may contain area names like 'name name' <size>
                            // Extract area names (single-quoted strings) and row sizes
                            let mut areas = Vec::new();
                            let mut rows = Vec::new();
                            // Tokenize: area names are in single or double quotes, row sizes are outside
                            let mut in_quote = false;
                            let mut quote_char = '\0';
                            let mut current_area = String::new();
                            let mut current_token = String::new();
                            for ch in rows_part.chars() {
                                if (ch == '\'' || ch == '"') && (!in_quote || ch == quote_char) {
                                    if in_quote {
                                        // End of area name
                                        areas.push(current_area.clone());
                                        current_area.clear();
                                        in_quote = false;
                                        quote_char = '\0';
                                    } else {
                                        // Start of area name — flush any accumulated row size token
                                        let token = current_token.trim().to_string();
                                        if !token.is_empty() {
                                            rows.push(token);
                                        }
                                        current_token.clear();
                                        in_quote = true;
                                        quote_char = ch;
                                    }
                                } else if in_quote {
                                    current_area.push(ch);
                                } else {
                                    current_token.push(ch);
                                }
                            }
                            // Flush any remaining row size token
                            let token = current_token.trim().to_string();
                            if !token.is_empty() {
                                rows.push(token);
                            }
                            if !areas.is_empty() {
                                // Format areas as CSS grid-template-areas value:
                                // "'name1 name2' 'name3 name4'"
                                let areas_str = areas.iter()
                                    .map(|a| format!("'{}'", a))
                                    .collect::<Vec<_>>()
                                    .join(" ");
                                styles.grid_template_areas = Some(areas_str);
                            }
                            if !rows.is_empty() {
                                styles.grid_template_rows = Some(rows.join(" "));
                            }
                        }
                    }
                }
                "column-gap" => styles.gap = Some(clean_value),
                "row-gap" => {
                    // Only set gap if column-gap hasn't already set it
                    if styles.gap.is_none() {
                        styles.gap = Some(clean_value);
                    }
                }
                _ => {
                    styles.other.insert(prop.clone(), clean_value);
                }
            }
        }
    }

    /// Extract a color value from a CSS `background` shorthand.
    ///
    /// The background shorthand can contain: color, image (url/gradient), position, size,
    /// repeat, origin, clip, attachment. lightningcss often serializes values like:
    ///   `#eaecf0 none` or `#fff no-repeat` or `transparent url(...)` or `rgb(0, 0, 0) none`
    ///
    /// This extracts the color token by:
    /// 1. If it's a single token with no spaces, return it directly (simple color value)
    /// 2. If it starts with a color function (rgb/hsl/rgba/hsla), extract the full function call
    /// 3. Scan tokens for hex colors, named colors, or 'transparent'/'inherit'
    /// 4. Filter out known background-specific keywords (none, no-repeat, repeat, etc.)
    fn extract_color_from_background(value: &str) -> Option<String> {
        let v = value.trim();

        // Empty
        if v.is_empty() {
            return None;
        }

        // url(...) backgrounds don't have a simple color to extract
        if v.starts_with("url(") {
            return None;
        }

        // Single token, no spaces: treat as a color directly
        if !v.contains(' ') {
            if v == "none" || v == "initial" || v == "unset" {
                return None;
            }
            return Some(v.to_string());
        }

        // If it starts with rgb/hsl/rgba/hsla, extract the function call
        if v.starts_with("rgb") || v.starts_with("hsl") {
            // Find the matching closing paren
            if let Some(open) = v.find('(') {
                let mut depth = 0;
                for (i, ch) in v[open..].char_indices() {
                    match ch {
                        '(' => depth += 1,
                        ')' => {
                            depth -= 1;
                            if depth == 0 {
                                return Some(v[..open + i + 1].to_string());
                            }
                        }
                        _ => {}
                    }
                }
            }
            // Malformed but starts with color function — return entire value
            return Some(v.to_string());
        }

        // Multi-token: scan for a color token
        // Background-specific keywords to filter out
        let bg_keywords = [
            "none", "no-repeat", "repeat", "repeat-x", "repeat-y", "space", "round",
            "scroll", "fixed", "local", "border-box", "padding-box", "content-box",
            "cover", "contain", "center", "top", "bottom", "left", "right",
            "initial", "unset",
        ];

        // Scan tokens — first token that looks like a color wins
        for token in v.split_whitespace() {
            // Skip background-specific keywords
            if bg_keywords.contains(&token.to_lowercase().as_str()) {
                continue;
            }
            // Skip url(...)
            if token.starts_with("url(") {
                return None; // url() background — no simple color
            }
            // Skip percentage/length values (position/size)
            if token.ends_with('%') || token.ends_with("px") || token.ends_with("em")
                || token.ends_with("rem") || token.ends_with("vw") || token.ends_with("vh")
            {
                continue;
            }
            // Skip bare numbers (e.g., "0 0" for position)
            if token.parse::<f64>().is_ok() {
                continue;
            }
            // Skip gradient functions
            if token.starts_with("linear-gradient") || token.starts_with("radial-gradient")
                || token.starts_with("conic-gradient") || token.starts_with("repeating-")
            {
                return None; // gradient background — no simple color
            }

            // Hex color
            if token.starts_with('#') {
                return Some(token.to_string());
            }
            // Named color or transparent/inherit/currentcolor
            if token == "transparent" || token == "inherit" || token == "currentcolor"
                || token == "currentColor"
            {
                return Some(token.to_string());
            }
            // Anything else that doesn't match bg keywords is likely a named color
            // (e.g., "red", "white", "aliceblue")
            if token.chars().all(|c| c.is_ascii_alphabetic()) {
                return Some(token.to_string());
            }
        }

        None
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
    fn test_media_query_screen_and_em_units() {
        // Cloudflare uses Tachyons with `@media screen and (min-width: 60em)` (= 960px)
        let mut processor = CssProcessor::new_with_viewport(1280.0);
        let css = r#"
            @media screen and (min-width: 60em) {
                .dn-l { display: none }
                .flex-l { display: flex }
            }
        "#;
        processor.parse(css).unwrap();
        let rules = processor.get_rules();
        eprintln!("Rules parsed: {} (expected >= 2)", rules.len());
        for rule in rules {
            eprintln!("  selector='{}' declarations={:?}", rule.selector, rule.declarations);
        }
        assert!(rules.len() >= 2, "Expected at least 2 rules from @media screen and (min-width: 60em) at 1280px viewport");

        let styles = processor.compute_style(".dn-l");
        assert_eq!(styles.display, Some("none".to_string()), "Expected .dn-l to have display:none");
    }

    #[test]
    fn test_media_query_only_screen() {
        // Test `only screen and (min-width: 960px)` variant
        let mut processor = CssProcessor::new_with_viewport(1280.0);
        let css = r#"
            @media only screen and (min-width: 960px) {
                .hidden-desktop { display: none }
            }
        "#;
        processor.parse(css).unwrap();
        let rules = processor.get_rules();
        eprintln!("Only screen rules: {}", rules.len());
        assert!(rules.len() >= 1, "Expected rule from @media only screen and (min-width: 960px)");
    }

    #[test]
    fn test_media_query_override_source_order() {
        // Simulates Tachyons: .db (display:block, unconditional) then .dn-l (display:none, media query)
        // At 1280px viewport with min-width: 60em (960px) matching, .dn-l should win by source order
        let mut processor = CssProcessor::new_with_viewport(1280.0);
        let css = r#"
            .db { display: block }
            .dn { display: none }
            @media screen and (min-width: 60em) {
                .dn-l { display: none }
                .db-l { display: block }
            }
        "#;
        processor.parse(css).unwrap();

        let rules = processor.get_rules();
        eprintln!("All rules (count={}):", rules.len());
        for rule in rules {
            eprintln!("  [order={}] selector='{}' spec={:?} decls={:?}",
                rule.source_order, rule.selector, rule.specificity, rule.declarations);
        }

        // Check: an element with class="db dn-l" should get display:none
        // because .dn-l has higher source order
        let html = r#"<html><body><nav class="db dn-l">test</nav></body></html>"#;
        let document = scraper::Html::parse_document(html);
        let nav_sel = scraper::Selector::parse("nav").unwrap();
        let nav = document.select(&nav_sel).next().unwrap();

        let styles = processor.compute_style_for_element(&nav);
        eprintln!("nav.db.dn-l display = {:?}", styles.display);
        assert_eq!(styles.display, Some("none".to_string()),
            "Expected .dn-l (media query, higher source order) to override .db");
    }

    #[test]
    fn test_real_cloudflare_css_media_query() {
        // Load the REAL Cloudflare CSS and test media query application
        let css_path = "/tmp/cloudflare_test.css";
        if !std::path::Path::new(css_path).exists() {
            eprintln!("Skipping test: {} not found", css_path);
            return;
        }
        let css = std::fs::read_to_string(css_path).unwrap();
        let mut processor = CssProcessor::new_with_viewport(1280.0);
        processor.parse(&css).unwrap();

        let rules = processor.get_rules();
        eprintln!("Total rules from Cloudflare CSS: {}", rules.len());

        // Find .dn-l and .db rules specifically
        for rule in rules {
            if rule.selector.contains("dn-l") || rule.selector == ".db" {
                eprintln!("  [order={}] '{}' spec={:?} decls={:?}",
                    rule.source_order, rule.selector, rule.specificity, rule.declarations);
            }
        }

        // Test against a simulated nav element with the actual Cloudflare classes
        let html = r#"<html><body><nav class="bb b--black-10 db dn-l w-100 ph3">test</nav></body></html>"#;
        let document = scraper::Html::parse_document(html);
        let nav_sel = scraper::Selector::parse("nav").unwrap();
        let nav = document.select(&nav_sel).next().unwrap();

        let styles = processor.compute_style_for_element(&nav);
        eprintln!("nav display = {:?}", styles.display);
        assert_eq!(styles.display, Some("none".to_string()),
            "Expected .dn-l from @media screen and (min-width: 60em) to apply at 1280px viewport");
    }

    #[test]
    fn test_real_cloudflare_both_stylesheets() {
        // Test with BOTH external CSS files in order, plus HTML style blocks
        let ashes_path = "/tmp/ashes_test.css";
        let index_path = "/tmp/cloudflare_test.css";
        if !std::path::Path::new(ashes_path).exists() || !std::path::Path::new(index_path).exists() {
            eprintln!("Skipping test: CSS files not found");
            return;
        }

        let ashes_css = std::fs::read_to_string(ashes_path).unwrap();
        let index_css = std::fs::read_to_string(index_path).unwrap();

        let mut processor = CssProcessor::new_with_viewport(1280.0);
        // Parse external CSS in order (same as browser)
        processor.parse(&ashes_css).unwrap();
        processor.parse(&index_css).unwrap();

        // Then parse inline <style> blocks from the actual page
        let style_block_1 = ":root{--header-nav-height:60px;--footer-height:200px}body{margin:0;padding:0}";
        let style_block_2 = "a{color:inherit}";
        processor.parse(style_block_1).unwrap();
        processor.parse(style_block_2).unwrap();

        let rules = processor.get_rules();
        eprintln!("Total rules with both CSS: {}", rules.len());

        // Find .dn-l and .db rules
        let mut db_orders = vec![];
        let mut dn_l_orders = vec![];
        for rule in rules {
            if rule.selector == ".db" {
                db_orders.push(rule.source_order);
            }
            if rule.selector == ".dn-l" {
                dn_l_orders.push(rule.source_order);
            }
        }
        eprintln!(".db source orders: {:?}", db_orders);
        eprintln!(".dn-l source orders: {:?}", dn_l_orders);

        // Test nav element
        let html = r#"<html><body><nav class="bb b--black-10 db dn-l w-100 ph3">test</nav></body></html>"#;
        let document = scraper::Html::parse_document(html);
        let nav_sel = scraper::Selector::parse("nav").unwrap();
        let nav = document.select(&nav_sel).next().unwrap();
        let styles = processor.compute_style_for_element(&nav);
        eprintln!("nav display = {:?}", styles.display);
        assert_eq!(styles.display, Some("none".to_string()),
            "Expected .dn-l to override .db at 1280px viewport");
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
