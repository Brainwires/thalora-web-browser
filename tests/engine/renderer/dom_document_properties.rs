use thalora::renderer::{RustRenderer, CssProcessor, LayoutEngine};

#[tokio::test]
async fn test_dom_document_properties() {
    let mut renderer = RustRenderer::new();

    // Test document properties
    let js_code = r#"
        if (typeof document === 'undefined') throw new Error('document not defined');
        if (!document.title && document.title !== '') throw new Error('title not available');
        if (!document.URL) throw new Error('URL not available');
        if (!document.domain) throw new Error('domain not available');
        'document_properties_ok'
    "#;

    let result = renderer.execute_javascript_safely(js_code).await;
    assert!(result.is_ok());
}
