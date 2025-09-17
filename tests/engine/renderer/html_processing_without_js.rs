use thalora::renderer::{RustRenderer, CssProcessor, LayoutEngine};

#[tokio::test]
async fn test_html_processing_without_js() {
    let mut renderer = RustRenderer::new();
    let html = "<html><body><h1>Test</h1><script>var x = 1;</script></body></html>";
    
    let result = renderer.render_with_js(html, "https://example.com").await.unwrap();
    
    // Should return HTML (possibly modified)
    assert!(result.contains("<h1>Test</h1>"));
}
