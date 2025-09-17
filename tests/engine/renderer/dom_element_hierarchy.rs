use thalora::renderer::{RustRenderer, CssProcessor, LayoutEngine};

#[tokio::test]
async fn test_dom_element_hierarchy() {
    let mut renderer = RustRenderer::new();

    // Test parent-child relationships
    let js_code = r#"
        var parent = document.createElement('div');
        var child = document.createElement('span');

        // Test appendChild
        parent.appendChild(child);
        if (child.parentNode !== parent) throw new Error('parentNode not set correctly');
        if (child.parentElement !== parent) throw new Error('parentElement not set correctly');
        if (parent.childNodes.length !== 1) throw new Error('childNodes length incorrect');
        if (parent.children.length !== 1) throw new Error('children length incorrect');

        // Test removeChild
        parent.removeChild(child);
        if (child.parentNode !== null) throw new Error('parentNode not cleared');
        if (parent.childNodes.length !== 0) throw new Error('child not removed');

        'hierarchy_ok'
    "#;

    let result = renderer.execute_javascript_safely(js_code).await;
    assert!(result.is_ok());
}
