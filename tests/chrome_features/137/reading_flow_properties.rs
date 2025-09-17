use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_137_reading_flow_properties() {
    println!("🧪 Testing Chrome 137: reading-flow and reading-order CSS properties...");

    let browser = HeadlessWebBrowser::new();

    // Test reading-flow and reading-order CSS properties
    let js_code = r#"
        try {
            if (typeof CSS !== 'undefined' && CSS.supports) {
                // Test reading-flow property
                var supportsReadingFlow = CSS.supports('reading-flow', 'flex-visual');

                // Test reading-order property
                var supportsReadingOrder = CSS.supports('reading-order', '1');

                'reading-flow: ' + supportsReadingFlow + ', reading-order: ' + supportsReadingOrder;
            } else {
                'CSS.supports not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Reading flow properties test: {}", value_str);
            assert!(!value_str.contains("error:"), "Reading flow properties should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test reading flow properties: {:?}", e),
    }

    println!("✅ Reading flow properties test completed");
}
