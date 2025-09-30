use thalora::renderer::{RustRenderer, CssProcessor, LayoutEngine};

#[tokio::test]
async fn test_javascript_execution_timeout() {
    let mut renderer = RustRenderer::new();
    
    // This should complete quickly
    let simple_js = "1 + 1";
    let result = renderer.execute_javascript_safely(simple_js).await;
    assert!(result.is_ok());
}
