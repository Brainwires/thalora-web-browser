//! CSS Engine with lightningcss integration
//!
//! Provides CSS parsing, style computation, and minification using the
//! lightningcss library for high-performance CSS processing.

mod declarations;
mod matching;
mod media;
mod parsing;
pub mod types;

pub use types::{
    BorderStyles, BoxModel, CompiledRule, ComputedStyles, CssProcessor, FontFaceEntry, ParsedRule,
    RuleIndex,
};

use anyhow::Result;
use lightningcss::stylesheet::{ParserOptions, PrinterOptions, StyleSheet};
use lightningcss::targets::{Browsers, Targets};
use std::collections::HashMap;

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
            keyframes: HashMap::new(),
            font_faces: Vec::new(),
            layer_order: HashMap::new(),
            layer_order_counter: 0,
            current_layer: None,
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
            keyframes: HashMap::new(),
            font_faces: Vec::new(),
            layer_order: HashMap::new(),
            layer_order_counter: 0,
            current_layer: None,
        }
    }

    /// Set the classes present on the <html> element.
    /// Used to scope CSS custom property definitions: selectors like
    /// `html.skin-theme-clientpref-night` should only contribute custom properties
    /// when the <html> element actually has that class.
    pub fn set_html_classes(&mut self, classes: Vec<String>) {
        self.html_classes = classes;
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
            "flex-wrap" => styles.flex_wrap,
            "align-self" => styles.align_self,
            "flex-grow" => styles.flex_grow,
            "flex-shrink" => styles.flex_shrink,
            "flex-basis" => styles.flex_basis,
            "min-width" => styles.min_width,
            "min-height" => styles.min_height,
            "max-width" => styles.max_width,
            "max-height" => styles.max_height,
            "font-style" => styles.font_style,
            "line-height" => styles.line_height,
            "text-align" => styles.text_align,
            "text-decoration" => styles.text_decoration,
            "text-transform" => styles.text_transform,
            "white-space" => styles.white_space,
            "letter-spacing" => styles.letter_spacing,
            "word-spacing" => styles.word_spacing,
            "border-radius" => styles.border_radius,
            "list-style-type" => styles.list_style_type,
            "cursor" => styles.cursor,
            "grid-template-columns" => styles.grid_template_columns,
            "grid-template-rows" => styles.grid_template_rows,
            "grid-template-areas" => styles.grid_template_areas,
            "grid-area" => styles.grid_area,
            "grid-auto-flow" => styles.grid_auto_flow,
            "grid-auto-rows" => styles.grid_auto_rows,
            "grid-auto-columns" => styles.grid_auto_columns,
            "grid-column" => styles.grid_column,
            "grid-row" => styles.grid_row,
            "float" => styles.float,
            "clear" => styles.clear,
            "transform" => styles.transform,
            "transform-origin" => styles.transform_origin,
            "filter" => styles.filter,
            "backdrop-filter" => styles.backdrop_filter,
            "animation" => styles.animation,
            "animation-name" => styles.animation_name,
            "animation-duration" => styles.animation_duration,
            "transition" => styles.transition,
            "clip-path" => styles.clip_path,
            "mask" => styles.mask,
            "mix-blend-mode" => styles.mix_blend_mode,
            "object-fit" => styles.object_fit,
            "object-position" => styles.object_position,
            "box-shadow" => styles.box_shadow,
            "text-shadow" => styles.text_shadow,
            "outline" => styles.outline,
            "overflow-x" => styles.overflow_x,
            "overflow-y" => styles.overflow_y,
            "text-overflow" => styles.text_overflow,
            "word-break" => styles.word_break,
            "overflow-wrap" => styles.overflow_wrap,
            "vertical-align" => styles.vertical_align,
            "content" => styles.content,
            "pointer-events" => styles.pointer_events,
            "user-select" => styles.user_select,
            "appearance" => styles.appearance,
            "will-change" => styles.will_change,
            "contain" => styles.contain,
            "container-type" => styles.container_type,
            "aspect-ratio" => styles.aspect_ratio,
            "justify-self" => styles.justify_self,
            "direction" => styles.direction,
            "writing-mode" => styles.writing_mode,
            "counter-reset" => styles.counter_reset,
            "counter-increment" => styles.counter_increment,
            _ => styles.other.get(property).cloned(),
        }
    }

    /// Minify CSS using lightningcss
    pub fn minify(&self, css: &str) -> Result<String> {
        let stylesheet = StyleSheet::parse(css, ParserOptions::default())
            .map_err(|e| anyhow::anyhow!("CSS parse error: {:?}", e))?;

        let minified = stylesheet
            .to_css(PrinterOptions {
                minify: true,
                ..Default::default()
            })
            .map_err(|e| anyhow::anyhow!("CSS minify error: {:?}", e))?;

        Ok(minified.code)
    }

    /// Process CSS with vendor prefixes and minification
    pub fn process_css(&self, css: &str) -> Result<String> {
        let stylesheet = StyleSheet::parse(css, ParserOptions::default())
            .map_err(|e| anyhow::anyhow!("CSS parse error: {:?}", e))?;

        // Use default browser targets - browserslist integration requires additional feature
        let targets = Targets::default();

        let result = stylesheet
            .to_css(PrinterOptions {
                minify: false,
                targets,
                ..Default::default()
            })
            .map_err(|e| anyhow::anyhow!("CSS processing error: {:?}", e))?;

        Ok(result.code)
    }

    /// Process CSS with specific browser targets
    pub fn process_css_with_targets(&self, css: &str, browsers: Browsers) -> Result<String> {
        let stylesheet = StyleSheet::parse(css, ParserOptions::default())
            .map_err(|e| anyhow::anyhow!("CSS parse error: {:?}", e))?;

        let result = stylesheet
            .to_css(PrinterOptions {
                minify: false,
                targets: Targets::from(browsers),
                ..Default::default()
            })
            .map_err(|e| anyhow::anyhow!("CSS processing error: {:?}", e))?;

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

    /// Get all known @font-face entries
    pub fn get_font_faces(&self) -> &[FontFaceEntry] {
        &self.font_faces
    }

    /// Get keyframe stops for a named animation
    pub fn get_keyframes(&self, name: &str) -> Option<&Vec<(String, HashMap<String, String>)>> {
        self.keyframes.get(name)
    }

    /// Check if a keyframes animation name is defined
    pub fn has_keyframes(&self, name: &str) -> bool {
        self.keyframes.contains_key(name)
    }

    /// Clear all parsed rules
    pub fn clear(&mut self) {
        self.rules.clear();
        self.sources.clear();
        self.custom_properties.clear();
        self.keyframes.clear();
        self.font_faces.clear();
        self.source_order_counter = 0;
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
        processor
            .parse(".container { display: flex; width: 100%; }")
            .unwrap();

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
        assert_eq!(
            processor.calculate_specificity("div.container#main"),
            (1, 1, 1)
        );
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
            eprintln!(
                "  selector='{}' declarations={:?}",
                rule.selector, rule.declarations
            );
        }
        assert!(
            rules.len() >= 2,
            "Expected at least 2 rules from @media screen and (min-width: 60em) at 1280px viewport"
        );

        let styles = processor.compute_style(".dn-l");
        assert_eq!(
            styles.display,
            Some("none".to_string()),
            "Expected .dn-l to have display:none"
        );
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
        assert!(
            rules.len() >= 1,
            "Expected rule from @media only screen and (min-width: 960px)"
        );
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
            eprintln!(
                "  [order={}] selector='{}' spec={:?} decls={:?}",
                rule.source_order, rule.selector, rule.specificity, rule.declarations
            );
        }

        // Check: an element with class="db dn-l" should get display:none
        // because .dn-l has higher source order
        let html = r#"<html><body><nav class="db dn-l">test</nav></body></html>"#;
        let document = scraper::Html::parse_document(html);
        let nav_sel = scraper::Selector::parse("nav").unwrap();
        let nav = document.select(&nav_sel).next().unwrap();

        let styles = processor.compute_style_for_element(&nav);
        eprintln!("nav.db.dn-l display = {:?}", styles.display);
        assert_eq!(
            styles.display,
            Some("none".to_string()),
            "Expected .dn-l (media query, higher source order) to override .db"
        );
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
                eprintln!(
                    "  [order={}] '{}' spec={:?} decls={:?}",
                    rule.source_order, rule.selector, rule.specificity, rule.declarations
                );
            }
        }

        // Test against a simulated nav element with the actual Cloudflare classes
        let html =
            r#"<html><body><nav class="bb b--black-10 db dn-l w-100 ph3">test</nav></body></html>"#;
        let document = scraper::Html::parse_document(html);
        let nav_sel = scraper::Selector::parse("nav").unwrap();
        let nav = document.select(&nav_sel).next().unwrap();

        let styles = processor.compute_style_for_element(&nav);
        eprintln!("nav display = {:?}", styles.display);
        assert_eq!(
            styles.display,
            Some("none".to_string()),
            "Expected .dn-l from @media screen and (min-width: 60em) to apply at 1280px viewport"
        );
    }

    #[test]
    fn test_real_cloudflare_both_stylesheets() {
        // Test with BOTH external CSS files in order, plus HTML style blocks
        let ashes_path = "/tmp/ashes_test.css";
        let index_path = "/tmp/cloudflare_test.css";
        if !std::path::Path::new(ashes_path).exists() || !std::path::Path::new(index_path).exists()
        {
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
        let style_block_1 =
            ":root{--header-nav-height:60px;--footer-height:200px}body{margin:0;padding:0}";
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
        let html =
            r#"<html><body><nav class="bb b--black-10 db dn-l w-100 ph3">test</nav></body></html>"#;
        let document = scraper::Html::parse_document(html);
        let nav_sel = scraper::Selector::parse("nav").unwrap();
        let nav = document.select(&nav_sel).next().unwrap();
        let styles = processor.compute_style_for_element(&nav);
        eprintln!("nav display = {:?}", styles.display);
        assert_eq!(
            styles.display,
            Some("none".to_string()),
            "Expected .dn-l to override .db at 1280px viewport"
        );
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
        let html =
            r#"<html><body><div class="container"><p class="text">Hello</p></div></body></html>"#;
        let document = scraper::Html::parse_document(html);

        let mut processor = CssProcessor::new();
        // lightningcss normalizes named colors: blue → #00f
        processor
            .parse(".container .text { color: blue; }")
            .unwrap();
        processor.parse("p { font-size: 14px; }").unwrap();

        let p_selector = scraper::Selector::parse("p.text").unwrap();
        if let Some(p_element) = document.select(&p_selector).next() {
            let styles = processor.compute_style_for_element(&p_element);
            assert_eq!(styles.color, Some("#00f".to_string()));
            assert_eq!(styles.font_size, Some("14px".to_string()));
        }
    }
}
