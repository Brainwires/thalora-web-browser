//! Layout Bridge
//!
//! Connects the taffy layout engine output to the Boa DOM by computing layout
//! geometry for all elements and storing the results as a HashMap keyed by
//! unique CSS selector paths. These paths are deterministic for a given HTML
//! document, allowing the DOM layer to look up geometry when elements are
//! created via querySelector/getElementById.

use super::layout::{BoxModelSides, ElementLayout, LayoutResult};
use std::collections::HashMap;
use thalora_browser_apis::dom::document::LayoutRect;

/// Flatten a `LayoutResult` tree into a HashMap keyed by CSS selector path.
///
/// The selector path for each element is computed from the layout tree structure
/// (tag names and child indices), producing paths like:
///   "html>body:nth-child(1)>div:nth-child(1)>p:nth-child(2)"
///
/// These same paths can be computed from a scraper parse tree for the same HTML,
/// allowing the DOM layer to look up geometry by path.
pub fn flatten_layout_to_rects(layout: &LayoutResult) -> HashMap<String, LayoutRect> {
    let mut map = HashMap::new();
    for element in &layout.elements {
        if element.tag.starts_with('#') || element.tag.starts_with(':') {
            continue;
        }
        // Root element (e.g., "html") has no nth-child
        let root_path = element.tag.to_lowercase();
        map.insert(root_path.clone(), element_to_rect(element));
        flatten_children(&mut map, element, &root_path);
    }
    map
}

/// Recursively flatten children, tracking nth-child per tag name
fn flatten_children(
    map: &mut HashMap<String, LayoutRect>,
    parent: &ElementLayout,
    parent_path: &str,
) {
    // Count occurrences of each tag name among non-text, non-pseudo children
    let mut tag_counts: HashMap<String, u32> = HashMap::new();

    for child in &parent.children {
        if child.tag.starts_with('#') || child.tag.starts_with(':') {
            continue;
        }

        let tag_lower = child.tag.to_lowercase();
        let count = tag_counts.entry(tag_lower.clone()).or_insert(0);
        *count += 1;

        let child_path = format!("{}>{}:nth-child({})", parent_path, tag_lower, *count);
        map.insert(child_path.clone(), element_to_rect(child));
        flatten_children(map, child, &child_path);
    }
}

fn element_to_rect(element: &ElementLayout) -> LayoutRect {
    let (content_x, content_y, content_width, content_height) =
        if let Some(ref cb) = element.content_box {
            (cb.x, cb.y, cb.width, cb.height)
        } else {
            // Approximate content box from padding/border
            let pt = element.padding.as_ref().map_or(0.0, |p| p.top);
            let pr = element.padding.as_ref().map_or(0.0, |p| p.right);
            let pb = element.padding.as_ref().map_or(0.0, |p| p.bottom);
            let pl = element.padding.as_ref().map_or(0.0, |p| p.left);
            let bt = element.border_sides.as_ref().map_or(0.0, |b| b.top);
            let br = element.border_sides.as_ref().map_or(0.0, |b| b.right);
            let bb = element.border_sides.as_ref().map_or(0.0, |b| b.bottom);
            let bl = element.border_sides.as_ref().map_or(0.0, |b| b.left);
            (
                element.x + pl + bl,
                element.y + pt + bt,
                (element.width - pl - pr - bl - br).max(0.0),
                (element.height - pt - pb - bt - bb).max(0.0),
            )
        };

    let sides = |s: &Option<BoxModelSides>| -> (f64, f64, f64, f64) {
        s.as_ref()
            .map_or((0.0, 0.0, 0.0, 0.0), |s| (s.top, s.right, s.bottom, s.left))
    };

    let (pt, pr, pb, pl) = sides(&element.padding);
    let (bt, br, bb, bl) = sides(&element.border_sides);
    let (mt, mr, mb, ml) = sides(&element.margin);

    LayoutRect {
        x: element.x,
        y: element.y,
        width: element.width,
        height: element.height,
        content_x,
        content_y,
        content_width,
        content_height,
        padding_top: pt,
        padding_right: pr,
        padding_bottom: pb,
        padding_left: pl,
        border_top: bt,
        border_right: br,
        border_bottom: bb,
        border_left: bl,
        margin_top: mt,
        margin_right: mr,
        margin_bottom: mb,
        margin_left: ml,
    }
}

/// Compute the CSS selector path for a scraper ElementRef.
///
/// This produces the same path format as `flatten_layout_to_rects`, allowing
/// lookup of layout geometry by path when creating DOM elements from HTML.
///
/// Returns paths like: "html>body:nth-child(1)>div:nth-child(1)>p:nth-child(2)"
pub fn css_path_for_element(element_ref: &scraper::ElementRef) -> String {
    let mut parts: Vec<String> = Vec::new();
    let mut current = Some(*element_ref);

    while let Some(el) = current {
        let tag = el.value().name().to_lowercase();

        // Count this element's position among same-tag siblings
        let nth = if let Some(parent) = el.parent() {
            let mut count = 0u32;
            for sibling in parent.children() {
                if let Some(sibling_el) = scraper::ElementRef::wrap(sibling)
                    && sibling_el.value().name().eq_ignore_ascii_case(&tag)
                {
                    count += 1;
                    if sibling_el == el {
                        break;
                    }
                }
            }
            count
        } else {
            1
        };

        // Root element (html) doesn't get nth-child
        if el.parent().and_then(scraper::ElementRef::wrap).is_none() {
            parts.push(tag);
        } else {
            parts.push(format!("{}:nth-child({})", tag, nth));
        }

        current = el.parent().and_then(scraper::ElementRef::wrap);
    }

    parts.reverse();
    parts.join(">")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_to_rect_basic() {
        let el = ElementLayout {
            id: "e0".to_string(),
            tag: "div".to_string(),
            x: 10.0,
            y: 20.0,
            width: 200.0,
            height: 100.0,
            content_box: None,
            children: vec![],
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
        };

        let rect = element_to_rect(&el);
        assert_eq!(rect.x, 10.0);
        assert_eq!(rect.y, 20.0);
        assert_eq!(rect.width, 200.0);
        assert_eq!(rect.height, 100.0);
    }

    #[test]
    fn test_css_path_for_element() {
        let html = r#"<html><body><div><p>Hello</p><p>World</p></div></body></html>"#;
        let document = scraper::Html::parse_document(html);
        let selector = scraper::Selector::parse("p").unwrap();
        let elements: Vec<_> = document.select(&selector).collect();

        let path1 = css_path_for_element(&elements[0]);
        let path2 = css_path_for_element(&elements[1]);

        assert!(path1.contains("p:nth-child(1)"), "path1={}", path1);
        assert!(path2.contains("p:nth-child(2)"), "path2={}", path2);
        assert_ne!(path1, path2);
    }
}
