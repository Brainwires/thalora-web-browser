//! Integration tests for layout geometry bridge
//!
//! Verifies that getBoundingClientRect() and offset/client properties
//! return real values computed by the taffy layout engine.

use thalora::engine::renderer::{compute_page_layout, flatten_layout_to_rects};
use thalora::engine::renderer::layout_bridge::css_path_for_element;

#[test]
fn test_flatten_layout_produces_rects() {
    let html = r#"<html><head></head><body><div style="width:200px;height:100px"><p>Hello</p></div></body></html>"#;
    let layout = compute_page_layout(html, 1024.0, 768.0).unwrap();
    let rects = flatten_layout_to_rects(&layout);

    // Should have rects for html, body, div, p at minimum
    assert!(!rects.is_empty(), "Layout rects should not be empty");

    // Check that the "html" root element exists
    assert!(rects.contains_key("html"), "Should have rect for html root. Keys: {:?}", rects.keys().collect::<Vec<_>>());
}

#[test]
fn test_css_path_matches_layout_path() {
    let html = r#"<html><head></head><body><div style="width:200px;height:100px"><p>Hello</p><p>World</p></div></body></html>"#;

    // Compute layout and get rects
    let layout = compute_page_layout(html, 1024.0, 768.0).unwrap();
    let rects = flatten_layout_to_rects(&layout);

    // Parse the same HTML with scraper and compute CSS paths
    let document = scraper::Html::parse_document(html);
    let p_selector = scraper::Selector::parse("p").unwrap();
    let p_elements: Vec<_> = document.select(&p_selector).collect();

    assert_eq!(p_elements.len(), 2, "Should find 2 <p> elements");

    let path1 = css_path_for_element(&p_elements[0]);
    let path2 = css_path_for_element(&p_elements[1]);

    // The paths should be different
    assert_ne!(path1, path2, "Two different <p> elements should have different paths");

    // Print available keys for debugging
    eprintln!("Layout rect keys: {:?}", rects.keys().collect::<Vec<_>>());
    eprintln!("CSS path for p[0]: {}", path1);
    eprintln!("CSS path for p[1]: {}", path2);
}

#[test]
fn test_div_with_explicit_dimensions() {
    let html = r#"<html><head></head><body><div id="target" style="width:300px;height:150px">Content</div></body></html>"#;
    let layout = compute_page_layout(html, 1024.0, 768.0).unwrap();
    let rects = flatten_layout_to_rects(&layout);

    // Find the div in the rects - should be body's first child div
    let div_rect = rects.iter()
        .find(|(k, _)| k.ends_with("div:nth-child(1)") || k.ends_with(">div:nth-child(1)"))
        .map(|(_, v)| v);

    if let Some(rect) = div_rect {
        // The div should have approximately the dimensions we set
        assert!(rect.width >= 299.0 && rect.width <= 301.0,
            "div width should be ~300px, got {}", rect.width);
        assert!(rect.height >= 149.0 && rect.height <= 151.0,
            "div height should be ~150px, got {}", rect.height);
    } else {
        // Print available keys for debugging
        eprintln!("Available rect keys: {:?}", rects.keys().collect::<Vec<_>>());
        panic!("Could not find div rect in layout results");
    }
}

#[test]
fn test_nested_elements_have_positions() {
    let html = r#"<html><head></head><body>
        <div style="width:400px;padding:10px">
            <div style="width:200px;height:50px">Inner</div>
        </div>
    </body></html>"#;
    let layout = compute_page_layout(html, 1024.0, 768.0).unwrap();
    let rects = flatten_layout_to_rects(&layout);

    // There should be multiple rects
    assert!(rects.len() >= 3, "Should have at least 3 rects (html, body, outer div, inner div), got {}", rects.len());

    // Print all keys and values for debugging
    for (key, rect) in &rects {
        eprintln!("  {} => ({}, {}, {}x{})", key, rect.x, rect.y, rect.width, rect.height);
    }
}
