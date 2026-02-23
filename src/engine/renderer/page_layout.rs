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

/// User-agent default styles for block-level elements.
/// These mirror the CSS 2.1 spec defaults that browsers apply.
fn apply_ua_defaults(tag: &str, styles: &mut ComputedStyles) {
    match tag {
        "html" => {
            styles.display = Some("block".to_string());
        }
        "body" => {
            styles.display = Some("block".to_string());
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
            styles.display = Some("block".to_string());
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
            styles.display = Some("block".to_string());
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
            styles.display = Some("block".to_string());
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
            styles.display = Some("block".to_string());
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
            styles.display = Some("block".to_string());
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
            styles.display = Some("block".to_string());
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
            styles.display = Some("block".to_string());
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
            styles.display = Some("block".to_string());
        }
        "blockquote" => {
            styles.display = Some("block".to_string());
            if styles.margin.is_none() {
                styles.margin = Some(BoxModel {
                    top: "16px".to_string(), right: "40px".to_string(),
                    bottom: "16px".to_string(), left: "40px".to_string(),
                });
            }
        }
        "pre" => {
            styles.display = Some("block".to_string());
            if styles.font_family.is_none() { styles.font_family = Some("monospace".to_string()); }
            styles.other.entry("white-space".to_string()).or_insert_with(|| "pre".to_string());
            if styles.margin.is_none() {
                styles.margin = Some(BoxModel {
                    top: "16px".to_string(), right: "0px".to_string(),
                    bottom: "16px".to_string(), left: "0px".to_string(),
                });
            }
        }
        "code" => {
            if styles.font_family.is_none() { styles.font_family = Some("monospace".to_string()); }
        }
        "hr" => {
            styles.display = Some("block".to_string());
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
            styles.display = Some("block".to_string());
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
            styles.display = Some("list-item".to_string());
            styles.other.entry("list-style-type".to_string()).or_insert_with(|| "disc".to_string());
        }
        "a" => {
            if styles.color.is_none() { styles.color = Some("rgb(100, 149, 237)".to_string()); }
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
            styles.display = Some("table".to_string());
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
            styles.display = Some("table-row".to_string());
        }
        "td" => {
            styles.display = Some("table-cell".to_string());
            if styles.padding.is_none() {
                styles.padding = Some(BoxModel {
                    top: "4px".to_string(), right: "8px".to_string(),
                    bottom: "4px".to_string(), left: "8px".to_string(),
                });
            }
        }
        "th" => {
            styles.display = Some("table-cell".to_string());
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

/// Compute the full page layout from raw HTML.
///
/// This is the main bridge function:
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

    // Build children
    let mut children = Vec::new();
    for child in element_ref.children() {
        match child.value() {
            Node::Element(_) => {
                if let Some(child_el_ref) = ElementRef::wrap(child) {
                    let child_tag = child_el_ref.value().name().to_lowercase();
                    if SKIP_TAGS.contains(&child_tag.as_str()) {
                        continue;
                    }

                    let child_layout = build_layout_tree_from_dom(
                        &child_el_ref,
                        css_processor,
                        id_counter,
                        viewport_w,
                        child_available_width,
                        Some(&styles),
                    );

                    // Skip display:none
                    if child_layout.styles.display.as_deref() == Some("none") {
                        continue;
                    }
                    if child_layout.styles.visibility.as_deref() == Some("hidden") {
                        continue;
                    }

                    children.push(child_layout);
                }
            }
            Node::Text(text) => {
                let raw_text = text.text.as_ref();
                // Skip whitespace-only text nodes (unless parent preserves whitespace)
                let ws = styles.other.get("white-space").map(|s| s.as_str()).unwrap_or("normal");
                if ws == "normal" && raw_text.trim().is_empty() {
                    continue;
                }
                // Collapse whitespace like browsers do: newlines, tabs, and runs of
                // spaces all become a single space in normal white-space mode.
                let text_str: String = if ws == "normal" || ws == "nowrap" {
                    raw_text.split_whitespace().collect::<Vec<_>>().join(" ")
                } else {
                    raw_text.to_string()
                };

                // Text nodes — measure with real font shaping using per-line data
                let font_size = styles.font_size.as_ref()
                    .and_then(|s| super::layout::resolve_css_length(s, 16.0))
                    .unwrap_or(16.0);
                let line_height = styles.other.get("line-height")
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(1.4);
                let font_family = styles.font_family.as_deref().unwrap_or("sans-serif");
                let font_weight = styles.font_weight.as_deref();
                let container_w = child_available_width.max(50.0);

                let lines_result = super::text_measure::measure_text_lines(
                    &text_str,
                    font_family,
                    font_size as f32,
                    font_weight,
                    Some(container_w as f32),
                    line_height as f32,
                );

                // Helper: build inherited text styles from parent
                let make_text_styles = |text_content: &str, line_h: f64| -> ComputedStyles {
                    let mut ts = ComputedStyles::default();
                    ts.font_size = styles.font_size.clone();
                    ts.font_family = styles.font_family.clone();
                    ts.font_weight = styles.font_weight.clone();
                    ts.color = styles.color.clone();
                    if let Some(fs) = styles.other.get("font-style") {
                        ts.other.insert("font-style".to_string(), fs.clone());
                    }
                    if let Some(lh) = styles.other.get("line-height") {
                        ts.other.insert("line-height".to_string(), lh.clone());
                    }
                    if let Some(ws_val) = styles.other.get("white-space") {
                        ts.other.insert("white-space".to_string(), ws_val.clone());
                    }
                    ts.other.insert("min-height".to_string(), format!("{}px", line_h));
                    ts.display = Some("block".to_string());
                    ts.other.insert("__text_content".to_string(), text_content.to_string());
                    ts.other.insert("__pre_split_line".to_string(), "true".to_string());
                    ts
                };

                // Link href for <a> elements
                let link_href = if tag == "a" {
                    el.attr("href").map(|h| h.to_string())
                } else {
                    None
                };

                if lines_result.lines.len() <= 1 {
                    // Single line: keep current behavior with pre_split_line marker
                    let text_id = format!("t{}", *id_counter);
                    *id_counter += 1;

                    let line = lines_result.lines.first();
                    let line_h = line.map(|l| l.height as f64).unwrap_or(font_size * line_height);

                    let mut text_styles = make_text_styles(&text_str, line_h);

                    if let Some(ref href) = link_href {
                        text_styles.other.insert("__link_href".to_string(), href.clone());
                    }

                    let text_layout = LayoutElement {
                        id: text_id,
                        tag: "#text".to_string(),
                        styles: text_styles,
                        children: Vec::new(),
                    };

                    children.push(text_layout);
                } else {
                    // Multiple lines: create a #text parent container with N #text-line children
                    let parent_id = format!("t{}", *id_counter);
                    *id_counter += 1;

                    let mut parent_styles = ComputedStyles::default();
                    parent_styles.font_size = styles.font_size.clone();
                    parent_styles.font_family = styles.font_family.clone();
                    parent_styles.font_weight = styles.font_weight.clone();
                    parent_styles.color = styles.color.clone();
                    if let Some(fs) = styles.other.get("font-style") {
                        parent_styles.other.insert("font-style".to_string(), fs.clone());
                    }
                    if let Some(lh) = styles.other.get("line-height") {
                        parent_styles.other.insert("line-height".to_string(), lh.clone());
                    }
                    if let Some(ws_val) = styles.other.get("white-space") {
                        parent_styles.other.insert("white-space".to_string(), ws_val.clone());
                    }
                    parent_styles.display = Some("block".to_string());
                    parent_styles.other.insert("min-height".to_string(), format!("{}px", lines_result.total_height));

                    if let Some(ref href) = link_href {
                        parent_styles.other.insert("__link_href".to_string(), href.clone());
                    }

                    let mut line_children = Vec::new();
                    for line in &lines_result.lines {
                        let line_id = format!("t{}", *id_counter);
                        *id_counter += 1;

                        let mut line_styles = make_text_styles(&line.text, line.height as f64);

                        if let Some(ref href) = link_href {
                            line_styles.other.insert("__link_href".to_string(), href.clone());
                        }

                        line_children.push(LayoutElement {
                            id: line_id,
                            tag: "#text-line".to_string(),
                            styles: line_styles,
                            children: Vec::new(),
                        });
                    }

                    let text_layout = LayoutElement {
                        id: parent_id,
                        tag: "#text".to_string(),
                        styles: parent_styles,
                        children: line_children,
                    };

                    children.push(text_layout);
                }
            }
            _ => {}
        }
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

/// Visual data extracted from LayoutElement tree for post-processing
struct VisualData {
    text_content: Option<String>,
    link_href: Option<String>,
    img_src: Option<String>,
    tag: String,
    pre_split_line: bool,
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
    let pre_split_line = element.styles.other.get("__pre_split_line")
        .map(|v| v == "true")
        .unwrap_or(false);

    map.insert(element.id.clone(), VisualData {
        text_content,
        link_href,
        img_src,
        tag: element.tag.clone(),
        pre_split_line,
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
        layout.pre_split_line = visual.pre_split_line;
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
}
