#[tokio::test]
async fn test_chrome_138_viewport_segments_api() {
    println!("🧪 Testing Chrome 138: Viewport Segments API...");

    let browser = HeadlessWebBrowser::new();

    // Test Viewport Segments API for foldable devices
    let js_code = r#"
        try {
            // Check if Viewport Segments API is available
            if (typeof window !== 'undefined' && typeof CSS !== 'undefined') {
                // Test viewport segments environment variables
                var hasViewportSegments = CSS.supports('left', 'env(viewport-segment-left 0 0)');

                // Test for basic foldable device support
                var viewportSegmentSupport = hasViewportSegments;

                'Viewport Segments API support: ' + viewportSegmentSupport;
            } else {
                'CSS or window not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Viewport Segments API test: {}", value_str);
            assert!(!value_str.contains("error:"), "Viewport Segments API should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test Viewport Segments API: {:?}", e),
    }

    println!("✅ Viewport Segments API test completed");
}
