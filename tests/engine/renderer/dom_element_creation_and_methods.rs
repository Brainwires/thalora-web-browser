use thalora::renderer::{RustRenderer, CssProcessor, LayoutEngine};

#[tokio::test]
async fn test_dom_element_creation_and_methods() {
    let mut renderer = RustRenderer::new();

    // Test element creation and methods
    let js_code = r#"
        var div = document.createElement('div');
        if (!div) throw new Error('createElement failed');
        if (div.tagName !== 'DIV') throw new Error('tagName incorrect');
        if (div.nodeType !== 1) throw new Error('nodeType incorrect');

        // Test attributes
        div.setAttribute('id', 'test-id');
        if (div.getAttribute('id') !== 'test-id') throw new Error('setAttribute/getAttribute failed');
        if (div.id !== 'test-id') throw new Error('id property not synced');

        div.setAttribute('class', 'test-class');
        if (!div.hasAttribute('class')) throw new Error('hasAttribute failed');
        if (div.className !== 'test-class') throw new Error('className not synced');

        // Test classList
        div.classList.add('another-class');
        if (!div.classList.contains('another-class')) throw new Error('classList.add/contains failed');

        'element_methods_ok'
    "#;

    let result = renderer.execute_javascript_safely(js_code).await;
    assert!(result.is_ok());
}
