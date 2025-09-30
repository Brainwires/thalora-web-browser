use thalora::renderer::{RustRenderer, CssProcessor, LayoutEngine};

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
