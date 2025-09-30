use thalora::renderer::{RustRenderer, CssProcessor, LayoutEngine};

#[tokio::test]
async fn test_dom_polyfill() {
    let mut renderer = RustRenderer::new();

    // Test basic DOM operations
    let js_code = r#"
        var div = document.createElement('div');
        div.innerHTML = 'test content';
        'success'
    "#;

    let result = renderer.execute_javascript_safely(js_code).await;
    assert!(result.is_ok());
}
