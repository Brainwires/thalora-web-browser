//! Page Layout Bridge
//!
//! Takes raw HTML, extracts `<style>` blocks, walks the DOM to compute CSS per element
//! via CssProcessor (lightningcss), builds a LayoutElement tree, runs LayoutEngine (taffy),
//! and returns a LayoutResult with full visual properties suitable for JSON serialization
//! and consumption by the C# GUI layer.

use anyhow::{Result, Context};
use scraper::{Html, Selector, Node, ElementRef};
use std::collections::HashMap;

use super::css::{CssProcessor, ComputedStyles, BoxModel, BorderStyles};
use super::layout::{
    LayoutEngine, LayoutElement, LayoutResult, ElementLayout, BoxModelSides, parse_px_value,
};
use super::styled_tree::{StyledTreeResult, StyledElement, ResolvedStyles, StyleBoxSides};

/// User-agent default styles for block-level elements.
/// These mirror the CSS 2.1 spec defaults that browsers apply.
fn apply_ua_defaults(tag: &str, styles: &mut ComputedStyles) {
    match tag {
        "html" => {
            if styles.display.is_none() { styles.display = Some("block".to_string()); }
            // Browsers render the html element with a white canvas background by default.
            // CSS spec says transparent, but the viewport/canvas is white — we apply it here
            // so the C# renderer picks it up. Page stylesheets can override this.
            if styles.background_color.is_none() {
                styles.background_color = Some("#ffffff".to_string());
            }
            // Default text color is black (CSS initial value). This inherits to all children.
            if styles.color.is_none() {
                styles.color = Some("#000000".to_string());
            }
        }
        "body" => {
            if styles.display.is_none() { styles.display = Some("block".to_string()); }
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
            if styles.display.is_none() { styles.display = Some("block".to_string()); }
            if styles.font_size.is_none() { styles.font_size = Some("32px".to_string()); }
            if styles.font_weight.is_none() { styles.font_weight = Some("bold".to_string()); }
            if styles.margin.is_none() {
                styles.margin = Some(BoxModel {
                    top: "21.44px".to_string(), right: "0px".to_string(),
                    bottom: "21.44px".to_string(), left: "0px".to_string(),
                });
            }
        }
        "h2" => {
            if styles.display.is_none() { styles.display = Some("block".to_string()); }
            if styles.font_size.is_none() { styles.font_size = Some("24px".to_string()); }
            if styles.font_weight.is_none() { styles.font_weight = Some("bold".to_string()); }
            if styles.margin.is_none() {
                styles.margin = Some(BoxModel {
                    top: "19.92px".to_string(), right: "0px".to_string(),
                    bottom: "19.92px".to_string(), left: "0px".to_string(),
                });
            }
        }
        "h3" => {
            if styles.display.is_none() { styles.display = Some("block".to_string()); }
            if styles.font_size.is_none() { styles.font_size = Some("18.72px".to_string()); }
            if styles.font_weight.is_none() { styles.font_weight = Some("bold".to_string()); }
            if styles.margin.is_none() {
                styles.margin = Some(BoxModel {
                    top: "18.72px".to_string(), right: "0px".to_string(),
                    bottom: "18.72px".to_string(), left: "0px".to_string(),
                });
            }
        }
        "h4" => {
            if styles.display.is_none() { styles.display = Some("block".to_string()); }
            if styles.font_size.is_none() { styles.font_size = Some("16px".to_string()); }
            if styles.font_weight.is_none() { styles.font_weight = Some("bold".to_string()); }
            if styles.margin.is_none() {
                styles.margin = Some(BoxModel {
                    top: "21.28px".to_string(), right: "0px".to_string(),
                    bottom: "21.28px".to_string(), left: "0px".to_string(),
                });
            }
        }
        "h5" => {
            if styles.display.is_none() { styles.display = Some("block".to_string()); }
            if styles.font_size.is_none() { styles.font_size = Some("13.28px".to_string()); }
            if styles.font_weight.is_none() { styles.font_weight = Some("bold".to_string()); }
            if styles.margin.is_none() {
                styles.margin = Some(BoxModel {
                    top: "22.18px".to_string(), right: "0px".to_string(),
                    bottom: "22.18px".to_string(), left: "0px".to_string(),
                });
            }
        }
        "h6" => {
            if styles.display.is_none() { styles.display = Some("block".to_string()); }
            if styles.font_size.is_none() { styles.font_size = Some("10.72px".to_string()); }
            if styles.font_weight.is_none() { styles.font_weight = Some("bold".to_string()); }
            if styles.margin.is_none() {
                styles.margin = Some(BoxModel {
                    top: "24.97px".to_string(), right: "0px".to_string(),
                    bottom: "24.97px".to_string(), left: "0px".to_string(),
                });
            }
        }
        "p" => {
            if styles.display.is_none() { styles.display = Some("block".to_string()); }
            if styles.margin.is_none() {
                styles.margin = Some(BoxModel {
                    top: "16px".to_string(), right: "0px".to_string(),
                    bottom: "16px".to_string(), left: "0px".to_string(),
                });
            }
        }
        "div" | "section" | "article" | "header" | "footer" | "main" | "nav" | "aside"
        | "form" | "figure" | "figcaption" | "details" | "summary" | "dialog" | "address"
        | "fieldset" | "legend" => {
            if styles.display.is_none() {
                styles.display = Some("block".to_string());
            }
        }
        "blockquote" => {
            if styles.display.is_none() { styles.display = Some("block".to_string()); }
            if styles.margin.is_none() {
                styles.margin = Some(BoxModel {
                    top: "16px".to_string(), right: "40px".to_string(),
                    bottom: "16px".to_string(), left: "40px".to_string(),
                });
            }
        }
        "pre" => {
            if styles.display.is_none() { styles.display = Some("block".to_string()); }
            if styles.font_family.is_none() { styles.font_family = Some("monospace".to_string()); }
            styles.other.entry("white-space".to_string()).or_insert_with(|| "pre".to_string());
            if styles.background_color.is_none() {
                styles.background_color = Some("#f4f4f4".to_string());
            }
            if styles.padding.is_none() {
                styles.padding = Some(BoxModel {
                    top: "16px".to_string(), right: "16px".to_string(),
                    bottom: "16px".to_string(), left: "16px".to_string(),
                });
            }
            if styles.margin.is_none() {
                styles.margin = Some(BoxModel {
                    top: "16px".to_string(), right: "0px".to_string(),
                    bottom: "16px".to_string(), left: "0px".to_string(),
                });
            }
            styles.other.entry("overflow".to_string()).or_insert_with(|| "hidden".to_string());
            styles.other.entry("border-radius".to_string()).or_insert_with(|| "4px".to_string());
        }
        "code" => {
            if styles.font_family.is_none() { styles.font_family = Some("monospace".to_string()); }
            // Inline code gets subtle background (not inside <pre> — parent handles that)
            if styles.background_color.is_none() {
                styles.background_color = Some("#f0f0f0".to_string());
            }
            if styles.padding.is_none() {
                styles.padding = Some(BoxModel {
                    top: "2px".to_string(), right: "4px".to_string(),
                    bottom: "2px".to_string(), left: "4px".to_string(),
                });
            }
            styles.other.entry("border-radius".to_string()).or_insert_with(|| "3px".to_string());
        }
        "hr" => {
            if styles.display.is_none() { styles.display = Some("block".to_string()); }
            if styles.margin.is_none() {
                styles.margin = Some(BoxModel {
                    top: "8px".to_string(), right: "0px".to_string(),
                    bottom: "8px".to_string(), left: "0px".to_string(),
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
            if styles.display.is_none() { styles.display = Some("block".to_string()); }
            if styles.margin.is_none() {
                styles.margin = Some(BoxModel {
                    top: "16px".to_string(), right: "0px".to_string(),
                    bottom: "16px".to_string(), left: "0px".to_string(),
                });
            }
            if styles.padding.is_none() {
                styles.padding = Some(BoxModel {
                    top: "0px".to_string(), right: "0px".to_string(),
                    bottom: "0px".to_string(), left: "40px".to_string(),
                });
            }
        }
        "li" => {
            if styles.display.is_none() { styles.display = Some("list-item".to_string()); }
            styles.other.entry("list-style-type".to_string()).or_insert_with(|| "disc".to_string());
        }
        "a" => {
            if styles.color.is_none() { styles.color = Some("#0051C3".to_string()); }
            styles.other.entry("text-decoration".to_string()).or_insert_with(|| "underline".to_string());
        }
        "strong" | "b" => {
            if styles.font_weight.is_none() { styles.font_weight = Some("bold".to_string()); }
        }
        "em" | "i" => {
            styles.other.entry("font-style".to_string()).or_insert_with(|| "italic".to_string());
        }
        "span" | "label" => {
            if styles.display.is_none() { styles.display = Some("inline".to_string()); }
        }
        "img" | "input" | "textarea" | "select" => {
            if styles.display.is_none() { styles.display = Some("inline-block".to_string()); }
        }
        "br" => {
            if styles.display.is_none() { styles.display = Some("inline".to_string()); }
        }
        "table" => {
            if styles.display.is_none() { styles.display = Some("table".to_string()); }
            if styles.margin.is_none() {
                styles.margin = Some(BoxModel {
                    top: "16px".to_string(), right: "0px".to_string(),
                    bottom: "16px".to_string(), left: "0px".to_string(),
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
            if styles.display.is_none() { styles.display = Some("table-row".to_string()); }
        }
        "td" => {
            if styles.display.is_none() { styles.display = Some("table-cell".to_string()); }
            if styles.padding.is_none() {
                styles.padding = Some(BoxModel {
                    top: "4px".to_string(), right: "8px".to_string(),
                    bottom: "4px".to_string(), left: "8px".to_string(),
                });
            }
        }
        "th" => {
            if styles.display.is_none() { styles.display = Some("table-cell".to_string()); }
            if styles.font_weight.is_none() { styles.font_weight = Some("bold".to_string()); }
            if styles.padding.is_none() {
                styles.padding = Some(BoxModel {
                    top: "4px".to_string(), right: "8px".to_string(),
                    bottom: "4px".to_string(), left: "8px".to_string(),
                });
            }
        }
        "button" => {
            if styles.display.is_none() { styles.display = Some("inline-block".to_string()); }
            if styles.padding.is_none() {
                styles.padding = Some(BoxModel {
                    top: "4px".to_string(), right: "8px".to_string(),
                    bottom: "4px".to_string(), left: "8px".to_string(),
                });
            }
        }
        _ => {}
    }
}

/// Tags that should be skipped during layout (metadata/invisible)
const SKIP_TAGS: &[&str] = &[
    "script", "style", "link", "meta", "head", "title", "noscript", "template",
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
pub fn compute_styled_tree(html: &str, viewport_w: f32, viewport_h: f32) -> Result<StyledTreeResult> {
    compute_styled_tree_with_css(html, viewport_w, viewport_h, &[])
}

/// Compute a styled element tree from raw HTML with external CSS stylesheets.
pub fn compute_styled_tree_with_css(
    html: &str,
    viewport_w: f32,
    viewport_h: f32,
    external_css: &[String],
) -> Result<StyledTreeResult> {
    let document = Html::parse_document(html);

    // Step 1: Parse external stylesheets FIRST (lower source-order precedence)
    let mut css_processor = CssProcessor::new_with_viewport(viewport_w);
    for css_text in external_css {
        if !css_text.trim().is_empty() {
            if let Err(e) = css_processor.parse(css_text) {
                eprintln!("[styled_tree] Failed to parse external stylesheet: {}", e);
            }
        }
    }

    // Step 1b: Then parse <style> blocks (higher source-order precedence)
    let style_selector = Selector::parse("style").unwrap();
    for style_el in document.select(&style_selector) {
        let css_text: String = style_el.text().collect();
        if !css_text.trim().is_empty() {
            if let Err(e) = css_processor.parse(&css_text) {
                eprintln!("[styled_tree] Failed to parse <style> block: {}", e);
            }
        }
    }

    // Step 2: Walk the DOM tree and build StyledElement tree directly.
    // This preserves inline element structure (<a>, <strong>, <em>, <code>, <span>)
    // so C# can properly render links, bold, italic, inline code, etc.
    let root_node = document.root_element();
    let mut id_counter: u32 = 0;
    let root = build_styled_element_from_dom(
        &root_node,
        &css_processor,
        &mut id_counter,
        viewport_w,
        None,
    );

    Ok(StyledTreeResult {
        root,
        viewport_width: viewport_w,
        viewport_height: viewport_h,
    })
}

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
) -> StyledElement {
    let el = element_ref.value();
    let tag = el.name().to_lowercase();

    // Compute CSS styles
    let mut styles = css_processor.compute_style_for_element(element_ref);

    // Handle inline style attribute
    let elem_id = format!("e{}", *id_counter);
    *id_counter += 1;

    if let Some(inline_style) = el.attr("style") {
        let mut inline_processor = CssProcessor::new();
        if inline_processor.parse_inline_style(inline_style, &elem_id).is_ok() {
            let inline_styles = inline_processor.compute_style(&format!("#{}", elem_id));
            merge_styles(&mut styles, &inline_styles);
        }
    }

    // Inherit properties from parent
    if let Some(parent) = parent_styles {
        inherit_properties(&mut styles, parent);
    }

    // Apply UA defaults
    apply_ua_defaults(&tag, &mut styles);

    // Default display for elements if not set
    if styles.display.is_none() {
        styles.display = Some(if is_block_element(&tag) { "block" } else { "inline" }.to_string());
    }

    // Extract HTML attributes
    let link_href = if tag == "a" {
        el.attr("href").map(|h| h.to_string())
    } else {
        None
    };

    let img_src = if tag == "img" {
        el.attr("src").map(|s| s.to_string())
    } else {
        None
    };

    let img_alt = if tag == "img" {
        el.attr("alt").map(|a| a.to_string())
    } else {
        None
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

    // White-space mode for text collapsing
    let ws = styles.other.get("white-space").cloned().unwrap_or_else(|| "normal".to_string());

    // Build children — preserve ALL elements (inline and block) as children
    let mut children = Vec::new();

    for child_node in element_ref.children() {
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
                        // Whitespace-only text between inline elements → single space
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
                    font_style: styles.other.get("font-style").cloned(),
                    line_height: styles.other.get("line-height").cloned(),
                    color: styles.color.clone(),
                    white_space: styles.other.get("white-space").cloned(),
                    text_decoration: styles.other.get("text-decoration").cloned(),
                    letter_spacing: styles.other.get("letter-spacing").cloned(),
                    ..ResolvedStyles::default()
                };

                children.push(StyledElement {
                    id: text_id,
                    tag: "#text".to_string(),
                    text_content: Some(text_str),
                    img_src: None,
                    img_alt: None,
                    link_href: None,
                    styles: text_resolved,
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
                    );

                    // Skip display:none and visibility:hidden elements
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

    // Convert ComputedStyles → ResolvedStyles
    let resolved = computed_to_resolved(&styles);

    StyledElement {
        id: elem_id,
        tag,
        text_content: None, // Block/inline elements carry text via #text children
        img_src,
        img_alt,
        link_href,
        styles: resolved,
        children,
    }
}

/// Convert ComputedStyles to ResolvedStyles (shared conversion logic)
fn computed_to_resolved(styles: &ComputedStyles) -> ResolvedStyles {
    ResolvedStyles {
        display: styles.display.clone(),
        position: styles.position.clone(),
        flex_direction: styles.flex_direction.clone(),
        flex_wrap: styles.other.get("flex-wrap").cloned(),
        justify_content: styles.justify_content.clone(),
        align_items: styles.align_items.clone(),
        align_self: styles.other.get("align-self").cloned(),
        gap: styles.gap.clone(),
        flex_grow: styles.other.get("flex-grow").cloned(),
        flex_shrink: styles.other.get("flex-shrink").cloned(),
        flex_basis: styles.other.get("flex-basis").cloned(),

        width: styles.width.clone(),
        height: styles.height.clone(),
        min_width: styles.other.get("min-width").cloned(),
        min_height: styles.other.get("min-height").cloned(),
        max_width: styles.other.get("max-width").cloned(),
        max_height: styles.other.get("max-height").cloned(),

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
        font_style: styles.other.get("font-style").cloned(),
        line_height: styles.other.get("line-height").cloned(),
        text_align: styles.other.get("text-align").cloned(),
        text_decoration: styles.other.get("text-decoration").cloned(),
        text_transform: styles.other.get("text-transform").cloned(),
        white_space: styles.other.get("white-space").cloned(),
        letter_spacing: styles.other.get("letter-spacing").cloned(),
        word_spacing: styles.other.get("word-spacing").cloned(),

        color: styles.color.clone(),
        background_color: styles.background_color.clone(),

        border_width: styles.border.as_ref().map(|b| {
            let w = b.width.clone();
            StyleBoxSides {
                top: w.clone(),
                right: w.clone(),
                bottom: w.clone(),
                left: w,
            }
        }),
        border_color: styles.border.as_ref().and_then(|b| {
            if b.color.is_empty() { None } else { Some(b.color.clone()) }
        }),
        border_style: styles.border.as_ref().and_then(|b| {
            if b.style.is_empty() { None } else { Some(b.style.clone()) }
        }),
        border_radius: styles.other.get("border-radius").cloned(),

        opacity: styles.opacity,
        overflow: styles.overflow.clone(),
        visibility: styles.visibility.clone(),
        z_index: styles.z_index,
        list_style_type: styles.other.get("list-style-type").cloned(),
        cursor: styles.other.get("cursor").cloned(),
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

    // Convert ComputedStyles → ResolvedStyles
    let resolved = ResolvedStyles {
        display: styles.display.clone(),
        position: styles.position.clone(),
        flex_direction: styles.flex_direction.clone(),
        flex_wrap: styles.other.get("flex-wrap").cloned(),
        justify_content: styles.justify_content.clone(),
        align_items: styles.align_items.clone(),
        align_self: styles.other.get("align-self").cloned(),
        gap: styles.gap.clone(),
        flex_grow: styles.other.get("flex-grow").cloned(),
        flex_shrink: styles.other.get("flex-shrink").cloned(),
        flex_basis: styles.other.get("flex-basis").cloned(),

        width: styles.width.clone(),
        height: styles.height.clone(),
        min_width: styles.other.get("min-width").cloned(),
        min_height: styles.other.get("min-height").cloned(),
        max_width: styles.other.get("max-width").cloned(),
        max_height: styles.other.get("max-height").cloned(),

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
        font_style: styles.other.get("font-style").cloned(),
        line_height: styles.other.get("line-height").cloned(),
        text_align: styles.other.get("text-align").cloned(),
        text_decoration: styles.other.get("text-decoration").cloned(),
        text_transform: styles.other.get("text-transform").cloned(),
        white_space: styles.other.get("white-space").cloned(),
        letter_spacing: styles.other.get("letter-spacing").cloned(),
        word_spacing: styles.other.get("word-spacing").cloned(),

        color: styles.color.clone(),
        background_color: styles.background_color.clone(),

        border_width: styles.border.as_ref().map(|b| {
            let w = b.width.clone();
            StyleBoxSides {
                top: w.clone(),
                right: w.clone(),
                bottom: w.clone(),
                left: w,
            }
        }),
        border_color: styles.border.as_ref().and_then(|b| {
            if b.color.is_empty() { None } else { Some(b.color.clone()) }
        }),
        border_style: styles.border.as_ref().and_then(|b| {
            if b.style.is_empty() { None } else { Some(b.style.clone()) }
        }),
        border_radius: styles.other.get("border-radius").cloned(),

        opacity: styles.opacity,
        overflow: styles.overflow.clone(),
        visibility: styles.visibility.clone(),
        z_index: styles.z_index,
        list_style_type: styles.other.get("list-style-type").cloned(),
        cursor: styles.other.get("cursor").cloned(),
    };

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
        styles: resolved,
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
pub fn compute_page_layout_with_css(html: &str, viewport_w: f32, viewport_h: f32, external_css: &[String]) -> Result<LayoutResult> {
    let document = Html::parse_document(html);

    // Step 1: Parse external stylesheets FIRST (lower source-order precedence)
    let mut css_processor = CssProcessor::new_with_viewport(viewport_w);
    for css_text in external_css {
        if !css_text.trim().is_empty() {
            if let Err(e) = css_processor.parse(css_text) {
                eprintln!("[page_layout] Failed to parse external stylesheet: {}", e);
            }
        }
    }

    // Step 1b: Then parse <style> blocks (higher source-order precedence, overrides external)
    let style_selector = Selector::parse("style").unwrap();
    for style_el in document.select(&style_selector) {
        let css_text: String = style_el.text().collect();
        if !css_text.trim().is_empty() {
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
    );

    // Step 2.5: Ensure html and body span the full viewport (CSS spec behavior)
    // The root element should have min-height of 100% of viewport.
    // Body inherits this stretching so backgrounds cover the full viewport.
    let vh = format!("{}px", viewport_h);
    if layout_tree.tag == "html" {
        layout_tree.styles.other
            .entry("min-height".to_string())
            .or_insert_with(|| vh.clone());

        // Find body child and set its min-height too
        for child in &mut layout_tree.children {
            if child.tag == "body" {
                child.styles.other
                    .entry("min-height".to_string())
                    .or_insert_with(|| vh.clone());
                break;
            }
        }
    }

    // Step 3: Run taffy layout
    let mut engine = LayoutEngine::with_viewport(viewport_w, viewport_h);
    let mut layout_result = engine.calculate_layout_from_elements(&layout_tree)
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
    if child.font_size.is_none() {
        child.font_size = parent.font_size.clone();
    }
    if child.font_family.is_none() {
        child.font_family = parent.font_family.clone();
    }
    if child.font_weight.is_none() {
        child.font_weight = parent.font_weight.clone();
    }
    // font-style
    if !child.other.contains_key("font-style") {
        if let Some(fs) = parent.other.get("font-style") {
            child.other.insert("font-style".to_string(), fs.clone());
        }
    }
    // line-height
    if !child.other.contains_key("line-height") {
        if let Some(lh) = parent.other.get("line-height") {
            child.other.insert("line-height".to_string(), lh.clone());
        }
    }
    // text-align
    if !child.other.contains_key("text-align") {
        if let Some(ta) = parent.other.get("text-align") {
            child.other.insert("text-align".to_string(), ta.clone());
        }
    }
    // white-space
    if !child.other.contains_key("white-space") {
        if let Some(ws) = parent.other.get("white-space") {
            child.other.insert("white-space".to_string(), ws.clone());
        }
    }
    // visibility
    if child.visibility.is_none() {
        child.visibility = parent.visibility.clone();
    }
    // text-decoration
    if !child.other.contains_key("text-decoration") {
        if let Some(td) = parent.other.get("text-decoration") {
            child.other.insert("text-decoration".to_string(), td.clone());
        }
    }
    // text-transform
    if !child.other.contains_key("text-transform") {
        if let Some(tt) = parent.other.get("text-transform") {
            child.other.insert("text-transform".to_string(), tt.clone());
        }
    }
    // letter-spacing
    if !child.other.contains_key("letter-spacing") {
        if let Some(ls) = parent.other.get("letter-spacing") {
            child.other.insert("letter-spacing".to_string(), ls.clone());
        }
    }
    // word-spacing
    if !child.other.contains_key("word-spacing") {
        if let Some(ws) = parent.other.get("word-spacing") {
            child.other.insert("word-spacing".to_string(), ws.clone());
        }
    }
    // cursor
    if !child.other.contains_key("cursor") {
        if let Some(c) = parent.other.get("cursor") {
            child.other.insert("cursor".to_string(), c.clone());
        }
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
        if inline_processor.parse_inline_style(inline_style, &elem_id).is_ok() {
            let inline_styles = inline_processor.compute_style(&format!("#{}", elem_id));
            merge_styles(&mut styles, &inline_styles);
        }
    }

    // Inherit properties from parent (Phase 4: CSS inheritance)
    if let Some(parent) = parent_styles {
        inherit_properties(&mut styles, parent);
    }

    // Apply UA defaults (only for properties not already set by CSS)
    apply_ua_defaults(&tag, &mut styles);

    // Capture HTML attributes that affect rendering
    if tag == "img" {
        if let Some(src) = el.attr("src") {
            styles.other.insert("__img_src".to_string(), src.to_string());
        }
        // Use alt text as fallback display content and store separately for the styled tree
        if let Some(alt) = el.attr("alt") {
            styles.other.insert("__text_content".to_string(), alt.to_string());
            styles.other.insert("__img_alt".to_string(), alt.to_string());
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
        styles.display = Some(if is_block_element(&tag) { "block" } else { "inline" }.to_string());
    }

    // Compute available width for children based on this element's styles
    let vw = viewport_w as f64;
    let vh = viewport_w as f64 * 0.5625; // approximate viewport height
    let child_available_width = {
        let mut w = available_width;
        let mut has_explicit_width = false;
        // If this element has an explicit width, use that instead
        if let Some(ref width_str) = styles.width {
            let font_size = styles.font_size.as_ref()
                .and_then(|s| super::layout::resolve_css_length(s, 16.0))
                .unwrap_or(16.0);
            if width_str.ends_with('%') {
                if let Ok(pct) = width_str.trim_end_matches('%').parse::<f64>() {
                    w = available_width * pct / 100.0;
                }
                has_explicit_width = true;
            } else if let Some(px) = super::layout::resolve_css_length_vp(width_str, font_size, viewport_w, viewport_w * 9.0 / 16.0) {
                w = px;
                has_explicit_width = true;
            }
        }
        // Only subtract padding when width is auto (filling parent).
        // Explicit widths are content-box (taffy default) — they already
        // represent the content area, so padding is NOT subtracted.
        if !has_explicit_width {
            if let Some(ref padding) = styles.padding {
                let fs = styles.font_size.as_ref()
                    .and_then(|s| super::layout::resolve_css_length(s, 16.0))
                    .unwrap_or(16.0);
                let pl = super::layout::resolve_css_length_vp(&padding.left, fs, viewport_w, viewport_w * 9.0 / 16.0).unwrap_or(0.0);
                let pr = super::layout::resolve_css_length_vp(&padding.right, fs, viewport_w, viewport_w * 9.0 / 16.0).unwrap_or(0.0);
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
        InlineText { text: String, link_href: Option<String> },
        /// An inline element like <a>, <strong>, <em> — extract its text inline
        InlineElement { element_ref: ElementRef<'a>, tag: String },
        /// A block-level element — process recursively
        BlockElement { element_ref: ElementRef<'a> },
    }

    let ws = styles.other.get("white-space").map(|s| s.as_str()).unwrap_or("normal");
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
                        if !text.is_empty() && !collapsed.is_empty()
                            && !text.ends_with(' ') && !collapsed.starts_with(' ')
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
                        if !text.is_empty() && !inner.is_empty()
                            && !text.ends_with(' ') && !inner.starts_with(' ')
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

        let font_size = styles.font_size.as_ref()
            .and_then(|s| super::layout::resolve_css_length(s, 16.0))
            .unwrap_or(16.0);
        let line_height = styles.other.get("line-height")
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
        if let Some(fs_val) = styles.other.get("font-style") {
            text_styles.other.insert("font-style".to_string(), fs_val.clone());
        }
        if let Some(lh) = styles.other.get("line-height") {
            text_styles.other.insert("line-height".to_string(), lh.clone());
        }
        if let Some(ws_val) = styles.other.get("white-space") {
            text_styles.other.insert("white-space".to_string(), ws_val.clone());
        }
        text_styles.other.insert("min-height".to_string(), format!("{}px", buffered_height));
        text_styles.display = Some("block".to_string());
        text_styles.other.insert("__text_content".to_string(), combined_text.to_string());

        if let Some(href) = link_href {
            text_styles.other.insert("__link_href".to_string(), href.clone());
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
                if !inline_buffer.is_empty() && !inline_buffer.ends_with(' ')
                    && !text.starts_with(' ')
                {
                    inline_buffer.push(' ');
                }
                inline_buffer.push_str(text);
                i += 1;
            }
            ChildSegment::InlineElement { element_ref: inline_el, tag: _inline_tag } => {
                // Extract text from the inline element and append to the buffer
                let inline_text = extract_inline_text(inline_el, ws);
                if !inline_text.is_empty() {
                    if !inline_buffer.is_empty() && !inline_buffer.ends_with(' ')
                        && !inline_text.starts_with(' ')
                    {
                        inline_buffer.push(' ');
                    }
                    inline_buffer.push_str(&inline_text);
                }
                i += 1;
            }
            ChildSegment::BlockElement { element_ref: block_el } => {
                // Flush any accumulated inline content first
                if !inline_buffer.is_empty() {
                    flush_inline_run(
                        &inline_buffer, &styles, id_counter,
                        child_available_width, &link_href_from_parent,
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
            &inline_buffer, &styles, id_counter,
            child_available_width, &link_href_from_parent,
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
    if source.display.is_some() { dest.display = source.display.clone(); }
    if source.position.is_some() { dest.position = source.position.clone(); }
    if source.width.is_some() { dest.width = source.width.clone(); }
    if source.height.is_some() { dest.height = source.height.clone(); }
    if source.background_color.is_some() { dest.background_color = source.background_color.clone(); }
    if source.color.is_some() { dest.color = source.color.clone(); }
    if source.font_size.is_some() { dest.font_size = source.font_size.clone(); }
    if source.font_family.is_some() { dest.font_family = source.font_family.clone(); }
    if source.font_weight.is_some() { dest.font_weight = source.font_weight.clone(); }
    if source.flex_direction.is_some() { dest.flex_direction = source.flex_direction.clone(); }
    if source.justify_content.is_some() { dest.justify_content = source.justify_content.clone(); }
    if source.align_items.is_some() { dest.align_items = source.align_items.clone(); }
    if source.gap.is_some() { dest.gap = source.gap.clone(); }
    if source.overflow.is_some() { dest.overflow = source.overflow.clone(); }
    if source.visibility.is_some() { dest.visibility = source.visibility.clone(); }
    if source.opacity.is_some() { dest.opacity = source.opacity; }
    if source.z_index.is_some() { dest.z_index = source.z_index; }
    if source.margin.is_some() { dest.margin = source.margin.clone(); }
    if source.padding.is_some() { dest.padding = source.padding.clone(); }
    if source.border.is_some() { dest.border = source.border.clone(); }
    for (k, v) in &source.other {
        dest.other.insert(k.clone(), v.clone());
    }
}

/// Check if a tag is a block-level element
fn is_block_element(tag: &str) -> bool {
    matches!(tag,
        "html" | "body" | "div" | "p" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6"
        | "section" | "article" | "header" | "footer" | "main" | "nav" | "aside"
        | "blockquote" | "pre" | "hr" | "ul" | "ol" | "li" | "table" | "form"
        | "figure" | "figcaption" | "details" | "summary" | "dialog" | "address"
        | "fieldset" | "legend"
    )
}

/// Check if a tag is an inline-level element (flows with text).
fn is_inline_element(tag: &str) -> bool {
    matches!(tag,
        "a" | "span" | "strong" | "b" | "em" | "i" | "u" | "s" | "strike"
        | "code" | "kbd" | "samp" | "var" | "mark" | "small" | "big" | "sub" | "sup"
        | "abbr" | "cite" | "dfn" | "q" | "time" | "data" | "ruby" | "bdo" | "bdi"
        | "wbr" | "del" | "ins" | "label"
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

    map.insert(element.id.clone(), VisualData {
        text_content,
        link_href,
        img_src,
        tag: element.tag.clone(),
    });

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
        assert!(texts.iter().any(|t| t.contains("Example Domain")), "Should contain 'Example Domain'");
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
            if el.tag == tag { return Some(el); }
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
            if el.tag == tag { return Some(el); }
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
            if el.text_content.is_some() { return Some(el); }
            for child in &el.children {
                if let Some(found) = find_text(child) {
                    return Some(found);
                }
            }
            None
        }

        let text_el = find_text(&result.root);
        assert!(text_el.is_some(), "Should find element with text_content");
        assert!(text_el.unwrap().text_content.as_ref().unwrap().contains("Hello world"));
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
        fn find_text_elements(el: &StyledElement, results: &mut Vec<(String, Option<String>, Option<String>)>) {
            if let Some(ref text) = el.text_content {
                if !text.trim().is_empty() {
                    results.push((text.clone(), el.styles.color.clone(), el.styles.background_color.clone()));
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
        let red_el = text_elements.iter().find(|(t, _, _)| t.contains("Red text")).expect("Should find 'Red text'");
        assert!(red_el.1.is_some(), "Red text should have a color set: {:?}", red_el);

        // Blue bg: background-color doesn't inherit to #text children, it stays on parent
        // So the text element won't have it. This is correct CSS behavior.

        // Inherited color should have color from body
        let inherited_el = text_elements.iter().find(|(t, _, _)| t.contains("Inherited")).expect("Should find 'Inherited color'");
        assert!(inherited_el.1.is_some(), "Inherited color should have color from body: {:?}", inherited_el);
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

        fn find_text_elements(el: &StyledElement, results: &mut Vec<(String, Option<String>, Option<String>)>) {
            if let Some(ref text) = el.text_content {
                if !text.trim().is_empty() {
                    results.push((text.clone(), el.styles.color.clone(), el.styles.background_color.clone()));
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
        let body_el = text_elements.iter().find(|(t, _, _)| t.contains("Body text")).expect("Should find 'Body text'");
        assert!(body_el.1.is_some(), "Body text should have color resolved from var(): {:?}", body_el);
        assert!(!body_el.1.as_ref().unwrap().contains("var("), "Color should be resolved, not contain var(): {:?}", body_el.1);

        // Tailwind black — rgb(0 0 0 / var(--tw-text-opacity, 1))
        let tw_el = text_elements.iter().find(|(t, _, _)| t.contains("Tailwind")).expect("Should find 'Tailwind black'");
        eprintln!("  Tailwind black resolved color: {:?}", tw_el.1);
        assert!(tw_el.1.is_some(), "Tailwind text should have a color: {:?}", tw_el);
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
        assert_eq!(navs[0].1.as_deref(), Some("none"),
            "First nav should be display:none at 1280px (dn-l media query)");
    }
}
