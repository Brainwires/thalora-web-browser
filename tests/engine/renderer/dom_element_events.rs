use thalora::renderer::{RustRenderer, CssProcessor, LayoutEngine};

#[tokio::test]
async fn test_dom_element_events() {
    let mut renderer = RustRenderer::new();

    // Test element event handling
    let js_code = r#"
        var div = document.createElement('div');

        // Test addEventListener
        var eventFired = false;
        div.addEventListener('click', function() {
            eventFired = true;
        });

        // Test event properties exist
        if (typeof div.onclick === 'undefined') throw new Error('onclick property not available');
        if (typeof div.onload === 'undefined') throw new Error('onload property not available');
        if (typeof div.onerror === 'undefined') throw new Error('onerror property not available');

        // Test getBoundingClientRect
        var rect = div.getBoundingClientRect();
        if (typeof rect.width !== 'number') throw new Error('getBoundingClientRect width not available');
        if (typeof rect.height !== 'number') throw new Error('getBoundingClientRect height not available');

        // Test other element methods
        if (typeof div.focus !== 'function') throw new Error('focus method not available');
        if (typeof div.blur !== 'function') throw new Error('blur method not available');
        if (typeof div.click !== 'function') throw new Error('click method not available');

        'element_events_ok'
    "#;

    let result = renderer.execute_javascript_safely(js_code).await;
    assert!(result.is_ok());
}
