use thalora::renderer::{RustRenderer, CssProcessor, LayoutEngine};

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
