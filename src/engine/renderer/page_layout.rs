//! Page Layout Bridge
//!
//! Takes raw HTML, extracts `<style>` blocks, walks the DOM to compute CSS per element
//! via CssProcessor (lightningcss), builds a LayoutElement tree, runs LayoutEngine (taffy),
//! and returns a LayoutResult with full visual properties suitable for JSON serialization
//! and consumption by the C# GUI layer.

use anyhow::{Context, Result};
use scraper::{ElementRef, Html, Node, Selector};
use std::collections::HashMap;
use std::time::Instant;

/// Document compatibility mode per HTML5 spec §13.2.3.3.
/// Determined by inspecting the DOCTYPE declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentMode {
    /// Standards mode: `<!DOCTYPE html>` or valid HTML5 DOCTYPE
    Standards,
    /// Almost standards mode (limited quirks): HTML 4.01 Transitional/Frameset with system ID
    AlmostStandards,
    /// Quirks mode: no DOCTYPE, legacy DOCTYPEs, or missing system ID on transitional DOCTYPEs
    Quirks,
}

/// Detect the document compatibility mode from the parsed HTML document.
///
/// Implements the DOCTYPE sniffing algorithm from the HTML5 spec:
/// - No DOCTYPE → Quirks
/// - `<!DOCTYPE html>` (no public/system ID) → Standards
/// - Known quirks-mode public IDs → Quirks
/// - Transitional/Frameset with system ID → AlmostStandards
/// - Everything else with `html` name → Standards
pub fn detect_document_mode(document: &Html) -> DocumentMode {
    // Walk the document tree to find the doctype node
    let tree = document.tree.root();
    let mut doctype_found = false;
    let mut doctype_name = String::new();
    let mut public_id = String::new();
    let mut system_id = String::new();

    for child in tree.children() {
        if let Some(dt) = child.value().as_doctype() {
            doctype_found = true;
            doctype_name = dt.name().to_lowercase();
            public_id = dt.public_id().to_lowercase();
            system_id = dt.system_id().to_lowercase();
            break;
        }
    }

    if !doctype_found {
        return DocumentMode::Quirks;
    }

    // DOCTYPE name must be "html" for standards mode
    if doctype_name != "html" {
        return DocumentMode::Quirks;
    }

    // `<!DOCTYPE html>` with no public/system ID → Standards (HTML5)
    if public_id.is_empty() && system_id.is_empty() {
        return DocumentMode::Standards;
    }

    // `<!DOCTYPE html SYSTEM "about:legacy-compat">` → Standards (HTML5 legacy-compat)
    if public_id.is_empty() && system_id == "about:legacy-compat" {
        return DocumentMode::Standards;
    }

    // Known quirks-mode public ID prefixes (per HTML5 spec §13.2.3.3)
    let quirks_public_prefixes = [
        "+//silmaril//dtd html pro v0r11 19970101//",
        "-//as//dtd html 3.0 aswedit 7//",
        "-//advasoft ltd//dtd html 3.0 aswedit 7//",
        "-//ietf//dtd html 2.0 level 1//",
        "-//ietf//dtd html 2.0 level 2//",
        "-//ietf//dtd html 2.0 strict level 1//",
        "-//ietf//dtd html 2.0 strict level 2//",
        "-//ietf//dtd html 2.0 strict//",
        "-//ietf//dtd html 2.0//",
        "-//ietf//dtd html 2.1e//",
        "-//ietf//dtd html 3.0//",
        "-//ietf//dtd html 3.2 final//",
        "-//ietf//dtd html 3.2//",
        "-//ietf//dtd html 3//",
        "-//ietf//dtd html level 0//",
        "-//ietf//dtd html level 1//",
        "-//ietf//dtd html level 2//",
        "-//ietf//dtd html level 3//",
        "-//ietf//dtd html strict level 0//",
        "-//ietf//dtd html strict level 1//",
        "-//ietf//dtd html strict level 2//",
        "-//ietf//dtd html strict level 3//",
        "-//ietf//dtd html strict//",
        "-//ietf//dtd html//",
        "-//metrius//dtd metrius presentational//",
        "-//microsoft//dtd internet explorer 2.0 html strict//",
        "-//microsoft//dtd internet explorer 2.0 html//",
        "-//microsoft//dtd internet explorer 2.0 tables//",
        "-//microsoft//dtd internet explorer 3.0 html strict//",
        "-//microsoft//dtd internet explorer 3.0 html//",
        "-//microsoft//dtd internet explorer 3.0 tables//",
        "-//netscape comm. corp.//dtd html//",
        "-//netscape comm. corp.//dtd strict html//",
        "-//o'reilly and associates//dtd html 2.0//",
        "-//o'reilly and associates//dtd html extended 1.0//",
        "-//o'reilly and associates//dtd html extended relaxed 1.0//",
        "-//sq//dtd html 2.0 hotmetal + extensions//",
        "-//softquad software//dtd hotmetal pro 6.0::19990601::extensions to html 4.0//",
        "-//softquad//dtd hotmetal pro 4.0::19971010::extensions to html 4.0//",
        "-//spyglass//dtd html 2.0 extended//",
        "-//sun microsystems corp.//dtd hotjava html//",
        "-//sun microsystems corp.//dtd hotjava strict html//",
        "-//w3c//dtd html 3 1995-03-24//",
        "-//w3c//dtd html 3.2 draft//",
        "-//w3c//dtd html 3.2 final//",
        "-//w3c//dtd html 3.2//",
        "-//w3c//dtd html 3.2s draft//",
        "-//w3c//dtd html 4.0 frameset//",
        "-//w3c//dtd html 4.0 transitional//",
        "-//w3c//dtd html experimental 19960712//",
        "-//w3c//dtd html experimental 970421//",
        "-//w3c//dtd w3 html//",
        "-//w3o//dtd w3 html 3.0//",
        "-//webtechs//dtd mozilla html 2.0//",
        "-//webtechs//dtd mozilla html//",
    ];

    for prefix in &quirks_public_prefixes {
        if public_id.starts_with(prefix) {
            return DocumentMode::Quirks;
        }
    }

    // HTML 4.01 Transitional/Frameset without system ID → Quirks
    let transitional_public = "-//w3c//dtd html 4.01 transitional//";
    let frameset_public = "-//w3c//dtd html 4.01 frameset//";
    if (public_id.starts_with(transitional_public) || public_id.starts_with(frameset_public))
        && system_id.is_empty()
    {
        return DocumentMode::Quirks;
    }

    // HTML 4.01 Transitional/Frameset WITH system ID → AlmostStandards
    if public_id.starts_with(transitional_public) || public_id.starts_with(frameset_public) {
        return DocumentMode::AlmostStandards;
    }

    // XHTML 1.0 Transitional/Frameset → AlmostStandards
    let xhtml_transitional = "-//w3c//dtd xhtml 1.0 transitional//";
    let xhtml_frameset = "-//w3c//dtd xhtml 1.0 frameset//";
    if public_id.starts_with(xhtml_transitional) || public_id.starts_with(xhtml_frameset) {
        return DocumentMode::AlmostStandards;
    }

    // Default for any other DOCTYPE html with a public ID → Standards
    DocumentMode::Standards
}

use super::css::{BorderStyles, BoxModel, ComputedStyles, CssProcessor};

/// CSS counter state machine for `counter-reset`, `counter-increment`, and `counter()`.
#[derive(Debug, Clone, Default)]
struct CounterState {
    /// Current counter values: name → value
    counters: HashMap<String, Vec<i32>>,
}

impl CounterState {
    /// Process `counter-reset` property (e.g., "section 0", "item", "section 0 item 0")
    fn apply_reset(&mut self, value: &str) {
        let tokens: Vec<&str> = value.split_whitespace().collect();
        let mut i = 0;
        while i < tokens.len() {
            let name = tokens[i];
            let init_val = if i + 1 < tokens.len() {
                tokens[i + 1].parse::<i32>().unwrap_or(0)
            } else {
                0
            };
            // Check if next token was consumed as a number
            if i + 1 < tokens.len() && tokens[i + 1].parse::<i32>().is_ok() {
                i += 2;
            } else {
                i += 1;
            }
            // Push a new counter scope
            self.counters
                .entry(name.to_string())
                .or_default()
                .push(init_val);
        }
    }

    /// Process `counter-increment` property (e.g., "section", "item 2")
    fn apply_increment(&mut self, value: &str) {
        let tokens: Vec<&str> = value.split_whitespace().collect();
        let mut i = 0;
        while i < tokens.len() {
            let name = tokens[i];
            let inc = if i + 1 < tokens.len() {
                tokens[i + 1].parse::<i32>().unwrap_or(1)
            } else {
                1
            };
            if i + 1 < tokens.len() && tokens[i + 1].parse::<i32>().is_ok() {
                i += 2;
            } else {
                i += 1;
            }
            // Increment the innermost counter with this name
            if let Some(stack) = self.counters.get_mut(name) {
                if let Some(last) = stack.last_mut() {
                    *last += inc;
                }
            } else {
                // Auto-create at document level per spec
                self.counters.entry(name.to_string()).or_default().push(inc);
            }
        }
    }

    /// Get current value of a counter
    fn get(&self, name: &str) -> i32 {
        self.counters
            .get(name)
            .and_then(|stack| stack.last().copied())
            .unwrap_or(0)
    }

    /// Get all values of a counter (for `counters()` function with separator)
    fn get_all(&self, name: &str) -> Vec<i32> {
        self.counters.get(name).cloned().unwrap_or_default()
    }

    /// Resolve `counter()` and `counters()` references in a CSS content value.
    /// Handles: `counter(name)`, `counter(name, list-style)`, `counters(name, separator)`
    fn resolve_content(&self, content: &str) -> String {
        let mut result = content.to_string();

        // Resolve counter(name) and counter(name, style)
        while let Some(start) = result.find("counter(") {
            let after = start + 8;
            if let Some(rel_end) = result[after..].find(')') {
                let end = after + rel_end;
                let inner = &result[after..end];
                let parts: Vec<&str> = inner.splitn(2, ',').map(|s| s.trim()).collect();
                let name = parts[0];
                let val = self.get(name);
                let formatted = if parts.len() > 1 {
                    format_counter_value(val, parts[1])
                } else {
                    val.to_string()
                };
                result = format!("{}{}{}", &result[..start], formatted, &result[end + 1..]);
            } else {
                break;
            }
        }

        // Resolve counters(name, separator) and counters(name, separator, style)
        while let Some(start) = result.find("counters(") {
            let after = start + 9;
            if let Some(rel_end) = result[after..].find(')') {
                let end = after + rel_end;
                let inner = &result[after..end];
                let parts: Vec<&str> = inner.splitn(3, ',').map(|s| s.trim()).collect();
                if parts.len() >= 2 {
                    let name = parts[0];
                    let separator = parts[1].trim_matches('"').trim_matches('\'');
                    let style = parts.get(2).map(|s| s.trim()).unwrap_or("decimal");
                    let values = self.get_all(name);
                    let formatted: Vec<String> = values
                        .iter()
                        .map(|v| format_counter_value(*v, style))
                        .collect();
                    let joined = formatted.join(separator);
                    result = format!("{}{}{}", &result[..start], joined, &result[end + 1..]);
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        result
    }

    /// Pop counter scopes that were pushed by counter-reset in this element
    fn pop_reset(&mut self, value: &str) {
        let tokens: Vec<&str> = value.split_whitespace().collect();
        let mut i = 0;
        while i < tokens.len() {
            let name = tokens[i];
            if i + 1 < tokens.len() && tokens[i + 1].parse::<i32>().is_ok() {
                i += 2;
            } else {
                i += 1;
            }
            if let Some(stack) = self.counters.get_mut(name) {
                stack.pop();
                if stack.is_empty() {
                    self.counters.remove(name);
                }
            }
        }
    }
}

/// Format a counter value according to a list-style-type
fn format_counter_value(val: i32, style: &str) -> String {
    match style {
        "decimal" | "" => val.to_string(),
        "decimal-leading-zero" => format!("{:02}", val),
        "lower-alpha" | "lower-latin" => {
            if val >= 1 && val <= 26 {
                char::from(b'a' + (val - 1) as u8).to_string()
            } else {
                val.to_string()
            }
        }
        "upper-alpha" | "upper-latin" => {
            if val >= 1 && val <= 26 {
                char::from(b'A' + (val - 1) as u8).to_string()
            } else {
                val.to_string()
            }
        }
        "lower-roman" => to_roman(val, false),
        "upper-roman" => to_roman(val, true),
        "disc" => "\u{2022}".to_string(),   // •
        "circle" => "\u{25CB}".to_string(), // ○
        "square" => "\u{25A0}".to_string(), // ■
        "none" => String::new(),
        _ => val.to_string(),
    }
}

/// Convert an integer to Roman numerals
fn to_roman(mut val: i32, upper: bool) -> String {
    if val <= 0 || val > 3999 {
        return val.to_string();
    }
    let numerals = [
        (1000, "m"),
        (900, "cm"),
        (500, "d"),
        (400, "cd"),
        (100, "c"),
        (90, "xc"),
        (50, "l"),
        (40, "xl"),
        (10, "x"),
        (9, "ix"),
        (5, "v"),
        (4, "iv"),
        (1, "i"),
    ];
    let mut result = String::new();
    for &(value, numeral) in &numerals {
        while val >= value {
            result.push_str(numeral);
            val -= value;
        }
    }
    if upper { result.to_uppercase() } else { result }
}

/// Resolve CSS `content` property value to plain text.
/// Handles: quoted strings, `counter()`, `counters()`, `attr()`, and concatenation.
fn resolve_css_content(
    content: &str,
    counter_state: &CounterState,
    el: Option<&scraper::node::Element>,
) -> Option<String> {
    let content = content.trim();
    if content == "none" || content == "normal" || content.is_empty() {
        return None;
    }

    let mut result = String::new();
    let mut chars = content.char_indices().peekable();

    while let Some(&(i, c)) = chars.peek() {
        if c == '"' || c == '\'' {
            // Quoted string
            let quote = c;
            chars.next();
            while let Some(&(_, ch)) = chars.peek() {
                chars.next();
                if ch == quote {
                    break;
                }
                if ch == '\\' {
                    // Escape
                    if let Some(&(_, esc)) = chars.peek() {
                        chars.next();
                        result.push(esc);
                    }
                } else {
                    result.push(ch);
                }
            }
        } else if c == 'c' && content[i..].starts_with("counter(") {
            // counter() — find matching paren
            let start = i;
            let after = i + 8;
            if let Some(rel_end) = content[after..].find(')') {
                let end = after + rel_end;
                let fragment = &content[start..end + 1];
                result.push_str(&counter_state.resolve_content(fragment));
                // Advance past
                while let Some(&(j, _)) = chars.peek() {
                    if j > end {
                        break;
                    }
                    chars.next();
                }
                continue;
            }
            chars.next();
        } else if c == 'c' && content[i..].starts_with("counters(") {
            let start = i;
            let after = i + 9;
            if let Some(rel_end) = content[after..].find(')') {
                let end = after + rel_end;
                let fragment = &content[start..end + 1];
                result.push_str(&counter_state.resolve_content(fragment));
                while let Some(&(j, _)) = chars.peek() {
                    if j > end {
                        break;
                    }
                    chars.next();
                }
                continue;
            }
            chars.next();
        } else if c == 'a' && content[i..].starts_with("attr(") {
            let after = i + 5;
            if let Some(rel_end) = content[after..].find(')') {
                let end = after + rel_end;
                let attr_name = content[after..end].trim();
                if let Some(el) = el {
                    if let Some(val) = el.attr(attr_name) {
                        result.push_str(val);
                    }
                }
                while let Some(&(j, _)) = chars.peek() {
                    if j > end {
                        break;
                    }
                    chars.next();
                }
                continue;
            }
            chars.next();
        } else if c == ' ' {
            // Skip whitespace between content parts
            chars.next();
        } else {
            chars.next();
        }
    }

    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}
use super::layout::{
    BoxModelSides, ElementLayout, LayoutElement, LayoutEngine, LayoutResult, parse_px_value,
};
use super::styled_tree::{ResolvedStyles, StyleBoxSides, StyledElement, StyledTreeResult};

/// User-agent default styles for block-level elements.
/// These mirror the CSS 2.1 spec defaults that browsers apply.
fn apply_ua_defaults(tag: &str, styles: &mut ComputedStyles, doc_mode: DocumentMode) {
    match tag {
        "html" => {
            if styles.display.is_none() {
                styles.display = Some("block".to_string());
            }
            // CSS spec: the html element's background is transparent by default.
            // When html has no background, the body's background propagates to the canvas.
            // We do NOT set a default background here — the C# side handles canvas background
            // propagation (WebContentControl.Background is white by default).
            // Default text color is black (CSS initial value). This inherits to all children.
            if styles.color.is_none() {
                styles.color = Some("#000000".to_string());
            }
        }
        "body" => {
            if styles.display.is_none() {
                styles.display = Some("block".to_string());
            }
            if styles.margin.is_none() {
                styles.margin = Some(BoxModel {
                    top: "8px".to_string(),
                    right: "8px".to_string(),
                    bottom: "8px".to_string(),
                    left: "8px".to_string(),
                });
            }
        }
        "h1" => {
            if styles.display.is_none() {
                styles.display = Some("block".to_string());
            }
            if styles.font_size.is_none() {
                styles.font_size = Some("32px".to_string());
            }
            if styles.font_weight.is_none() {
                styles.font_weight = Some("bold".to_string());
            }
            if styles.margin.is_none() {
                styles.margin = Some(BoxModel {
                    top: "21.44px".to_string(),
                    right: "0px".to_string(),
                    bottom: "21.44px".to_string(),
                    left: "0px".to_string(),
                });
            }
        }
        "h2" => {
            if styles.display.is_none() {
                styles.display = Some("block".to_string());
            }
            if styles.font_size.is_none() {
                styles.font_size = Some("24px".to_string());
            }
            if styles.font_weight.is_none() {
                styles.font_weight = Some("bold".to_string());
            }
            if styles.margin.is_none() {
                styles.margin = Some(BoxModel {
                    top: "19.92px".to_string(),
                    right: "0px".to_string(),
                    bottom: "19.92px".to_string(),
                    left: "0px".to_string(),
                });
            }
        }
        "h3" => {
            if styles.display.is_none() {
                styles.display = Some("block".to_string());
            }
            if styles.font_size.is_none() {
                styles.font_size = Some("18.72px".to_string());
            }
            if styles.font_weight.is_none() {
                styles.font_weight = Some("bold".to_string());
            }
            if styles.margin.is_none() {
                styles.margin = Some(BoxModel {
                    top: "18.72px".to_string(),
                    right: "0px".to_string(),
                    bottom: "18.72px".to_string(),
                    left: "0px".to_string(),
                });
            }
        }
        "h4" => {
            if styles.display.is_none() {
                styles.display = Some("block".to_string());
            }
            if styles.font_size.is_none() {
                styles.font_size = Some("16px".to_string());
            }
            if styles.font_weight.is_none() {
                styles.font_weight = Some("bold".to_string());
            }
            if styles.margin.is_none() {
                styles.margin = Some(BoxModel {
                    top: "21.28px".to_string(),
                    right: "0px".to_string(),
                    bottom: "21.28px".to_string(),
                    left: "0px".to_string(),
                });
            }
        }
        "h5" => {
            if styles.display.is_none() {
                styles.display = Some("block".to_string());
            }
            if styles.font_size.is_none() {
                styles.font_size = Some("13.28px".to_string());
            }
            if styles.font_weight.is_none() {
                styles.font_weight = Some("bold".to_string());
            }
            if styles.margin.is_none() {
                styles.margin = Some(BoxModel {
                    top: "22.18px".to_string(),
                    right: "0px".to_string(),
                    bottom: "22.18px".to_string(),
                    left: "0px".to_string(),
                });
            }
        }
        "h6" => {
            if styles.display.is_none() {
                styles.display = Some("block".to_string());
            }
            if styles.font_size.is_none() {
                styles.font_size = Some("10.72px".to_string());
            }
            if styles.font_weight.is_none() {
                styles.font_weight = Some("bold".to_string());
            }
            if styles.margin.is_none() {
                styles.margin = Some(BoxModel {
                    top: "24.97px".to_string(),
                    right: "0px".to_string(),
                    bottom: "24.97px".to_string(),
                    left: "0px".to_string(),
                });
            }
        }
        "p" => {
            if styles.display.is_none() {
                styles.display = Some("block".to_string());
            }
            if styles.margin.is_none() {
                styles.margin = Some(BoxModel {
                    top: "16px".to_string(),
                    right: "0px".to_string(),
                    bottom: "16px".to_string(),
                    left: "0px".to_string(),
                });
            }
        }
        "div" | "section" | "article" | "header" | "footer" | "main" | "nav" | "aside" | "form"
        | "figure" | "figcaption" | "details" | "summary" | "dialog" | "address" | "fieldset"
        | "legend" => {
            if styles.display.is_none() {
                styles.display = Some("block".to_string());
            }
        }
        "blockquote" => {
            if styles.display.is_none() {
                styles.display = Some("block".to_string());
            }
            if styles.margin.is_none() {
                styles.margin = Some(BoxModel {
                    top: "16px".to_string(),
                    right: "40px".to_string(),
                    bottom: "16px".to_string(),
                    left: "40px".to_string(),
                });
            }
        }
        "pre" => {
            if styles.display.is_none() {
                styles.display = Some("block".to_string());
            }
            if styles.font_family.is_none() {
                styles.font_family = Some("monospace".to_string());
            }
            if styles.white_space.is_none() {
                styles.white_space = Some("pre".to_string());
            }
            if styles.background_color.is_none() {
                styles.background_color = Some("#f4f4f4".to_string());
            }
            if styles.padding.is_none() {
                styles.padding = Some(BoxModel {
                    top: "16px".to_string(),
                    right: "16px".to_string(),
                    bottom: "16px".to_string(),
                    left: "16px".to_string(),
                });
            }
            if styles.margin.is_none() {
                styles.margin = Some(BoxModel {
                    top: "16px".to_string(),
                    right: "0px".to_string(),
                    bottom: "16px".to_string(),
                    left: "0px".to_string(),
                });
            }
            if styles.overflow.is_none() {
                styles.overflow = Some("hidden".to_string());
            }
            if styles.border_radius.is_none() {
                styles.border_radius = Some("4px".to_string());
            }
        }
        "code" => {
            if styles.font_family.is_none() {
                styles.font_family = Some("monospace".to_string());
            }
            // Inline code gets subtle background (not inside <pre> — parent handles that)
            if styles.background_color.is_none() {
                styles.background_color = Some("#f0f0f0".to_string());
            }
            if styles.padding.is_none() {
                styles.padding = Some(BoxModel {
                    top: "2px".to_string(),
                    right: "4px".to_string(),
                    bottom: "2px".to_string(),
                    left: "4px".to_string(),
                });
            }
            if styles.border_radius.is_none() {
                styles.border_radius = Some("3px".to_string());
            }
        }
        "hr" => {
            if styles.display.is_none() {
                styles.display = Some("block".to_string());
            }
            if styles.margin.is_none() {
                styles.margin = Some(BoxModel {
                    top: "8px".to_string(),
                    right: "0px".to_string(),
                    bottom: "8px".to_string(),
                    left: "0px".to_string(),
                });
            }
            if styles.border.is_none() {
                styles.border = Some(BorderStyles {
                    width: "1px".to_string(),
                    style: "solid".to_string(),
                    color: "gray".to_string(),
                });
            }
        }
        "ul" | "ol" => {
            if styles.display.is_none() {
                styles.display = Some("block".to_string());
            }
            if styles.margin.is_none() {
                styles.margin = Some(BoxModel {
                    top: "16px".to_string(),
                    right: "0px".to_string(),
                    bottom: "16px".to_string(),
                    left: "0px".to_string(),
                });
            }
            if styles.padding.is_none() {
                styles.padding = Some(BoxModel {
                    top: "0px".to_string(),
                    right: "0px".to_string(),
                    bottom: "0px".to_string(),
                    left: "40px".to_string(),
                });
            }
        }
        "li" => {
            if styles.display.is_none() {
                styles.display = Some("list-item".to_string());
            }
            if styles.list_style_type.is_none() {
                styles.list_style_type = Some("disc".to_string());
            }
        }
        "a" => {
            if styles.color.is_none() {
                styles.color = Some("#0051C3".to_string());
            }
            if styles.text_decoration.is_none() {
                styles.text_decoration = Some("underline".to_string());
            }
        }
        "strong" | "b" => {
            if styles.font_weight.is_none() {
                styles.font_weight = Some("bold".to_string());
            }
        }
        "em" | "i" => {
            if styles.font_style.is_none() {
                styles.font_style = Some("italic".to_string());
            }
        }
        "span" | "label" => {
            if styles.display.is_none() {
                styles.display = Some("inline".to_string());
            }
        }
        "img" | "input" | "textarea" | "select" => {
            if styles.display.is_none() {
                styles.display = Some("inline-block".to_string());
            }
        }
        "br" => {
            if styles.display.is_none() {
                styles.display = Some("inline".to_string());
            }
        }
        "table" => {
            if styles.display.is_none() {
                styles.display = Some("table".to_string());
            }
            if styles.margin.is_none() {
                styles.margin = Some(BoxModel {
                    top: "16px".to_string(),
                    right: "0px".to_string(),
                    bottom: "16px".to_string(),
                    left: "0px".to_string(),
                });
            }
            if styles.border.is_none() {
                styles.border = Some(BorderStyles {
                    width: "1px".to_string(),
                    style: "solid".to_string(),
                    color: "gray".to_string(),
                });
            }
        }
        "tr" => {
            if styles.display.is_none() {
                styles.display = Some("table-row".to_string());
            }
        }
        "td" => {
            if styles.display.is_none() {
                styles.display = Some("table-cell".to_string());
            }
            if styles.padding.is_none() {
                styles.padding = Some(BoxModel {
                    top: "4px".to_string(),
                    right: "8px".to_string(),
                    bottom: "4px".to_string(),
                    left: "8px".to_string(),
                });
            }
        }
        "th" => {
            if styles.display.is_none() {
                styles.display = Some("table-cell".to_string());
            }
            if styles.font_weight.is_none() {
                styles.font_weight = Some("bold".to_string());
            }
            if styles.padding.is_none() {
                styles.padding = Some(BoxModel {
                    top: "4px".to_string(),
                    right: "8px".to_string(),
                    bottom: "4px".to_string(),
                    left: "8px".to_string(),
                });
            }
        }
        "button" => {
            if styles.display.is_none() {
                styles.display = Some("inline-block".to_string());
            }
            if styles.padding.is_none() {
                styles.padding = Some(BoxModel {
                    top: "4px".to_string(),
                    right: "8px".to_string(),
                    bottom: "4px".to_string(),
                    left: "8px".to_string(),
                });
            }
        }
        "progress" | "meter" => {
            // Replaced elements: render as visible blocks with their specified dimensions
            if styles.display.is_none() {
                styles.display = Some("block".to_string());
            }
        }
        _ => {}
    }

    // Quirks mode behavioral differences
    if doc_mode == DocumentMode::Quirks {
        // In quirks mode, table cells don't inherit font-size from ancestors
        // (they use the initial value of medium/16px instead)
        if matches!(tag, "td" | "th") {
            if styles.font_size.is_none() {
                styles.font_size = Some("16px".to_string());
            }
        }

        // In quirks mode, the default box model is border-box for certain form elements
        // This doesn't directly apply as a CSS property since taffy only supports border-box,
        // but we note it for width/height calculation: in quirks mode, width/height
        // includes padding+border (IE box model) so we DON'T inflate by padding+border
        // in the layout engine. We signal this via the "other" map.
        if matches!(
            tag,
            "input" | "textarea" | "select" | "button" | "table" | "img"
        ) {
            styles
                .other
                .entry("box-sizing".to_string())
                .or_insert_with(|| "border-box".to_string());
        }

        // In quirks mode, body height fills the viewport by default
        if tag == "body" {
            if styles.min_height.is_none() {
                styles.min_height = Some("100%".to_string());
            }
        }
    }
}

/// Tags that should be skipped during layout (metadata/invisible/non-renderable).
/// Includes elements we can't render whose subtrees would be wasted CSS work.
const SKIP_TAGS: &[&str] = &[
    // Metadata / invisible
    "script", "style", "link", "meta", "head", "title", "noscript", "template",
    // Embedded content we don't render (svg and audio handled specially below)
    "canvas", "video", "source", "track", "embed", "object", "param", "iframe",
    // Form internals not individually visible
    "datalist", // Table column styling (not visual elements themselves)
    "colgroup", "col", // Image maps
    "map", "area",
];

/// Compute a styled element tree from raw HTML (new pipeline).
///
/// This is the new bridge function for the Avalonia native rendering pipeline:
/// 1. Parses HTML with scraper
/// 2. Extracts `<style>` blocks and feeds them to CssProcessor
/// 3. Walks the DOM, computes CSS per element, builds LayoutElement tree
/// 4. Converts LayoutElement tree → StyledElement tree (NO taffy, NO positions)
///
/// The resulting StyledTreeResult is serialized to JSON and sent to C#,
/// where Avalonia handles layout and rendering using native controls.
pub fn compute_styled_tree(
    html: &str,
    viewport_w: f32,
    viewport_h: f32,
) -> Result<StyledTreeResult> {
    compute_styled_tree_with_css(html, viewport_w, viewport_h, &[])
}

/// Compute a styled element tree from raw HTML with external CSS stylesheets.
pub fn compute_styled_tree_with_css(
    html: &str,
    viewport_w: f32,
    viewport_h: f32,
    external_css: &[String],
) -> Result<StyledTreeResult> {
    let total_start = Instant::now();

    let parse_start = Instant::now();
    let document = Html::parse_document(html);
    eprintln!(
        "[TIMING] HTML parse: {}ms ({} bytes)",
        parse_start.elapsed().as_millis(),
        html.len()
    );

    // Detect document compatibility mode from DOCTYPE
    let doc_mode = detect_document_mode(&document);
    if doc_mode != DocumentMode::Standards {
        eprintln!(
            "[QUIRKS] Document mode: {:?} (may affect box model and CSS defaults)",
            doc_mode
        );
    }

    // Step 0: Extract html element's classes for CSS custom property scoping.
    // Selectors like `html.skin-theme-clientpref-night` define dark-mode CSS variables
    // that should only be stored when the <html> element actually has that class.
    let html_classes: Vec<String> = {
        let html_sel = Selector::parse("html").unwrap();
        if let Some(html_el) = document.select(&html_sel).next() {
            html_el
                .value()
                .attr("class")
                .map(|c| c.split_whitespace().map(|s| s.to_string()).collect())
                .unwrap_or_default()
        } else {
            Vec::new()
        }
    };
    eprintln!(
        "[DIAG] html classes: {:?}",
        &html_classes[..html_classes.len().min(5)]
    );

    // Step 1: Parse external stylesheets FIRST (lower source-order precedence)
    let ext_css_start = Instant::now();
    let mut css_processor = CssProcessor::new_with_viewport_and_height(viewport_w, viewport_h);
    css_processor.set_html_classes(html_classes);
    for css_text in external_css {
        if !css_text.trim().is_empty() {
            if let Err(e) = css_processor.parse(css_text) {
                eprintln!("[styled_tree] Failed to parse external stylesheet: {}", e);
            }
        }
    }
    eprintln!(
        "[TIMING] External CSS parse: {}ms ({} stylesheets)",
        ext_css_start.elapsed().as_millis(),
        external_css.len()
    );

    // Step 1b: Then parse <style> blocks (higher source-order precedence)
    // CSP: Check if inline styles are allowed before parsing <style> blocks
    let inline_css_start = Instant::now();
    let style_selector = Selector::parse("style").unwrap();
    let mut style_block_count = 0u32;
    let csp_allows_inline = thalora_browser_apis::csp::csp_allows_inline_style();
    for style_el in document.select(&style_selector) {
        let css_text: String = style_el.text().collect();
        if !css_text.trim().is_empty() {
            if !csp_allows_inline {
                eprintln!(
                    "🔒 CSP: Inline <style> block blocked by style-src (missing 'unsafe-inline')"
                );
                continue;
            }
            style_block_count += 1;
            if let Err(e) = css_processor.parse(&css_text) {
                eprintln!("[styled_tree] Failed to parse <style> block: {}", e);
            }
        }
    }
    eprintln!(
        "[TIMING] Inline CSS parse: {}ms ({} style blocks)",
        inline_css_start.elapsed().as_millis(),
        style_block_count
    );

    // Step 1c: Pre-compile all selectors for fast matching
    css_processor.compile_selectors();

    // Step 2: Walk the DOM tree and build StyledElement tree directly.
    // This preserves inline element structure (<a>, <strong>, <em>, <code>, <span>)
    // so C# can properly render links, bold, italic, inline code, etc.
    let walk_start = Instant::now();
    let root_node = document.root_element();
    let mut id_counter: u32 = 0;
    let mut element_selectors: HashMap<String, String> = HashMap::new();
    let mut counter_state = CounterState::default();
    let root = build_styled_element_from_dom(
        &root_node,
        &css_processor,
        &mut id_counter,
        viewport_w,
        None,
        &mut element_selectors,
        0,
        &mut counter_state,
        doc_mode,
    );
    eprintln!(
        "[TIMING] DOM tree walk (build_styled_element_from_dom): {}ms ({} elements)",
        walk_start.elapsed().as_millis(),
        id_counter
    );

    // Diagnostic: log html/body background colors and element count cap
    if id_counter >= MAX_ELEMENT_COUNT {
        eprintln!(
            "[DIAG] MAX_ELEMENT_COUNT ({}) reached — page tree was truncated!",
            MAX_ELEMENT_COUNT
        );
    }
    eprintln!(
        "[DIAG] html background_color={:?}",
        root.styles.background_color
    );
    if let Some(body) = root.children.iter().find(|c| c.tag == "body") {
        eprintln!(
            "[DIAG] body background_color={:?}",
            body.styles.background_color
        );
    }

    let selectors = if element_selectors.is_empty() {
        None
    } else {
        Some(element_selectors)
    };

    eprintln!(
        "[TIMING] Total compute_styled_tree_with_css: {}ms",
        total_start.elapsed().as_millis()
    );

    Ok(StyledTreeResult {
        root,
        viewport_width: viewport_w,
        viewport_height: viewport_h,
        element_selectors: selectors,
    })
}

/// Maximum number of elements to process before stopping.
/// Prevents extremely large pages from taking too long.
/// With the indexed rule lookup (compile_selectors + RuleIndex), CSS matching is
/// O(relevant_rules) per element, so 5000 elements is safe for performance.
const MAX_ELEMENT_COUNT: u32 = 5000;

/// Maximum recursion depth for DOM tree traversal.
/// Prevents stack overflow on deeply nested pages (Wikipedia/Wiktionary can nest 200+ levels).
/// Each recursion frame uses 2-4KB of stack; at 100 depth that's ~400KB of the 8MB stack.
const MAX_RECURSION_DEPTH: u32 = 100;

/// Tags that can have meaningful :hover styles (links, buttons, form elements).
/// Skipping hover computation for all other elements saves significant time.
const HOVER_INTERACTIVE_TAGS: &[&str] = &[
    "a", "button", "input", "select", "textarea", "summary", "details", "label",
];

/// Build a StyledElement tree directly from the DOM, preserving inline element structure.
///
/// Unlike `build_layout_tree_from_dom` (which flattens inline elements into text runs),
/// this function keeps every element as a proper node in the tree. This allows the C#
/// ControlTreeBuilder to style `<a>`, `<strong>`, `<em>`, `<code>`, `<span>` etc. individually.
fn build_styled_element_from_dom(
    element_ref: &ElementRef,
    css_processor: &CssProcessor,
    id_counter: &mut u32,
    viewport_w: f32,
    parent_styles: Option<&ComputedStyles>,
    element_selectors: &mut HashMap<String, String>,
    depth: u32,
    counter_state: &mut CounterState,
    doc_mode: DocumentMode,
) -> StyledElement {
    let el = element_ref.value();
    let tag = el.name().to_lowercase();

    // Compute CSS styles
    let mut styles = css_processor.compute_style_for_element(element_ref);

    // Handle inline style attribute — use direct parser (no CssProcessor overhead)
    let elem_id = format!("e{}", *id_counter);
    *id_counter += 1;

    if let Some(inline_style) = el.attr("style") {
        let inline_styles = CssProcessor::parse_inline_style_direct(inline_style);
        merge_styles(&mut styles, &inline_styles);
    }

    // Inherit properties from parent
    if let Some(parent) = parent_styles {
        inherit_properties(&mut styles, parent);
    }

    // Apply UA defaults
    apply_ua_defaults(&tag, &mut styles, doc_mode);

    // Default display for elements if not set
    if styles.display.is_none() {
        styles.display = Some(
            if is_block_element(&tag) {
                "block"
            } else {
                "inline"
            }
            .to_string(),
        );
    }

    // Early exit for display:none — skip entire subtree (children, hover, selectors).
    // This is a major optimization: avoids computing CSS for all descendants of hidden elements.
    if styles.display.as_deref() == Some("none") {
        let resolved = computed_to_resolved(&styles);
        return StyledElement {
            id: elem_id,
            tag,
            text_content: None,
            img_src: None,
            img_alt: None,
            link_href: None,
            attributes: None,
            styles: resolved,
            hover_styles: None,
            children: Vec::new(),
        };
    }

    // Depth limit — stop recursion to prevent stack overflow on deeply nested DOMs.
    // We still compute this element's own styles (above), but don't recurse into children.
    if depth >= MAX_RECURSION_DEPTH {
        let resolved = computed_to_resolved(&styles);
        return StyledElement {
            id: elem_id,
            tag,
            text_content: None,
            img_src: None,
            img_alt: None,
            link_href: None,
            attributes: None,
            styles: resolved,
            hover_styles: None,
            children: Vec::new(),
        };
    }

    // Inline SVG: extract dimensions and recurse into SVG children
    // so that querySelector('svg path') etc. work for DOM access.
    if tag == "svg" {
        let (w, h) = extract_svg_dimensions(el);
        if styles.width.is_none() {
            styles.width = Some(format!("{}px", w));
        }
        if styles.height.is_none() {
            styles.height = Some(format!("{}px", h));
        }
        // Fall through to normal child processing below
    }

    // Audio element: create a placeholder with source URL extracted from <source> children.
    // C# renders this as a minimal audio widget placeholder.
    if tag == "audio" {
        let mut audio_src = el.attr("src").map(|s| s.to_string());
        // If no src on <audio>, look for <source> children
        if audio_src.is_none() {
            for child_node in element_ref.children() {
                if let Some(child_el_ref) = ElementRef::wrap(child_node) {
                    if child_el_ref.value().name().eq_ignore_ascii_case("source") {
                        if let Some(src) = child_el_ref.value().attr("src") {
                            audio_src = Some(src.to_string());
                            break;
                        }
                    }
                }
            }
        }
        let mut attrs = HashMap::new();
        if let Some(src) = audio_src {
            attrs.insert("src".to_string(), src);
        }
        if el.attr("controls").is_some() {
            attrs.insert("controls".to_string(), "true".to_string());
        }
        let resolved = computed_to_resolved(&styles);
        return StyledElement {
            id: elem_id,
            tag,
            text_content: None,
            img_src: None,
            img_alt: None,
            link_href: None,
            attributes: if attrs.is_empty() { None } else { Some(attrs) },
            styles: resolved,
            hover_styles: None,
            children: Vec::new(),
        };
    }

    // Compute hover styles only for interactive elements (links, buttons, etc.)
    // This saves significant time on large pages — hover matching iterates all CSS rules.
    let hover_styles = if HOVER_INTERACTIVE_TAGS.contains(&tag.as_str())
        || el
            .attr("class")
            .map_or(false, |c| c.contains("btn") || c.contains("button"))
    {
        let hover_computed = css_processor.compute_hover_style_for_element(element_ref);
        let hover_resolved = computed_to_resolved(&hover_computed);
        if has_any_hover_property(&hover_resolved) {
            Some(hover_resolved)
        } else {
            None
        }
    } else {
        None
    };

    // Build a unique CSS selector for this element (for JS event dispatch)
    let css_selector = build_element_selector(element_ref);
    element_selectors.insert(elem_id.clone(), css_selector);

    // Extract HTML attributes
    let link_href = if tag == "a" {
        el.attr("href").map(|h| h.to_string())
    } else {
        None
    };

    let img_src = if tag == "img" {
        el.attr("src").and_then(|s| {
            // CSP: Check img-src before allowing the image URL
            if thalora_browser_apis::csp::csp_allows_image(s) {
                Some(s.to_string())
            } else {
                eprintln!("🔒 CSP: Image blocked by img-src: {}", s);
                None
            }
        })
    } else {
        None
    };

    let img_alt = if tag == "img" {
        el.attr("alt").map(|a| a.to_string())
    } else {
        None
    };

    // Extract HTML attributes for form elements and semantic elements.
    // These are needed by C# to render <input>, <button>, <select>, <textarea>, etc.
    let attributes = {
        const FORM_TAGS: &[&str] = &[
            "input", "button", "select", "textarea", "option", "label", "form", "fieldset",
        ];
        const ATTR_NAMES: &[&str] = &[
            "type",
            "placeholder",
            "value",
            "name",
            "checked",
            "selected",
            "disabled",
            "readonly",
            "required",
            "multiple",
            "size",
            "maxlength",
            "min",
            "max",
            "step",
            "pattern",
            "autocomplete",
            "autofocus",
            "for",
            "action",
            "method",
            "enctype",
            "role",
            "aria-label",
            "aria-hidden",
            "aria-expanded",
            "title",
            "tabindex",
        ];
        if FORM_TAGS.contains(&tag.as_str()) {
            let mut attrs = HashMap::new();
            for &name in ATTR_NAMES {
                if let Some(val) = el.attr(name) {
                    attrs.insert(name.to_string(), val.to_string());
                }
            }
            // For boolean attributes (checked, selected, disabled, etc.),
            // the attribute's presence means true
            for &name in &[
                "checked",
                "selected",
                "disabled",
                "readonly",
                "required",
                "multiple",
                "autofocus",
            ] {
                if el.attr(name).is_some() && !attrs.contains_key(name) {
                    attrs.insert(name.to_string(), "true".to_string());
                }
            }
            if attrs.is_empty() { None } else { Some(attrs) }
        } else {
            None
        }
    };

    // Capture width/height attributes for img if not set by CSS
    if tag == "img" {
        if styles.width.is_none() {
            if let Some(w) = el.attr("width") {
                styles.width = Some(format!("{}px", w));
            }
        }
        if styles.height.is_none() {
            if let Some(h) = el.attr("height") {
                styles.height = Some(format!("{}px", h));
            }
        }
    }

    // CSS Counters: apply counter-reset and counter-increment
    let counter_reset_value = styles.counter_reset.clone();
    if let Some(ref cr) = counter_reset_value {
        counter_state.apply_reset(cr);
    }
    if let Some(ref ci) = styles.counter_increment {
        counter_state.apply_increment(ci);
    }

    // White-space mode for text collapsing
    let ws = styles
        .white_space
        .clone()
        .unwrap_or_else(|| "normal".to_string());

    // Build children — preserve ALL elements (inline and block) as children
    let mut children = Vec::new();

    for child_node in element_ref.children() {
        // Stop processing if we've hit the element cap
        if *id_counter >= MAX_ELEMENT_COUNT {
            if *id_counter == MAX_ELEMENT_COUNT {
                eprintln!(
                    "[DIAG] Hit MAX_ELEMENT_COUNT={} during tree walk, truncating remaining children",
                    MAX_ELEMENT_COUNT
                );
            }
            break;
        }

        match child_node.value() {
            Node::Text(text) => {
                let raw_text = text.text.as_ref();
                // Collapse whitespace for normal/nowrap modes
                // IMPORTANT: Preserve leading/trailing single space — HTML whitespace
                // between inline elements must collapse to a single space, not disappear.
                let text_str = if ws == "normal" || ws == "nowrap" {
                    let has_leading_ws = raw_text.starts_with(char::is_whitespace);
                    let has_trailing_ws = raw_text.ends_with(char::is_whitespace);
                    let collapsed = raw_text.split_whitespace().collect::<Vec<_>>().join(" ");
                    if collapsed.is_empty() {
                        // Whitespace-only text nodes: per CSS 2.1 §9.2.1.1, if
                        // the whitespace-only text is a child of a block container
                        // and all its element siblings are block-level, suppress it
                        // (inter-block whitespace). Otherwise collapse to single space.
                        let all_siblings_block = element_ref.children().all(|sibling| {
                            match sibling.value() {
                                Node::Text(t) => {
                                    // Other whitespace-only text nodes don't prevent suppression
                                    t.text.as_ref().trim().is_empty()
                                }
                                Node::Element(el) => {
                                    let sib_tag = el.name.local.as_ref();
                                    matches!(
                                        sib_tag,
                                        "div"
                                            | "p"
                                            | "h1"
                                            | "h2"
                                            | "h3"
                                            | "h4"
                                            | "h5"
                                            | "h6"
                                            | "ul"
                                            | "ol"
                                            | "li"
                                            | "dl"
                                            | "dt"
                                            | "dd"
                                            | "article"
                                            | "aside"
                                            | "footer"
                                            | "header"
                                            | "main"
                                            | "nav"
                                            | "section"
                                            | "blockquote"
                                            | "pre"
                                            | "figure"
                                            | "figcaption"
                                            | "details"
                                            | "summary"
                                            | "hr"
                                            | "table"
                                            | "form"
                                            | "fieldset"
                                            | "address"
                                    )
                                }
                                _ => true,
                            }
                        });
                        if all_siblings_block {
                            continue; // Suppress inter-block whitespace
                        }
                        if has_leading_ws || has_trailing_ws {
                            " ".to_string()
                        } else {
                            continue;
                        }
                    } else {
                        let mut result = String::new();
                        if has_leading_ws {
                            result.push(' ');
                        }
                        result.push_str(&collapsed);
                        if has_trailing_ws {
                            result.push(' ');
                        }
                        result
                    }
                } else {
                    raw_text.to_string()
                };
                if text_str.is_empty() {
                    continue;
                }

                // Create a #text StyledElement
                let text_id = format!("t{}", *id_counter);
                *id_counter += 1;

                // Inherit text-related styles from parent
                let text_resolved = ResolvedStyles {
                    display: Some("inline".to_string()),
                    font_size: styles.font_size.clone(),
                    font_family: styles.font_family.clone(),
                    font_weight: styles.font_weight.clone(),
                    font_style: styles.font_style.clone(),
                    line_height: styles.line_height.clone(),
                    color: styles.color.clone(),
                    white_space: styles.white_space.clone(),
                    text_decoration: styles.text_decoration.clone(),
                    letter_spacing: styles.letter_spacing.clone(),
                    ..ResolvedStyles::default()
                };

                children.push(StyledElement {
                    id: text_id,
                    tag: "#text".to_string(),
                    text_content: Some(text_str),
                    img_src: None,
                    img_alt: None,
                    link_href: None,
                    attributes: None,
                    styles: text_resolved,
                    hover_styles: None,
                    children: Vec::new(),
                });
            }
            Node::Element(_) => {
                if let Some(child_el_ref) = ElementRef::wrap(child_node) {
                    let child_tag = child_el_ref.value().name().to_lowercase();

                    // Skip metadata/invisible tags
                    if SKIP_TAGS.contains(&child_tag.as_str()) {
                        continue;
                    }

                    // Recursively build child element
                    let child_styled = build_styled_element_from_dom(
                        &child_el_ref,
                        css_processor,
                        id_counter,
                        viewport_w,
                        Some(&styles),
                        element_selectors,
                        depth + 1,
                        counter_state,
                        doc_mode,
                    );

                    // display:none is now handled inside build_styled_element_from_dom
                    // (early exit before recursing into children), but we still filter
                    // the returned element to avoid including it in the parent's children.
                    if child_styled.styles.display.as_deref() == Some("none") {
                        continue;
                    }
                    if child_styled.styles.visibility.as_deref() == Some("hidden") {
                        continue;
                    }

                    children.push(child_styled);
                }
            }
            _ => {}
        }
    }

    // Generate ::before pseudo-element if content is set
    if let Some(ref content_val) = styles.content {
        if let Some(text) = resolve_css_content(content_val, counter_state, Some(el)) {
            let before_id = format!("pb{}", *id_counter);
            *id_counter += 1;
            let before_styles = ResolvedStyles {
                display: Some("inline".to_string()),
                font_size: styles.font_size.clone(),
                font_family: styles.font_family.clone(),
                font_weight: styles.font_weight.clone(),
                color: styles.color.clone(),
                ..ResolvedStyles::default()
            };
            children.insert(
                0,
                StyledElement {
                    id: before_id,
                    tag: "::before".to_string(),
                    text_content: Some(text),
                    img_src: None,
                    img_alt: None,
                    link_href: None,
                    attributes: None,
                    styles: before_styles,
                    hover_styles: None,
                    children: Vec::new(),
                },
            );
        }
    }

    // Pop counter-reset scopes when leaving this element
    if let Some(ref cr) = counter_reset_value {
        counter_state.pop_reset(cr);
    }

    // Convert ComputedStyles → ResolvedStyles
    let resolved = computed_to_resolved(&styles);

    StyledElement {
        id: elem_id,
        tag,
        text_content: None, // Block/inline elements carry text via #text children
        img_src,
        img_alt,
        link_href,
        attributes,
        styles: resolved,
        hover_styles,
        children,
    }
}

/// Convert ComputedStyles to ResolvedStyles (shared conversion logic)
fn computed_to_resolved(styles: &ComputedStyles) -> ResolvedStyles {
    ResolvedStyles {
        display: styles.display.clone(),
        position: styles.position.clone(),
        flex_direction: styles.flex_direction.clone(),
        flex_wrap: styles.flex_wrap.clone(),
        justify_content: styles.justify_content.clone(),
        align_items: styles.align_items.clone(),
        align_self: styles.align_self.clone(),
        gap: styles.gap.clone(),
        flex_grow: styles.flex_grow.clone(),
        flex_shrink: styles.flex_shrink.clone(),
        flex_basis: styles.flex_basis.clone(),

        width: styles.width.clone(),
        height: styles.height.clone(),
        min_width: styles.min_width.clone(),
        min_height: styles.min_height.clone(),
        max_width: styles.max_width.clone(),
        max_height: styles.max_height.clone(),

        margin: styles.margin.as_ref().map(|m| StyleBoxSides {
            top: m.top.clone(),
            right: m.right.clone(),
            bottom: m.bottom.clone(),
            left: m.left.clone(),
        }),
        padding: styles.padding.as_ref().map(|p| StyleBoxSides {
            top: p.top.clone(),
            right: p.right.clone(),
            bottom: p.bottom.clone(),
            left: p.left.clone(),
        }),

        font_size: styles.font_size.clone(),
        font_family: styles.font_family.clone(),
        font_weight: styles.font_weight.clone(),
        font_style: styles.font_style.clone(),
        line_height: styles.line_height.clone(),
        text_align: styles.text_align.clone(),
        text_decoration: styles.text_decoration.clone(),
        text_transform: styles.text_transform.clone(),
        white_space: styles.white_space.clone(),
        letter_spacing: styles.letter_spacing.clone(),
        word_spacing: styles.word_spacing.clone(),

        color: styles.color.clone(),
        background_color: styles.background_color.clone(),

        // Merge border shorthand + per-side overrides into StyleBoxSides.
        // Shorthand `border` sets all sides; `border-top/right/bottom/left` override individually.
        border_width: {
            let has_shorthand = styles.border.is_some();
            let has_sides = styles.border_top.is_some()
                || styles.border_right.is_some()
                || styles.border_bottom.is_some()
                || styles.border_left.is_some();
            if has_shorthand || has_sides {
                let default_w = styles
                    .border
                    .as_ref()
                    .map(|b| b.width.clone())
                    .unwrap_or_default();
                Some(StyleBoxSides {
                    top: styles
                        .border_top
                        .as_ref()
                        .map(|b| b.width.clone())
                        .unwrap_or_else(|| default_w.clone()),
                    right: styles
                        .border_right
                        .as_ref()
                        .map(|b| b.width.clone())
                        .unwrap_or_else(|| default_w.clone()),
                    bottom: styles
                        .border_bottom
                        .as_ref()
                        .map(|b| b.width.clone())
                        .unwrap_or_else(|| default_w.clone()),
                    left: styles
                        .border_left
                        .as_ref()
                        .map(|b| b.width.clone())
                        .unwrap_or(default_w),
                })
            } else {
                None
            }
        },
        // For border color: use the first non-empty color found (shorthand or any side).
        // CSS allows different colors per side, but our C# model uses a single BorderBrush,
        // so we pick the most relevant one.
        border_color: {
            let colors: Vec<&str> = [
                styles.border.as_ref().map(|b| b.color.as_str()),
                styles.border_top.as_ref().map(|b| b.color.as_str()),
                styles.border_right.as_ref().map(|b| b.color.as_str()),
                styles.border_bottom.as_ref().map(|b| b.color.as_str()),
                styles.border_left.as_ref().map(|b| b.color.as_str()),
            ]
            .into_iter()
            .flatten()
            .filter(|c| !c.is_empty())
            .collect();
            colors.first().map(|c| c.to_string())
        },
        // For border style: same approach — pick the first non-empty style.
        border_style: {
            let border_styles: Vec<&str> = [
                styles.border.as_ref().map(|b| b.style.as_str()),
                styles.border_top.as_ref().map(|b| b.style.as_str()),
                styles.border_right.as_ref().map(|b| b.style.as_str()),
                styles.border_bottom.as_ref().map(|b| b.style.as_str()),
                styles.border_left.as_ref().map(|b| b.style.as_str()),
            ]
            .into_iter()
            .flatten()
            .filter(|s| !s.is_empty())
            .collect();
            border_styles.first().map(|s| s.to_string())
        },
        border_radius: styles.border_radius.clone(),

        opacity: styles.opacity,
        overflow: styles.overflow.clone(),
        visibility: styles.visibility.clone(),
        z_index: styles.z_index,
        list_style_type: styles.list_style_type.clone(),
        cursor: styles.cursor.clone(),
        grid_template_columns: styles.grid_template_columns.clone(),
        grid_template_rows: styles.grid_template_rows.clone(),
        grid_template_areas: styles.grid_template_areas.clone(),
        grid_area: styles.grid_area.clone(),
    }
}

/// Recursively convert a LayoutElement (internal) to a StyledElement (output).
fn convert_to_styled_element(element: &LayoutElement) -> StyledElement {
    let styles = &element.styles;

    // Extract content from __xxx keys in styles.other
    let text_content = styles.other.get("__text_content").cloned();
    let img_src = if element.tag == "img" {
        styles.other.get("__img_src").cloned()
    } else {
        None
    };
    let img_alt = if element.tag == "img" {
        styles.other.get("__img_alt").cloned()
    } else {
        None
    };
    let link_href = styles.other.get("__link_href").cloned();

    // Convert ComputedStyles → ResolvedStyles (reuse shared conversion)
    let resolved = computed_to_resolved(styles);

    // Recursively convert children
    let children: Vec<StyledElement> = element
        .children
        .iter()
        .map(|child| convert_to_styled_element(child))
        .collect();

    StyledElement {
        id: element.id.clone(),
        tag: element.tag.clone(),
        text_content,
        img_src,
        img_alt,
        link_href,
        attributes: None, // Old pipeline doesn't extract attributes
        styles: resolved,
        hover_styles: None, // Old pipeline doesn't compute hover styles
        children,
    }
}

/// Compute the full page layout from raw HTML (old pipeline, kept for compatibility).
///
/// This is the legacy bridge function:
/// 1. Parses HTML with scraper
/// 2. Extracts `<style>` blocks and feeds them to CssProcessor
/// 3. Walks the DOM, computes CSS per element, builds LayoutElement tree
/// 4. Runs LayoutEngine (taffy) to compute positions/sizes
/// 5. Copies visual properties into the resulting ElementLayout tree
pub fn compute_page_layout(html: &str, viewport_w: f32, viewport_h: f32) -> Result<LayoutResult> {
    compute_page_layout_with_css(html, viewport_w, viewport_h, &[])
}

/// Compute the full page layout from raw HTML with external CSS stylesheets.
///
/// External CSS is parsed first (lower specificity by source order), then
/// inline `<style>` blocks are parsed (higher source order, so they override).
pub fn compute_page_layout_with_css(
    html: &str,
    viewport_w: f32,
    viewport_h: f32,
    external_css: &[String],
) -> Result<LayoutResult> {
    let document = Html::parse_document(html);
    let doc_mode = detect_document_mode(&document);

    // Step 1: Parse external stylesheets FIRST (lower source-order precedence)
    let mut css_processor = CssProcessor::new_with_viewport_and_height(viewport_w, viewport_h);
    for css_text in external_css {
        if !css_text.trim().is_empty() {
            if let Err(e) = css_processor.parse(css_text) {
                eprintln!("[page_layout] Failed to parse external stylesheet: {}", e);
            }
        }
    }

    // Step 1b: Then parse <style> blocks (higher source-order precedence, overrides external)
    // CSP: Check if inline styles are allowed
    let style_selector = Selector::parse("style").unwrap();
    let csp_allows_inline_legacy = thalora_browser_apis::csp::csp_allows_inline_style();
    for style_el in document.select(&style_selector) {
        let css_text: String = style_el.text().collect();
        if !css_text.trim().is_empty() {
            if !csp_allows_inline_legacy {
                eprintln!(
                    "🔒 CSP: Inline <style> block blocked by style-src (missing 'unsafe-inline')"
                );
                continue;
            }
            // lightningcss may fail on malformed CSS — log and skip
            if let Err(e) = css_processor.parse(&css_text) {
                eprintln!("[page_layout] Failed to parse <style> block: {}", e);
            }
        }
    }

    // Step 2: Walk the DOM tree and build LayoutElement tree
    let root_node = document.root_element();
    let mut layout_tree = build_layout_tree_from_dom(
        &root_node,
        &css_processor,
        &mut 0,
        viewport_w,
        viewport_w as f64,
        None, // no parent styles for root
        doc_mode,
    );

    // Step 2.5: Ensure html and body span the full viewport (CSS spec behavior)
    // The root element should have min-height of 100% of viewport.
    // Body inherits this stretching so backgrounds cover the full viewport.
    let vh = format!("{}px", viewport_h);
    if layout_tree.tag == "html" {
        if layout_tree.styles.min_height.is_none() {
            layout_tree.styles.min_height = Some(vh.clone());
        }

        // Find body child and set its min-height too
        for child in &mut layout_tree.children {
            if child.tag == "body" {
                if child.styles.min_height.is_none() {
                    child.styles.min_height = Some(vh.clone());
                }
                break;
            }
        }
    }

    // Step 3: Run taffy layout
    let mut engine = LayoutEngine::with_viewport(viewport_w, viewport_h);
    let mut layout_result = engine
        .calculate_layout_from_elements(&layout_tree)
        .context("Failed to compute layout")?;

    // Step 4: Post-process — copy visual properties from the LayoutElement tree
    // into the resulting ElementLayout tree (text content, links, images, etc.)
    let visual_map = build_visual_map(&layout_tree);
    for element in &mut layout_result.elements {
        apply_visual_properties(element, &visual_map);
    }

    Ok(layout_result)
}

/// Inherit CSS properties from parent to child per CSS spec.
/// Only inheritable properties are copied, and only when the child doesn't define them.
fn inherit_properties(child: &mut ComputedStyles, parent: &ComputedStyles) {
    // Per CSS spec, these properties are inherited by default:
    if child.color.is_none() {
        child.color = parent.color.clone();
    }
    // NOTE: font_size is NOT inherited here. CSS font-size inheritance copies the
    // COMPUTED value (e.g., 28.8px), not the specified value (e.g., 1.8em).
    // Since we store font_size as a raw CSS string, inheriting it would cause
    // relative units (em, %) to compound at each level (1.8em * 1.8em * 16 = 51.84px
    // instead of 1.8em * 16 = 28.8px). The C# side handles font-size inheritance
    // correctly via the parentFontSize parameter chain in BuildControl/ParseFontSize.
    if child.font_family.is_none() {
        child.font_family = parent.font_family.clone();
    }
    if child.font_weight.is_none() {
        child.font_weight = parent.font_weight.clone();
    }
    if child.font_style.is_none() {
        child.font_style = parent.font_style.clone();
    }
    if child.line_height.is_none() {
        child.line_height = parent.line_height.clone();
    }
    if child.text_align.is_none() {
        child.text_align = parent.text_align.clone();
    }
    if child.white_space.is_none() {
        child.white_space = parent.white_space.clone();
    }
    if child.visibility.is_none() {
        child.visibility = parent.visibility.clone();
    }
    if child.text_decoration.is_none() {
        child.text_decoration = parent.text_decoration.clone();
    }
    if child.text_transform.is_none() {
        child.text_transform = parent.text_transform.clone();
    }
    if child.letter_spacing.is_none() {
        child.letter_spacing = parent.letter_spacing.clone();
    }
    if child.word_spacing.is_none() {
        child.word_spacing = parent.word_spacing.clone();
    }
    if child.cursor.is_none() {
        child.cursor = parent.cursor.clone();
    }
    if child.list_style_type.is_none() {
        child.list_style_type = parent.list_style_type.clone();
    }
}

/// Build a LayoutElement tree from the scraper DOM.
/// `available_width` tracks the estimated content width of the current container
/// for text wrapping height estimates.
/// `parent_styles` enables CSS property inheritance from parent to child.
fn build_layout_tree_from_dom(
    element_ref: &ElementRef,
    css_processor: &CssProcessor,
    id_counter: &mut u32,
    viewport_w: f32,
    available_width: f64,
    parent_styles: Option<&ComputedStyles>,
    doc_mode: DocumentMode,
) -> LayoutElement {
    let el = element_ref.value();
    let tag = el.name().to_lowercase();

    // Compute CSS styles using scraper's element-based selector matching
    let mut styles = css_processor.compute_style_for_element(element_ref);

    // Handle inline style attribute
    let elem_id = format!("e{}", *id_counter);
    *id_counter += 1;

    if let Some(inline_style) = el.attr("style") {
        let mut inline_processor = CssProcessor::new();
        if inline_processor
            .parse_inline_style(inline_style, &elem_id)
            .is_ok()
        {
            let inline_styles = inline_processor.compute_style(&format!("#{}", elem_id));
            merge_styles(&mut styles, &inline_styles);
        }
    }

    // Inherit properties from parent (Phase 4: CSS inheritance)
    if let Some(parent) = parent_styles {
        inherit_properties(&mut styles, parent);
    }

    // Apply UA defaults (only for properties not already set by CSS)
    apply_ua_defaults(&tag, &mut styles, doc_mode);

    // Capture HTML attributes that affect rendering
    if tag == "img" {
        if let Some(src) = el.attr("src") {
            // CSP: Check img-src before allowing the image URL
            if thalora_browser_apis::csp::csp_allows_image(src) {
                styles
                    .other
                    .insert("__img_src".to_string(), src.to_string());
            } else {
                eprintln!("🔒 CSP: Image blocked by img-src: {}", src);
            }
        }
        // Use alt text as fallback display content and store separately for the styled tree
        if let Some(alt) = el.attr("alt") {
            styles
                .other
                .insert("__text_content".to_string(), alt.to_string());
            styles
                .other
                .insert("__img_alt".to_string(), alt.to_string());
        }
        // Capture width/height attributes if not set by CSS
        if styles.width.is_none() {
            if let Some(w) = el.attr("width") {
                styles.width = Some(format!("{}px", w));
            }
        }
        if styles.height.is_none() {
            if let Some(h) = el.attr("height") {
                styles.height = Some(format!("{}px", h));
            }
        }
    }

    // Default display for block elements if not set
    if styles.display.is_none() {
        styles.display = Some(
            if is_block_element(&tag) {
                "block"
            } else {
                "inline"
            }
            .to_string(),
        );
    }

    // Compute available width for children based on this element's styles
    let vw = viewport_w as f64;
    let vh = viewport_w as f64 * 0.5625; // approximate viewport height
    let child_available_width = {
        let mut w = available_width;
        let mut has_explicit_width = false;
        // If this element has an explicit width, use that instead
        if let Some(ref width_str) = styles.width {
            let font_size = styles
                .font_size
                .as_ref()
                .and_then(|s| super::layout::resolve_css_length(s, 16.0))
                .unwrap_or(16.0);
            if width_str.ends_with('%') {
                if let Ok(pct) = width_str.trim_end_matches('%').parse::<f64>() {
                    w = available_width * pct / 100.0;
                }
                has_explicit_width = true;
            } else if let Some(px) = super::layout::resolve_css_length_vp(
                width_str,
                font_size,
                viewport_w,
                viewport_w * 9.0 / 16.0,
            ) {
                w = px;
                has_explicit_width = true;
            }
        }
        // Only subtract padding when width is auto (filling parent).
        // Explicit widths are content-box (taffy default) — they already
        // represent the content area, so padding is NOT subtracted.
        if !has_explicit_width {
            if let Some(ref padding) = styles.padding {
                let fs = styles
                    .font_size
                    .as_ref()
                    .and_then(|s| super::layout::resolve_css_length(s, 16.0))
                    .unwrap_or(16.0);
                let pl = super::layout::resolve_css_length_vp(
                    &padding.left,
                    fs,
                    viewport_w,
                    viewport_w * 9.0 / 16.0,
                )
                .unwrap_or(0.0);
                let pr = super::layout::resolve_css_length_vp(
                    &padding.right,
                    fs,
                    viewport_w,
                    viewport_w * 9.0 / 16.0,
                )
                .unwrap_or(0.0);
                w -= pl + pr;
            }
        }
        w.max(50.0)
    };

    // =========================================================================
    // Build children with inline formatting context support.
    //
    // Inline elements (text nodes, <a>, <span>, <strong>, <em>, <code>, etc.)
    // are collected into "inline runs". Consecutive inline runs are concatenated
    // into a single text block, measured together, and split into pre-split lines.
    // Block elements interrupt the inline flow and are processed separately.
    // =========================================================================

    // Phase 1: Classify children as inline or block segments.
    enum ChildSegment<'a> {
        /// Plain text content with optional link href (inherited from <a> parent)
        InlineText {
            text: String,
            link_href: Option<String>,
        },
        /// An inline element like <a>, <strong>, <em> — extract its text inline
        InlineElement {
            element_ref: ElementRef<'a>,
            tag: String,
        },
        /// A block-level element — process recursively
        BlockElement { element_ref: ElementRef<'a> },
    }

    let ws = styles.white_space.as_deref().unwrap_or("normal");
    let link_href_from_parent = if tag == "a" {
        el.attr("href").map(|h| h.to_string())
    } else {
        None
    };

    let mut segments: Vec<ChildSegment> = Vec::new();
    for child in element_ref.children() {
        match child.value() {
            Node::Element(_) => {
                if let Some(child_el_ref) = ElementRef::wrap(child) {
                    let child_tag = child_el_ref.value().name().to_lowercase();
                    if SKIP_TAGS.contains(&child_tag.as_str()) {
                        continue;
                    }
                    if is_inline_element(&child_tag) {
                        segments.push(ChildSegment::InlineElement {
                            element_ref: child_el_ref,
                            tag: child_tag,
                        });
                    } else {
                        segments.push(ChildSegment::BlockElement {
                            element_ref: child_el_ref,
                        });
                    }
                }
            }
            Node::Text(text) => {
                let raw_text = text.text.as_ref();
                if ws == "normal" && raw_text.trim().is_empty() {
                    continue;
                }
                let text_str: String = if ws == "normal" || ws == "nowrap" {
                    raw_text.split_whitespace().collect::<Vec<_>>().join(" ")
                } else {
                    raw_text.to_string()
                };
                if text_str.is_empty() {
                    continue;
                }
                segments.push(ChildSegment::InlineText {
                    text: text_str,
                    link_href: link_href_from_parent.clone(),
                });
            }
            _ => {}
        }
    }

    // Phase 2: Group consecutive inline segments and process.
    let mut children = Vec::new();

    // Helper: extract all text content from an inline element recursively
    fn extract_inline_text(el_ref: &ElementRef, ws: &str) -> String {
        let mut text = String::new();
        for child in el_ref.children() {
            match child.value() {
                scraper::Node::Text(t) => {
                    let raw = t.text.as_ref();
                    if ws == "normal" || ws == "nowrap" {
                        let collapsed = raw.split_whitespace().collect::<Vec<_>>().join(" ");
                        if !text.is_empty()
                            && !collapsed.is_empty()
                            && !text.ends_with(' ')
                            && !collapsed.starts_with(' ')
                        {
                            text.push(' ');
                        }
                        text.push_str(&collapsed);
                    } else {
                        text.push_str(raw);
                    }
                }
                scraper::Node::Element(_) => {
                    if let Some(child_el) = ElementRef::wrap(child) {
                        let inner = extract_inline_text(&child_el, ws);
                        if !text.is_empty()
                            && !inner.is_empty()
                            && !text.ends_with(' ')
                            && !inner.starts_with(' ')
                        {
                            text.push(' ');
                        }
                        text.push_str(&inner);
                    }
                }
                _ => {}
            }
        }
        text
    }

    /// Flush an accumulated inline run into a single text element.
    /// Uses measure_text() for total dimensions only — Avalonia handles all line wrapping.
    fn flush_inline_run(
        combined_text: &str,
        styles: &ComputedStyles,
        id_counter: &mut u32,
        child_available_width: f64,
        link_href: &Option<String>,
        children: &mut Vec<LayoutElement>,
    ) {
        if combined_text.trim().is_empty() {
            return;
        }

        let font_size = styles
            .font_size
            .as_ref()
            .and_then(|s| super::layout::resolve_css_length(s, 16.0))
            .unwrap_or(16.0);
        let line_height = styles
            .line_height
            .as_ref()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(1.4);
        let font_family = styles.font_family.as_deref().unwrap_or("sans-serif");
        let font_weight = styles.font_weight.as_deref();
        let container_w = child_available_width.max(50.0);

        // Measure total dimensions only — no line splitting
        let measurement = super::text_measure::measure_text(
            combined_text,
            font_family,
            font_size as f32,
            font_weight,
            Some(container_w as f32),
            line_height as f32,
        );

        // Apply 10% height buffer to account for metric differences between
        // cosmic_text (Rust) and Avalonia FormattedText (C#)
        let buffered_height = (measurement.height as f64) * 1.10;

        let text_id = format!("t{}", *id_counter);
        *id_counter += 1;

        let mut text_styles = ComputedStyles::default();
        text_styles.font_size = styles.font_size.clone();
        text_styles.font_family = styles.font_family.clone();
        text_styles.font_weight = styles.font_weight.clone();
        text_styles.color = styles.color.clone();
        text_styles.font_style = styles.font_style.clone();
        text_styles.line_height = styles.line_height.clone();
        text_styles.white_space = styles.white_space.clone();
        text_styles.min_height = Some(format!("{}px", buffered_height));
        text_styles.display = Some("block".to_string());
        text_styles
            .other
            .insert("__text_content".to_string(), combined_text.to_string());

        if let Some(href) = link_href {
            text_styles
                .other
                .insert("__link_href".to_string(), href.clone());
        }

        children.push(LayoutElement {
            id: text_id,
            tag: "#text".to_string(),
            styles: text_styles,
            children: Vec::new(),
        });
    }

    let mut inline_buffer = String::new();
    let mut i = 0;
    while i < segments.len() {
        match &segments[i] {
            ChildSegment::InlineText { text, link_href: _ } => {
                // Accumulate inline text
                if !inline_buffer.is_empty()
                    && !inline_buffer.ends_with(' ')
                    && !text.starts_with(' ')
                {
                    inline_buffer.push(' ');
                }
                inline_buffer.push_str(text);
                i += 1;
            }
            ChildSegment::InlineElement {
                element_ref: inline_el,
                tag: _inline_tag,
            } => {
                // Extract text from the inline element and append to the buffer
                let inline_text = extract_inline_text(inline_el, ws);
                if !inline_text.is_empty() {
                    if !inline_buffer.is_empty()
                        && !inline_buffer.ends_with(' ')
                        && !inline_text.starts_with(' ')
                    {
                        inline_buffer.push(' ');
                    }
                    inline_buffer.push_str(&inline_text);
                }
                i += 1;
            }
            ChildSegment::BlockElement {
                element_ref: block_el,
            } => {
                // Flush any accumulated inline content first
                if !inline_buffer.is_empty() {
                    flush_inline_run(
                        &inline_buffer,
                        &styles,
                        id_counter,
                        child_available_width,
                        &link_href_from_parent,
                        &mut children,
                    );
                    inline_buffer.clear();
                }

                // Process block element normally
                let child_layout = build_layout_tree_from_dom(
                    block_el,
                    css_processor,
                    id_counter,
                    viewport_w,
                    child_available_width,
                    Some(&styles),
                    doc_mode,
                );

                if child_layout.styles.display.as_deref() != Some("none")
                    && child_layout.styles.visibility.as_deref() != Some("hidden")
                {
                    children.push(child_layout);
                }
                i += 1;
            }
        }
    }

    // Flush remaining inline content
    if !inline_buffer.is_empty() {
        flush_inline_run(
            &inline_buffer,
            &styles,
            id_counter,
            child_available_width,
            &link_href_from_parent,
            &mut children,
        );
    }

    LayoutElement {
        id: elem_id,
        tag,
        styles,
        children,
    }
}

/// Build a CSS selector string for matching against parsed rules.
fn build_css_selector(el: &scraper::node::Element) -> String {
    let tag = el.name().to_lowercase();
    let mut selector = tag.clone();

    if let Some(id) = el.attr("id") {
        selector.push('#');
        selector.push_str(id);
    }

    if let Some(classes) = el.attr("class") {
        for cls in classes.split_whitespace() {
            selector.push('.');
            selector.push_str(cls);
        }
    }

    selector
}

/// Merge source styles into dest (source overrides dest for non-None fields)
fn merge_styles(dest: &mut ComputedStyles, source: &ComputedStyles) {
    if source.display.is_some() {
        dest.display = source.display.clone();
    }
    if source.position.is_some() {
        dest.position = source.position.clone();
    }
    if source.width.is_some() {
        dest.width = source.width.clone();
    }
    if source.height.is_some() {
        dest.height = source.height.clone();
    }
    if source.background_color.is_some() {
        dest.background_color = source.background_color.clone();
    }
    if source.color.is_some() {
        dest.color = source.color.clone();
    }
    if source.font_size.is_some() {
        dest.font_size = source.font_size.clone();
    }
    if source.font_family.is_some() {
        dest.font_family = source.font_family.clone();
    }
    if source.font_weight.is_some() {
        dest.font_weight = source.font_weight.clone();
    }
    if source.flex_direction.is_some() {
        dest.flex_direction = source.flex_direction.clone();
    }
    if source.justify_content.is_some() {
        dest.justify_content = source.justify_content.clone();
    }
    if source.align_items.is_some() {
        dest.align_items = source.align_items.clone();
    }
    if source.gap.is_some() {
        dest.gap = source.gap.clone();
    }
    if source.overflow.is_some() {
        dest.overflow = source.overflow.clone();
    }
    if source.visibility.is_some() {
        dest.visibility = source.visibility.clone();
    }
    if source.opacity.is_some() {
        dest.opacity = source.opacity;
    }
    if source.z_index.is_some() {
        dest.z_index = source.z_index;
    }
    if source.margin.is_some() {
        dest.margin = source.margin.clone();
    }
    if source.padding.is_some() {
        dest.padding = source.padding.clone();
    }
    if source.border.is_some() {
        dest.border = source.border.clone();
    }
    if source.border_top.is_some() {
        dest.border_top = source.border_top.clone();
    }
    if source.border_right.is_some() {
        dest.border_right = source.border_right.clone();
    }
    if source.border_bottom.is_some() {
        dest.border_bottom = source.border_bottom.clone();
    }
    if source.border_left.is_some() {
        dest.border_left = source.border_left.clone();
    }
    // Promoted properties
    if source.flex_wrap.is_some() {
        dest.flex_wrap = source.flex_wrap.clone();
    }
    if source.align_self.is_some() {
        dest.align_self = source.align_self.clone();
    }
    if source.flex_grow.is_some() {
        dest.flex_grow = source.flex_grow.clone();
    }
    if source.flex_shrink.is_some() {
        dest.flex_shrink = source.flex_shrink.clone();
    }
    if source.flex_basis.is_some() {
        dest.flex_basis = source.flex_basis.clone();
    }
    if source.min_width.is_some() {
        dest.min_width = source.min_width.clone();
    }
    if source.min_height.is_some() {
        dest.min_height = source.min_height.clone();
    }
    if source.max_width.is_some() {
        dest.max_width = source.max_width.clone();
    }
    if source.max_height.is_some() {
        dest.max_height = source.max_height.clone();
    }
    if source.font_style.is_some() {
        dest.font_style = source.font_style.clone();
    }
    if source.line_height.is_some() {
        dest.line_height = source.line_height.clone();
    }
    if source.text_align.is_some() {
        dest.text_align = source.text_align.clone();
    }
    if source.text_decoration.is_some() {
        dest.text_decoration = source.text_decoration.clone();
    }
    if source.text_transform.is_some() {
        dest.text_transform = source.text_transform.clone();
    }
    if source.white_space.is_some() {
        dest.white_space = source.white_space.clone();
    }
    if source.letter_spacing.is_some() {
        dest.letter_spacing = source.letter_spacing.clone();
    }
    if source.word_spacing.is_some() {
        dest.word_spacing = source.word_spacing.clone();
    }
    if source.border_radius.is_some() {
        dest.border_radius = source.border_radius.clone();
    }
    if source.list_style_type.is_some() {
        dest.list_style_type = source.list_style_type.clone();
    }
    if source.cursor.is_some() {
        dest.cursor = source.cursor.clone();
    }
    if source.grid_template_columns.is_some() {
        dest.grid_template_columns = source.grid_template_columns.clone();
    }
    if source.grid_template_rows.is_some() {
        dest.grid_template_rows = source.grid_template_rows.clone();
    }
    if source.grid_template_areas.is_some() {
        dest.grid_template_areas = source.grid_template_areas.clone();
    }
    if source.grid_area.is_some() {
        dest.grid_area = source.grid_area.clone();
    }
    for (k, v) in &source.other {
        dest.other.insert(k.clone(), v.clone());
    }
}

/// Check if a ResolvedStyles has any non-None property (used to avoid emitting empty hover_styles).
fn has_any_hover_property(styles: &ResolvedStyles) -> bool {
    styles.color.is_some()
        || styles.background_color.is_some()
        || styles.text_decoration.is_some()
        || styles.opacity.is_some()
        || styles.cursor.is_some()
        || styles.border_color.is_some()
        || styles.border_width.is_some()
        || styles.border_style.is_some()
        || styles.font_weight.is_some()
        || styles.font_size.is_some()
        || styles.font_style.is_some()
        || styles.display.is_some()
        || styles.visibility.is_some()
}

/// Build a unique CSS selector for an element (for JS event dispatch from GUI).
/// Produces selectors like "div#main", "a.nav-link", "nav > ul > li:nth-child(2) > a".
fn build_element_selector(element_ref: &ElementRef) -> String {
    let el = element_ref.value();
    let tag = el.name().to_lowercase();
    let mut selector = tag.clone();

    // Add ID if present (makes it unique immediately)
    if let Some(id) = el.attr("id") {
        selector.push('#');
        selector.push_str(id);
        return selector;
    }

    // Add classes
    if let Some(classes) = el.attr("class") {
        for cls in classes.split_whitespace() {
            // Skip dynamically generated class names (common in CSS-in-JS)
            if !cls.is_empty() && !cls.contains("__") {
                selector.push('.');
                selector.push_str(cls);
            }
        }
    }

    selector
}

/// Extract width and height from an SVG element's attributes.
/// Checks `width`/`height` attributes first, then falls back to `viewBox`.
/// Returns (width, height) in pixels, defaulting to 20×20 for icons.
fn extract_svg_dimensions(el: &scraper::node::Element) -> (f32, f32) {
    // Try explicit width/height attributes
    let w_attr = el.attr("width").and_then(|v| {
        let v = v.trim().trim_end_matches("px");
        v.parse::<f32>().ok()
    });
    let h_attr = el.attr("height").and_then(|v| {
        let v = v.trim().trim_end_matches("px");
        v.parse::<f32>().ok()
    });

    if let (Some(w), Some(h)) = (w_attr, h_attr) {
        return (w, h);
    }

    // Fall back to viewBox="minX minY width height"
    if let Some(vb) = el.attr("viewBox") {
        let parts: Vec<f32> = vb
            .split_whitespace()
            .filter_map(|p| p.parse::<f32>().ok())
            .collect();
        if parts.len() >= 4 {
            let vb_w = parts[2];
            let vb_h = parts[3];
            // If one explicit dimension is set, scale the other from viewBox aspect ratio
            if let Some(w) = w_attr {
                return (w, w * vb_h / vb_w.max(1.0));
            }
            if let Some(h) = h_attr {
                return (h * vb_w / vb_h.max(1.0), h);
            }
            return (vb_w, vb_h);
        }
    }

    // Default for icon SVGs (most Wiktionary/Wikipedia inline SVGs are 20×20)
    (w_attr.unwrap_or(20.0), h_attr.unwrap_or(20.0))
}

/// Check if a tag is a block-level element
fn is_block_element(tag: &str) -> bool {
    matches!(
        tag,
        "html"
            | "body"
            | "div"
            | "p"
            | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "section"
            | "article"
            | "header"
            | "footer"
            | "main"
            | "nav"
            | "aside"
            | "blockquote"
            | "pre"
            | "hr"
            | "ul"
            | "ol"
            | "li"
            | "table"
            | "form"
            | "figure"
            | "figcaption"
            | "details"
            | "summary"
            | "dialog"
            | "address"
            | "fieldset"
            | "legend"
    )
}

/// Check if a tag is an inline-level element (flows with text).
fn is_inline_element(tag: &str) -> bool {
    matches!(
        tag,
        "a" | "span"
            | "strong"
            | "b"
            | "em"
            | "i"
            | "u"
            | "s"
            | "strike"
            | "code"
            | "kbd"
            | "samp"
            | "var"
            | "mark"
            | "small"
            | "big"
            | "sub"
            | "sup"
            | "abbr"
            | "cite"
            | "dfn"
            | "q"
            | "time"
            | "data"
            | "ruby"
            | "bdo"
            | "bdi"
            | "wbr"
            | "del"
            | "ins"
            | "label"
    )
}

/// Visual data extracted from LayoutElement tree for post-processing
struct VisualData {
    text_content: Option<String>,
    link_href: Option<String>,
    img_src: Option<String>,
    tag: String,
}

/// Build a map from element ID -> visual data
fn build_visual_map(element: &LayoutElement) -> HashMap<String, VisualData> {
    let mut map = HashMap::new();
    collect_visual_data(element, &mut map);
    map
}

fn collect_visual_data(element: &LayoutElement, map: &mut HashMap<String, VisualData>) {
    let text_content = element.styles.other.get("__text_content").cloned();
    let link_href = element.styles.other.get("__link_href").cloned();
    let img_src = if element.tag == "img" {
        element.styles.other.get("__img_src").cloned()
    } else {
        None
    };

    map.insert(
        element.id.clone(),
        VisualData {
            text_content,
            link_href,
            img_src,
            tag: element.tag.clone(),
        },
    );

    for child in &element.children {
        collect_visual_data(child, map);
    }
}

/// Apply visual properties from the visual map to the ElementLayout tree
fn apply_visual_properties(layout: &mut ElementLayout, visual_map: &HashMap<String, VisualData>) {
    if let Some(visual) = visual_map.get(&layout.id) {
        layout.text_content = visual.text_content.clone();
        layout.link_href = visual.link_href.clone();
        layout.img_src = visual.img_src.clone();
    }

    for child in &mut layout.children {
        apply_visual_properties(child, visual_map);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_page_layout() {
        let html = r#"
        <html>
        <head><title>Test</title></head>
        <body>
            <h1>Hello World</h1>
            <p>This is a paragraph.</p>
        </body>
        </html>
        "#;

        let result = compute_page_layout(html, 1024.0, 768.0).unwrap();
        assert!(result.width > 0.0);
        assert!(!result.elements.is_empty());
    }

    #[test]
    fn test_style_extraction() {
        let html = r#"
        <html>
        <head>
            <style>
                body { background-color: #f0f0f0; }
                .container { max-width: 600px; margin: 0 auto; }
            </style>
        </head>
        <body>
            <div class="container">
                <p>Styled content</p>
            </div>
        </body>
        </html>
        "#;

        let result = compute_page_layout(html, 1024.0, 768.0).unwrap();
        assert!(!result.elements.is_empty());
    }

    #[test]
    fn test_inline_styles() {
        let html = r#"
        <html>
        <body>
            <div style="background-color: red; padding: 20px;">
                <p>Inline styled</p>
            </div>
        </body>
        </html>
        "#;

        let result = compute_page_layout(html, 800.0, 600.0).unwrap();
        assert!(!result.elements.is_empty());
    }

    #[test]
    fn test_example_com_layout() {
        let html = r#"<!doctype html>
<html>
<head>
    <title>Example Domain</title>
    <meta charset="utf-8" />
    <meta http-equiv="Content-type" content="text/html; charset=utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <style type="text/css">
    body {
        background-color: #f0f0f2;
        margin: 0;
        padding: 0;
        font-family: -apple-system, system-ui, BlinkMacSystemFont, "Segoe UI",
            "Open Sans", "Helvetica Neue", Helvetica, Arial, sans-serif;
    }
    div {
        width: 600px;
        margin: 5em auto;
        padding: 2em;
        background-color: #fdfdff;
        border-radius: 0.5em;
        box-shadow: 2px 3px 7px 2px rgba(0,0,0,0.02);
    }
    a:link, a:visited {
        color: #38488f;
        text-decoration: none;
    }
    @media (max-width: 700px) {
        div {
            margin: 0 auto;
            width: auto;
        }
    }
    </style>
</head>
<body>
<div>
    <h1>Example Domain</h1>
    <p>This domain is for use in illustrative examples in documents. You may use this
    domain in literature without prior coordination or asking for permission.</p>
    <p><a href="https://www.iana.org/domains/examples">More information...</a></p>
</div>
</body>
</html>"#;

        let result = compute_page_layout(html, 1024.0, 768.0).unwrap();
        let json = serde_json::to_string_pretty(&result).unwrap();

        assert!(result.width > 0.0);
        assert!(!result.elements.is_empty());

        // Check that text content is present
        fn find_text(el: &super::ElementLayout, texts: &mut Vec<String>) {
            if let Some(ref t) = el.text_content {
                texts.push(t.clone());
            }
            for child in &el.children {
                find_text(child, texts);
            }
        }
        let mut texts = Vec::new();
        for el in &result.elements {
            find_text(el, &mut texts);
        }

        assert!(!texts.is_empty(), "Should find text content in layout");
        assert!(
            texts.iter().any(|t| t.contains("Example Domain")),
            "Should contain 'Example Domain'"
        );
    }

    #[test]
    fn test_styled_tree_basic() {
        let html = r#"
        <html>
        <head><title>Test</title></head>
        <body>
            <h1>Hello World</h1>
            <p>This is a <strong>paragraph</strong>.</p>
        </body>
        </html>
        "#;

        let result = compute_styled_tree(html, 1024.0, 768.0).unwrap();
        assert_eq!(result.root.tag, "html");
        assert_eq!(result.viewport_width, 1024.0);

        // Should have body as a child of html
        let body = result.root.children.iter().find(|c| c.tag == "body");
        assert!(body.is_some(), "Should have body element");

        // Body should have children (h1, p)
        let body = body.unwrap();
        assert!(!body.children.is_empty(), "Body should have children");
    }

    #[test]
    fn test_styled_tree_preserves_css() {
        let html = r#"
        <html>
        <head>
            <style>
                .container { max-width: 600px; margin: 0 auto; background-color: #fdfdff; }
            </style>
        </head>
        <body>
            <div class="container"><p>Content</p></div>
        </body>
        </html>
        "#;

        let result = compute_styled_tree(html, 1024.0, 768.0).unwrap();

        // Find the container div
        fn find_by_tag<'a>(el: &'a StyledElement, tag: &str) -> Option<&'a StyledElement> {
            if el.tag == tag {
                return Some(el);
            }
            for child in &el.children {
                if let Some(found) = find_by_tag(child, tag) {
                    return Some(found);
                }
            }
            None
        }

        let div = find_by_tag(&result.root, "div");
        assert!(div.is_some(), "Should find div element");
        let div = div.unwrap();

        // Should have max-width from CSS
        assert_eq!(div.styles.max_width.as_deref(), Some("600px"));
        // Should have background-color from CSS
        assert_eq!(div.styles.background_color.as_deref(), Some("#fdfdff"));
    }

    #[test]
    fn test_styled_tree_images() {
        // Note: <a> is inline, so it gets merged into a #text element by flush_inline_run.
        // Only block-level elements like <img> (inline-block) appear as separate StyledElements.
        let html = r#"
        <html>
        <body>
            <img src="photo.jpg" alt="A photo" width="200" height="100" />
        </body>
        </html>
        "#;

        let result = compute_styled_tree(html, 1024.0, 768.0).unwrap();

        fn find_by_tag<'a>(el: &'a StyledElement, tag: &str) -> Option<&'a StyledElement> {
            if el.tag == tag {
                return Some(el);
            }
            for child in &el.children {
                if let Some(found) = find_by_tag(child, tag) {
                    return Some(found);
                }
            }
            None
        }

        // Check image
        let img = find_by_tag(&result.root, "img");
        assert!(img.is_some(), "Should find img element");
        let img = img.unwrap();
        assert_eq!(img.img_src.as_deref(), Some("photo.jpg"));
        assert_eq!(img.img_alt.as_deref(), Some("A photo"));
    }

    #[test]
    fn test_styled_tree_text_content() {
        // When <a> wraps text, the link href gets stored on the #text if <a> is the
        // direct parent being processed. When <a> is inside <p>, the text is concatenated
        // into the inline run. This tests that text content flows through correctly.
        let html = r#"
        <html>
        <body>
            <p>Hello world</p>
        </body>
        </html>
        "#;

        let result = compute_styled_tree(html, 1024.0, 768.0).unwrap();

        fn find_text<'a>(el: &'a StyledElement) -> Option<&'a StyledElement> {
            if el.text_content.is_some() {
                return Some(el);
            }
            for child in &el.children {
                if let Some(found) = find_text(child) {
                    return Some(found);
                }
            }
            None
        }

        let text_el = find_text(&result.root);
        assert!(text_el.is_some(), "Should find element with text_content");
        assert!(
            text_el
                .unwrap()
                .text_content
                .as_ref()
                .unwrap()
                .contains("Hello world")
        );
    }

    #[test]
    fn test_styled_tree_serializes_to_json() {
        let html = r#"<html><body><p>Hello</p></body></html>"#;
        let result = compute_styled_tree(html, 800.0, 600.0).unwrap();
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"tag\":\"html\""));
        assert!(json.contains("\"viewport_width\":800.0"));
    }

    #[test]
    fn test_styled_tree_css_colors() {
        let html = r#"<html><head><style>
            body { color: #333333; background-color: #ffffff; }
            .red { color: red; }
            .blue-bg { background-color: blue; }
        </style></head><body>
            <p class="red">Red text</p>
            <p class="blue-bg">Blue bg</p>
            <p>Inherited color</p>
        </body></html>"#;
        let result = compute_styled_tree(html, 800.0, 600.0).unwrap();
        let json = serde_json::to_string_pretty(&result).unwrap();
        eprintln!("STYLED TREE JSON:\n{}", json);

        // Find the elements and check colors
        fn find_text_elements(
            el: &StyledElement,
            results: &mut Vec<(String, Option<String>, Option<String>)>,
        ) {
            if let Some(ref text) = el.text_content {
                if !text.trim().is_empty() {
                    results.push((
                        text.clone(),
                        el.styles.color.clone(),
                        el.styles.background_color.clone(),
                    ));
                }
            }
            for child in &el.children {
                find_text_elements(child, results);
            }
        }
        let mut text_elements = Vec::new();
        find_text_elements(&result.root, &mut text_elements);

        eprintln!("TEXT ELEMENTS:");
        for (text, color, bg) in &text_elements {
            eprintln!("  text='{}' color={:?} bg={:?}", text, color, bg);
        }

        // Red text should have color: red
        let red_el = text_elements
            .iter()
            .find(|(t, _, _)| t.contains("Red text"))
            .expect("Should find 'Red text'");
        assert!(
            red_el.1.is_some(),
            "Red text should have a color set: {:?}",
            red_el
        );

        // Blue bg: background-color doesn't inherit to #text children, it stays on parent
        // So the text element won't have it. This is correct CSS behavior.

        // Inherited color should have color from body
        let inherited_el = text_elements
            .iter()
            .find(|(t, _, _)| t.contains("Inherited"))
            .expect("Should find 'Inherited color'");
        assert!(
            inherited_el.1.is_some(),
            "Inherited color should have color from body: {:?}",
            inherited_el
        );
    }

    #[test]
    fn test_styled_tree_css_variables() {
        // Simulate Tailwind-style CSS variables
        let html = r#"<html><head><style>
            :root { --text-color: #1a1a1a; --bg-color: #f5f5f5; }
            *, :before, :after { --tw-text-opacity: 1; }
            body { color: var(--text-color); background-color: var(--bg-color); }
            .tw-black { color: rgb(0 0 0 / var(--tw-text-opacity, 1)); }
        </style></head><body>
            <p>Body text should be #1a1a1a</p>
            <p class="tw-black">Tailwind black</p>
        </body></html>"#;
        let result = compute_styled_tree(html, 800.0, 600.0).unwrap();

        fn find_text_elements(
            el: &StyledElement,
            results: &mut Vec<(String, Option<String>, Option<String>)>,
        ) {
            if let Some(ref text) = el.text_content {
                if !text.trim().is_empty() {
                    results.push((
                        text.clone(),
                        el.styles.color.clone(),
                        el.styles.background_color.clone(),
                    ));
                }
            }
            for child in &el.children {
                find_text_elements(child, results);
            }
        }
        let mut text_elements = Vec::new();
        find_text_elements(&result.root, &mut text_elements);

        eprintln!("CSS VARIABLE TEST:");
        for (text, color, bg) in &text_elements {
            eprintln!("  text='{}' color={:?} bg={:?}", text, color, bg);
        }

        // Body text should have resolved var(--text-color) → #1a1a1a
        let body_el = text_elements
            .iter()
            .find(|(t, _, _)| t.contains("Body text"))
            .expect("Should find 'Body text'");
        assert!(
            body_el.1.is_some(),
            "Body text should have color resolved from var(): {:?}",
            body_el
        );
        assert!(
            !body_el.1.as_ref().unwrap().contains("var("),
            "Color should be resolved, not contain var(): {:?}",
            body_el.1
        );

        // Tailwind black — rgb(0 0 0 / var(--tw-text-opacity, 1))
        let tw_el = text_elements
            .iter()
            .find(|(t, _, _)| t.contains("Tailwind"))
            .expect("Should find 'Tailwind black'");
        eprintln!("  Tailwind black resolved color: {:?}", tw_el.1);
        assert!(
            tw_el.1.is_some(),
            "Tailwind text should have a color: {:?}",
            tw_el
        );
    }

    #[test]
    fn test_cloudflare_nav_display_none() {
        // End-to-end test: real HTML + real CSS → nav should have display:none
        let html_path = "/tmp/cloudflare_full.html";
        let ashes_path = "/tmp/ashes_test.css";
        let index_path = "/tmp/cloudflare_test.css";
        if !std::path::Path::new(html_path).exists()
            || !std::path::Path::new(ashes_path).exists()
            || !std::path::Path::new(index_path).exists()
        {
            eprintln!("Skipping: test files not found");
            return;
        }

        let html = std::fs::read_to_string(html_path).unwrap();
        let ashes = std::fs::read_to_string(ashes_path).unwrap();
        let index = std::fs::read_to_string(index_path).unwrap();

        let result = compute_styled_tree_with_css(&html, 1280.0, 800.0, &[ashes, index]).unwrap();

        // Find <nav> elements in the styled tree
        fn find_navs(el: &StyledElement, results: &mut Vec<(String, Option<String>)>) {
            if el.tag == "nav" {
                results.push((el.tag.clone(), el.styles.display.clone()));
            }
            for child in &el.children {
                find_navs(child, results);
            }
        }

        let mut navs = Vec::new();
        find_navs(&result.root, &mut navs);
        eprintln!("Found {} nav elements:", navs.len());
        for (tag, display) in &navs {
            eprintln!("  <{}> display={:?}", tag, display);
        }

        // The first nav (with class "db dn-l") should be display:none at 1280px
        assert!(!navs.is_empty(), "Should find at least one nav");
        assert_eq!(
            navs[0].1.as_deref(),
            Some("none"),
            "First nav should be display:none at 1280px (dn-l media query)"
        );
    }

    #[test]
    fn test_grid_template_areas_parsing() {
        // Verify grid-template shorthand, grid-template-areas, grid-area, and column-gap
        // all flow through correctly from CSS to the styled tree
        let html = r#"
        <html>
        <head><style>
            .grid-container {
                display: grid;
                column-gap: 24px;
                grid-template: min-content 1fr min-content / 12.25rem minmax(0,1fr);
                grid-template-areas: 'siteNotice siteNotice' 'columnStart pageContent' 'footer footer';
            }
            .child1 { grid-area: siteNotice; }
            .child2 { grid-area: columnStart; }
            .child3 { grid-area: pageContent; }
            .child4 { grid-area: footer; }
        </style></head>
        <body>
            <div class="grid-container">
                <div class="child1">Notice</div>
                <div class="child2">Sidebar</div>
                <div class="child3">Content</div>
                <div class="child4">Footer</div>
            </div>
        </body>
        </html>
        "#;

        let result = compute_styled_tree(html, 1280.0, 800.0).unwrap();

        // Find the grid container
        fn find_grid<'a>(el: &'a StyledElement) -> Option<&'a StyledElement> {
            if el.styles.display.as_deref() == Some("grid") {
                return Some(el);
            }
            for child in &el.children {
                if let Some(found) = find_grid(child) {
                    return Some(found);
                }
            }
            None
        }

        let grid = find_grid(&result.root).expect("Should find grid container");

        // Grid container should have columns, rows, areas, and gap
        assert!(
            grid.styles.grid_template_columns.is_some(),
            "Should have grid-template-columns"
        );
        assert!(
            grid.styles.grid_template_rows.is_some(),
            "Should have grid-template-rows"
        );
        assert!(
            grid.styles.grid_template_areas.is_some(),
            "Should have grid-template-areas"
        );
        assert_eq!(
            grid.styles.gap.as_deref(),
            Some("24px"),
            "Should have gap from column-gap"
        );

        // Children should have grid-area properties
        let children_with_area: Vec<_> = grid
            .children
            .iter()
            .filter(|c| c.styles.grid_area.is_some())
            .collect();
        assert_eq!(
            children_with_area.len(),
            4,
            "Should have 4 children with grid-area"
        );

        let area_names: Vec<&str> = children_with_area
            .iter()
            .map(|c| c.styles.grid_area.as_deref().unwrap())
            .collect();
        assert!(area_names.contains(&"siteNotice"));
        assert!(area_names.contains(&"columnStart"));
        assert!(area_names.contains(&"pageContent"));
        assert!(area_names.contains(&"footer"));
    }
}
