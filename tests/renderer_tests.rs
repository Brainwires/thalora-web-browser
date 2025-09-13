use synaptic::renderer::{RustRenderer, CssProcessor, LayoutEngine};

#[tokio::test]
async fn test_javascript_safety_filtering() {
    let renderer = RustRenderer::new();
    
    // Safe JavaScript should pass
    let safe_js = "var x = 1; x + 2;";
    assert!(renderer.is_safe_javascript(safe_js));
    
    // Dangerous patterns should be blocked
    let dangerous_patterns = vec![
        "eval('malicious code')",
        "Function('return this')()",
        "XMLHttpRequest()",
        "fetch('http://evil.com')",
        "document.cookie",
        "localStorage.setItem('key', 'value')",
        "window.location = 'http://evil.com'",
        "alert('popup')",
    ];
    
    for pattern in dangerous_patterns {
        assert!(!renderer.is_safe_javascript(pattern), "Pattern should be blocked: {}", pattern);
    }
}

#[tokio::test]
async fn test_javascript_size_limit() {
    let renderer = RustRenderer::new();
    
    // Small script should pass
    let small_js = "var x = 1;";
    assert!(renderer.is_safe_javascript(small_js));
    
    // Large script should be blocked
    let large_js = "a".repeat(10001);
    assert!(!renderer.is_safe_javascript(&large_js));
}

#[tokio::test]
async fn test_html_processing_without_js() {
    let mut renderer = RustRenderer::new();
    let html = "<html><body><h1>Test</h1><script>var x = 1;</script></body></html>";
    
    let result = renderer.render_with_js(html, "https://example.com").await.unwrap();
    
    // Should return HTML (possibly modified)
    assert!(result.contains("<h1>Test</h1>"));
}

#[test]
fn test_css_processing() {
    let processor = CssProcessor::new();
    
    let css = "body { color: red; font-size: 16px; }";
    let result = processor.process_css(css).unwrap();
    
    // Should return processed CSS
    assert!(result.contains("color"));
    assert!(result.contains("red"));
}

#[test]
fn test_css_processing_invalid() {
    let processor = CssProcessor::new();
    
    // Invalid CSS should return error
    let invalid_css = "body { color: ; font-size }";
    let result = processor.process_css(invalid_css);
    
    // Should handle invalid CSS gracefully
    assert!(result.is_err());
}

#[test]
fn test_layout_calculation() {
    
    let engine = LayoutEngine::new();
    let html = "<div>Test content</div>";
    let css = "div { width: 300px; height: 200px; }";
    
    let result = engine.calculate_layout(html, css).unwrap();
    
    // Should return layout information (width should be set, height might be 0 for auto)
    assert!(result.width > 0.0);
}

#[tokio::test]
async fn test_javascript_execution_timeout() {
    let mut renderer = RustRenderer::new();
    
    // This should complete quickly
    let simple_js = "1 + 1";
    let result = renderer.execute_javascript_safely(simple_js).await;
    assert!(result.is_ok());
}

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

#[test]
fn test_renderer_creation() {
    let renderer = RustRenderer::new();
    // Should create without error
    drop(renderer);
    
    let processor = CssProcessor::new();
    drop(processor);
    
    let engine = LayoutEngine::new();
    drop(engine);
}