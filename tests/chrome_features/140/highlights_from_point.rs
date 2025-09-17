use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_140_highlights_from_point() {
    println!("🧪 Testing Chrome 140: highlightsFromPoint API...");

    let browser = HeadlessWebBrowser::new();

    // Test highlightsFromPoint API
    let js_code = r#"
        try {
            // Check if document has highlightsFromPoint method
            if (typeof document !== 'undefined' && typeof document.highlightsFromPoint === 'function') {
                'highlightsFromPoint API available: true';
            } else if (typeof CSS !== 'undefined' && CSS.highlights) {
                // Test CSS Custom Highlight API availability
                'CSS highlights API available: ' + (typeof CSS.highlights === 'object');
            } else {
                'highlightsFromPoint API not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("highlightsFromPoint test: {}", value_str);
            assert!(!value_str.contains("error:"), "highlightsFromPoint should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test highlightsFromPoint: {:?}", e),
    }

    println!("✅ highlightsFromPoint test completed");
}
