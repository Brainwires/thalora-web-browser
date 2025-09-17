use thalora::renderer::{RustRenderer, CssProcessor, LayoutEngine};

#[tokio::test]
async fn test_dom_document_methods() {
    let mut renderer = RustRenderer::new();

    // Test document query methods
    let js_code = r#"
        // Test document methods exist
        if (typeof document.getElementById !== 'function') throw new Error('getElementById not available');
        if (typeof document.getElementsByTagName !== 'function') throw new Error('getElementsByTagName not available');
        if (typeof document.querySelector !== 'function') throw new Error('querySelector not available');
        if (typeof document.querySelectorAll !== 'function') throw new Error('querySelectorAll not available');

        // Test createTextNode and createDocumentFragment
        var textNode = document.createTextNode('test text');
        if (!textNode || textNode.nodeType !== 3) throw new Error('createTextNode failed');
        if (textNode.textContent !== 'test text') throw new Error('textContent incorrect');

        var fragment = document.createDocumentFragment();
        if (!fragment || fragment.nodeType !== 11) throw new Error('createDocumentFragment failed');

        'document_methods_ok'
    "#;

    let result = renderer.execute_javascript_safely(js_code).await;
    assert!(result.is_ok());
}
