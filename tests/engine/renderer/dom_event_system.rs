use thalora::renderer::{RustRenderer, CssProcessor, LayoutEngine};

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
