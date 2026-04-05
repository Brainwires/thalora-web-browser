use super::types::{BorderStyles, BoxModel, ComputedStyles, CssProcessor, FontFaceEntry, ParsedRule};
use anyhow::{Context, Result};
use lightningcss::declaration::DeclarationBlock;
use lightningcss::properties::Property;
use lightningcss::rules::CssRule;
use lightningcss::selector::{Component, Selector};
use lightningcss::stylesheet::{ParserOptions, PrinterOptions, StyleSheet};
use lightningcss::traits::ToCss;
use std::collections::HashMap;

impl CssProcessor {
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
                    "opacity" => {
                        if let Ok(v) = value.parse::<f32>() {
                            styles.opacity = Some(v);
                        }
                    }
                    "z-index" => {
                        if let Ok(v) = value.parse::<i32>() {
                            styles.z_index = Some(v);
                        }
                    }
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
                    }
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
                    // Border shorthand: border: <width> <style> <color>
                    "border" => {
                        if value.trim() == "none" || value.trim() == "0" {
                            styles.border = Some(BorderStyles {
                                width: "0".into(),
                                style: "none".into(),
                                color: String::new(),
                            });
                        } else {
                            styles.border = Some(Self::parse_border_shorthand(&value));
                        }
                    }
                    // Per-side border shorthands: border-top/right/bottom/left
                    "border-top" => {
                        if value.trim() == "none" || value.trim() == "0" {
                            styles.border_top = Some(BorderStyles {
                                width: "0".into(),
                                style: "none".into(),
                                color: String::new(),
                            });
                        } else {
                            styles.border_top = Some(Self::parse_border_shorthand(&value));
                        }
                    }
                    "border-right" => {
                        if value.trim() == "none" || value.trim() == "0" {
                            styles.border_right = Some(BorderStyles {
                                width: "0".into(),
                                style: "none".into(),
                                color: String::new(),
                            });
                        } else {
                            styles.border_right = Some(Self::parse_border_shorthand(&value));
                        }
                    }
                    "border-bottom" => {
                        if value.trim() == "none" || value.trim() == "0" {
                            styles.border_bottom = Some(BorderStyles {
                                width: "0".into(),
                                style: "none".into(),
                                color: String::new(),
                            });
                        } else {
                            styles.border_bottom = Some(Self::parse_border_shorthand(&value));
                        }
                    }
                    "border-left" => {
                        if value.trim() == "none" || value.trim() == "0" {
                            styles.border_left = Some(BorderStyles {
                                width: "0".into(),
                                style: "none".into(),
                                color: String::new(),
                            });
                        } else {
                            styles.border_left = Some(Self::parse_border_shorthand(&value));
                        }
                    }
                    // Individual longhand border properties
                    "border-width" => {
                        let b = styles.border.get_or_insert_with(BorderStyles::default);
                        b.width = value;
                    }
                    "border-style" => {
                        let b = styles.border.get_or_insert_with(BorderStyles::default);
                        b.style = value;
                    }
                    "border-color" => {
                        let b = styles.border.get_or_insert_with(BorderStyles::default);
                        b.color = value;
                    }
                    // Per-side longhand properties
                    "border-top-width" => {
                        let b = styles.border_top.get_or_insert_with(BorderStyles::default);
                        b.width = value;
                    }
                    "border-top-style" => {
                        let b = styles.border_top.get_or_insert_with(BorderStyles::default);
                        b.style = value;
                    }
                    "border-top-color" => {
                        let b = styles.border_top.get_or_insert_with(BorderStyles::default);
                        b.color = value;
                    }
                    "border-right-width" => {
                        let b = styles
                            .border_right
                            .get_or_insert_with(BorderStyles::default);
                        b.width = value;
                    }
                    "border-right-style" => {
                        let b = styles
                            .border_right
                            .get_or_insert_with(BorderStyles::default);
                        b.style = value;
                    }
                    "border-right-color" => {
                        let b = styles
                            .border_right
                            .get_or_insert_with(BorderStyles::default);
                        b.color = value;
                    }
                    "border-bottom-width" => {
                        let b = styles
                            .border_bottom
                            .get_or_insert_with(BorderStyles::default);
                        b.width = value;
                    }
                    "border-bottom-style" => {
                        let b = styles
                            .border_bottom
                            .get_or_insert_with(BorderStyles::default);
                        b.style = value;
                    }
                    "border-bottom-color" => {
                        let b = styles
                            .border_bottom
                            .get_or_insert_with(BorderStyles::default);
                        b.color = value;
                    }
                    "border-left-width" => {
                        let b = styles.border_left.get_or_insert_with(BorderStyles::default);
                        b.width = value;
                    }
                    "border-left-style" => {
                        let b = styles.border_left.get_or_insert_with(BorderStyles::default);
                        b.style = value;
                    }
                    "border-left-color" => {
                        let b = styles.border_left.get_or_insert_with(BorderStyles::default);
                        b.color = value;
                    }
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
                                    "disc"
                                    | "circle"
                                    | "square"
                                    | "decimal"
                                    | "decimal-leading-zero"
                                    | "lower-roman"
                                    | "upper-roman"
                                    | "lower-alpha"
                                    | "upper-alpha"
                                    | "lower-latin"
                                    | "upper-latin"
                                    | "lower-greek" => {
                                        styles.list_style_type = Some(part.to_string());
                                        break;
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    "cursor" => styles.cursor = Some(value),
                    "counter-reset" => styles.counter_reset = Some(value),
                    "counter-increment" => styles.counter_increment = Some(value),
                    "grid-template-columns" => styles.grid_template_columns = Some(value),
                    "grid-template-rows" => styles.grid_template_rows = Some(value),
                    "grid-template-areas" => styles.grid_template_areas = Some(value),
                    "grid-area" => styles.grid_area = Some(value),
                    "grid-auto-flow" => styles.grid_auto_flow = Some(value),
                    "grid-auto-rows" => styles.grid_auto_rows = Some(value),
                    "grid-auto-columns" => styles.grid_auto_columns = Some(value),
                    "grid-column" => styles.grid_column = Some(value),
                    "grid-row" => styles.grid_row = Some(value),
                    "grid-column-start" | "grid-column-end" => {
                        let existing = styles.grid_column.clone().unwrap_or_default();
                        if prop == "grid-column-start" {
                            if let Some(end) = existing.split('/').nth(1) {
                                styles.grid_column = Some(format!("{} / {}", value, end.trim()));
                            } else {
                                styles.grid_column = Some(value);
                            }
                        } else {
                            let start = existing.split('/').next().unwrap_or("auto").trim().to_string();
                            styles.grid_column = Some(format!("{} / {}", start, value));
                        }
                    }
                    "grid-row-start" | "grid-row-end" => {
                        let existing = styles.grid_row.clone().unwrap_or_default();
                        if prop == "grid-row-start" {
                            if let Some(end) = existing.split('/').nth(1) {
                                styles.grid_row = Some(format!("{} / {}", value, end.trim()));
                            } else {
                                styles.grid_row = Some(value);
                            }
                        } else {
                            let start = existing.split('/').next().unwrap_or("auto").trim().to_string();
                            styles.grid_row = Some(format!("{} / {}", start, value));
                        }
                    }
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
                                    if (ch == '\'' || ch == '"') && (!in_quote || ch == quote_char)
                                    {
                                        if in_quote {
                                            areas.push(current_area.clone());
                                            current_area.clear();
                                            in_quote = false;
                                            quote_char = '\0';
                                        } else {
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
                                let token = current_token.trim().to_string();
                                if !token.is_empty() {
                                    rows.push(token);
                                }
                                if !areas.is_empty() {
                                    let areas_str = areas
                                        .iter()
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
                        if styles.gap.is_none() {
                            styles.gap = Some(value);
                        }
                    }
                    "float" => styles.float = Some(value),
                    "clear" => styles.clear = Some(value),
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
        if s == ":root"
            || s == "html"
            || s == "*"
            || s == "*, :before, :after"
            || s == "*, ::before, ::after"
            || s == ":before"
            || s == ":after"
            || s == "::before"
            || s == "::after"
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
            let class_name = part
                .split(|c: char| c == ':' || c == '[' || c == '#')
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
                while j < len
                    && (bytes[j] == b' '
                        || bytes[j] == b'\n'
                        || bytes[j] == b'\r'
                        || bytes[j] == b'\t')
                {
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
                    let selector_str = style_rule
                        .selectors
                        .to_css_string(PrinterOptions::default())
                        .unwrap_or_default();

                    // Calculate specificity
                    let specificity = self.calculate_specificity(&selector_str);

                    // Extract declarations
                    let mut declarations = HashMap::new();
                    for decl in style_rule.declarations.declarations.iter() {
                        let prop_name = decl
                            .property_id()
                            .to_css_string(PrinterOptions::default())
                            .unwrap_or_default();
                        let prop_value = decl
                            .value_to_css_string(PrinterOptions::default())
                            .unwrap_or_default();
                        declarations.insert(prop_name, prop_value);
                    }

                    // Also handle important declarations
                    for decl in style_rule.declarations.important_declarations.iter() {
                        let prop_name = decl
                            .property_id()
                            .to_css_string(PrinterOptions::default())
                            .unwrap_or_default();
                        let prop_value = format!(
                            "{} !important",
                            decl.value_to_css_string(PrinterOptions::default())
                                .unwrap_or_default()
                        );
                        declarations.insert(prop_name, prop_value);
                    }

                    // Collect custom properties from :root rules (or any rule)
                    let is_root_selector = selector_str.split(',').any(|s| {
                        let s = s.trim();
                        s == ":root" || s == "html" || s == ":root, html" || s == "html, :root"
                    });

                    // Custom properties (CSS variables) need scoping:
                    // Only store them if at least one selector alternative would match
                    // the document root. Selectors like `html.skin-theme-clientpref-night`
                    // should only contribute their custom properties when the <html> element
                    // actually has that class.
                    let should_store_vars = selector_str
                        .split(',')
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

                    let layer = self.current_layer.as_ref()
                        .and_then(|name| self.layer_order.get(name).copied());

                    self.rules.push(ParsedRule {
                        selector: selector_str,
                        specificity,
                        declarations,
                        source_order,
                        layer,
                    });
                }
                CssRule::Media(media_rule) => {
                    // Evaluate media query against viewport
                    let media_str = media_rule
                        .query
                        .to_css_string(PrinterOptions::default())
                        .unwrap_or_default();

                    let matches = self.evaluate_media_query(&media_str);
                    if matches {
                        // Media query matches — process inner rules
                        self.process_rules(&media_rule.rules.0);
                    }
                }
                CssRule::Supports(supports_rule) => {
                    // Evaluate @supports condition — if the browser supports the
                    // declared property/value, process inner rules
                    if self.evaluate_supports_condition(&supports_rule.condition) {
                        self.process_rules(&supports_rule.rules.0);
                    }
                }
                CssRule::LayerBlock(layer_rule) => {
                    // @layer block: process inner rules transparently (source order).
                    // Full cascade layer priority ordering is not yet implemented,
                    // but this is far better than silently dropping the rules.
                    self.process_rules(&layer_rule.rules.0);
                }
                CssRule::Container(container_rule) => {
                    // @container: permissive strategy — always include inner rules.
                    // Without layout integration we can't evaluate the container
                    // condition, but dropping rules entirely breaks more sites than
                    // including them unconditionally.
                    self.process_rules(&container_rule.rules.0);
                }
                CssRule::Nesting(nesting_rule) => {
                    // CSS @nest rule: treat the inner style rule as a regular style rule
                    self.process_rules(&[CssRule::Style(nesting_rule.style.clone())]);
                }
                CssRule::Scope(scope_rule) => {
                    // @scope: process inner rules transparently.
                    // Proper scoping (scope_start/scope_end selectors) is not yet
                    // implemented, but including the rules prevents content loss.
                    self.process_rules(&scope_rule.rules.0);
                }
                CssRule::StartingStyle(starting_style_rule) => {
                    // @starting-style: process inner rules. These define initial
                    // styles for transition origins — include them so JS can read
                    // the computed values even without transition support.
                    self.process_rules(&starting_style_rule.rules.0);
                }
                CssRule::Keyframes(keyframes_rule) => {
                    // Store @keyframes with full declaration data per keyframe stop.
                    let name = keyframes_rule
                        .name
                        .to_css_string(PrinterOptions::default())
                        .unwrap_or_default();
                    if !name.is_empty() {
                        let mut stops = Vec::new();
                        for keyframe in &keyframes_rule.keyframes {
                            let selector_str = keyframe
                                .selectors
                                .iter()
                                .map(|s| {
                                    s.to_css_string(PrinterOptions::default())
                                        .unwrap_or_default()
                                })
                                .collect::<Vec<_>>()
                                .join(", ");
                            let mut decls = HashMap::new();
                            for decl in keyframe.declarations.declarations.iter() {
                                let prop = decl
                                    .property_id()
                                    .to_css_string(PrinterOptions::default())
                                    .unwrap_or_default();
                                let val = decl
                                    .value_to_css_string(PrinterOptions::default())
                                    .unwrap_or_default();
                                decls.insert(prop, val);
                            }
                            stops.push((selector_str, decls));
                        }
                        self.keyframes.insert(name, stops);
                    }
                }
                CssRule::FontFace(font_face_rule) => {
                    // Store @font-face for font resolution using the typed property API.
                    let mut family = String::new();
                    let mut src = String::new();
                    let mut weight = None;
                    let mut style = None;
                    let mut unicode_range = None;

                    for prop in &font_face_rule.properties {
                        match prop {
                            lightningcss::rules::font_face::FontFaceProperty::FontFamily(ff) => {
                                family = ff
                                    .to_css_string(PrinterOptions::default())
                                    .unwrap_or_default()
                                    .trim_matches('"')
                                    .trim_matches('\'')
                                    .to_string();
                            }
                            lightningcss::rules::font_face::FontFaceProperty::Source(sources) => {
                                let parts: Vec<String> = sources
                                    .iter()
                                    .filter_map(|s| {
                                        s.to_css_string(PrinterOptions::default()).ok()
                                    })
                                    .collect();
                                src = parts.join(", ");
                            }
                            lightningcss::rules::font_face::FontFaceProperty::FontWeight(w) => {
                                weight = w.to_css_string(PrinterOptions::default()).ok();
                            }
                            lightningcss::rules::font_face::FontFaceProperty::FontStyle(s) => {
                                style = s.to_css_string(PrinterOptions::default()).ok();
                            }
                            lightningcss::rules::font_face::FontFaceProperty::UnicodeRange(
                                ranges,
                            ) => {
                                let parts: Vec<String> = ranges
                                    .iter()
                                    .filter_map(|r| {
                                        r.to_css_string(PrinterOptions::default()).ok()
                                    })
                                    .collect();
                                unicode_range = Some(parts.join(", "));
                            }
                            _ => {}
                        }
                    }

                    if !family.is_empty() {
                        self.font_faces.push(FontFaceEntry {
                            family,
                            src,
                            weight,
                            style,
                            display: None, // font-display is not in the typed enum
                            unicode_range,
                        });
                    }
                }
                _ => {
                    // Skip @import, @font-palette-values, etc.
                }
            }
        }
    }
}
