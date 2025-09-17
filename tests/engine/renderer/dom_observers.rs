use thalora::renderer::{RustRenderer, CssProcessor, LayoutEngine};

#[tokio::test]
async fn test_dom_observers() {
    let mut renderer = RustRenderer::new();

    // Test Observer APIs
    let js_code = r#"
        // Test MutationObserver
        if (typeof MutationObserver === 'undefined') throw new Error('MutationObserver not available');
        var mutationCallback = function() {};
        var mutationObserver = new MutationObserver(mutationCallback);
        if (typeof mutationObserver.observe !== 'function') throw new Error('MutationObserver.observe not available');
        if (typeof mutationObserver.disconnect !== 'function') throw new Error('MutationObserver.disconnect not available');

        // Test IntersectionObserver
        if (typeof IntersectionObserver === 'undefined') throw new Error('IntersectionObserver not available');
        var intersectionCallback = function() {};
        var intersectionObserver = new IntersectionObserver(intersectionCallback);
        if (typeof intersectionObserver.observe !== 'function') throw new Error('IntersectionObserver.observe not available');

        // Test ResizeObserver
        if (typeof ResizeObserver === 'undefined') throw new Error('ResizeObserver not available');
        var resizeCallback = function() {};
        var resizeObserver = new ResizeObserver(resizeCallback);
        if (typeof resizeObserver.observe !== 'function') throw new Error('ResizeObserver.observe not available');

        'observers_ok'
    "#;

    let result = renderer.execute_javascript_safely(js_code).await;
    assert!(result.is_ok());
}
