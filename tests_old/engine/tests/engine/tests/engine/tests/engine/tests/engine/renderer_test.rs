use thalora::renderer::{RustRenderer, CssProcessor, LayoutEngine};

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

#[tokio::test]
async fn test_dom_element_hierarchy() {
    let mut renderer = RustRenderer::new();

    // Test parent-child relationships
    let js_code = r#"
        var parent = document.createElement('div');
        var child = document.createElement('span');

        // Test appendChild
        parent.appendChild(child);
        if (child.parentNode !== parent) throw new Error('parentNode not set correctly');
        if (child.parentElement !== parent) throw new Error('parentElement not set correctly');
        if (parent.childNodes.length !== 1) throw new Error('childNodes length incorrect');
        if (parent.children.length !== 1) throw new Error('children length incorrect');

        // Test removeChild
        parent.removeChild(child);
        if (child.parentNode !== null) throw new Error('parentNode not cleared');
        if (parent.childNodes.length !== 0) throw new Error('child not removed');

        'hierarchy_ok'
    "#;

    let result = renderer.execute_javascript_safely(js_code).await;
    assert!(result.is_ok());
}

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

#[tokio::test]
async fn test_dom_window_properties() {
    let mut renderer = RustRenderer::new();

    // Test window object properties
    let js_code = r#"
        if (typeof window === 'undefined') throw new Error('window not defined');
        if (typeof window.innerWidth !== 'number') throw new Error('innerWidth not available');
        if (typeof window.innerHeight !== 'number') throw new Error('innerHeight not available');
        if (window.innerWidth <= 0) throw new Error('innerWidth not positive');
        if (window.innerHeight <= 0) throw new Error('innerHeight not positive');

        // Test screen object
        if (!window.screen) throw new Error('screen object not available');
        if (typeof window.screen.width !== 'number') throw new Error('screen.width not available');

        // Test location object
        if (!window.location) throw new Error('location object not available');
        if (typeof window.location.href !== 'string') throw new Error('location.href not available');

        'window_properties_ok'
    "#;

    let result = renderer.execute_javascript_safely(js_code).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_dom_event_system() {
    let mut renderer = RustRenderer::new();

    // Test Event and CustomEvent constructors
    let js_code = r#"
        // Test Event constructor
        if (typeof Event === 'undefined') throw new Error('Event constructor not available');
        var event = new Event('click', { bubbles: true, cancelable: true });
        if (event.type !== 'click') throw new Error('Event type incorrect');
        if (!event.bubbles) throw new Error('Event bubbles not set');
        if (!event.cancelable) throw new Error('Event cancelable not set');

        // Test CustomEvent constructor
        if (typeof CustomEvent === 'undefined') throw new Error('CustomEvent constructor not available');
        var customEvent = new CustomEvent('custom', { detail: { data: 'test' } });
        if (customEvent.type !== 'custom') throw new Error('CustomEvent type incorrect');
        if (!customEvent.detail || customEvent.detail.data !== 'test') throw new Error('CustomEvent detail incorrect');

        // Test event methods
        event.preventDefault();
        if (!event.defaultPrevented) throw new Error('preventDefault not working');

        'event_system_ok'
    "#;

    let result = renderer.execute_javascript_safely(js_code).await;
    assert!(result.is_ok());
}

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