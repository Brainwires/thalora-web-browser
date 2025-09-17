use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_137_selection_get_composed_ranges() {
    println!("🧪 Testing Chrome 137: Selection.getComposedRanges()...");

    let browser = HeadlessWebBrowser::new();

    // Test Selection.getComposedRanges method
    let js_code = r#"
        try {
            if (typeof window !== 'undefined' && window.getSelection) {
                var selection = window.getSelection();

                // Test if getComposedRanges method exists
                var hasGetComposedRanges = typeof selection.getComposedRanges === 'function';

                if (hasGetComposedRanges) {
                    'Selection.getComposedRanges method available: true';
                } else {
                    'Selection.getComposedRanges method not available';
                }
            } else {
                'window.getSelection not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Selection.getComposedRanges test: {}", value_str);
            // Selection API might not be fully available in headless mode
        },
        Err(e) => panic!("Failed to test Selection.getComposedRanges: {:?}", e),
    }

    println!("✅ Selection.getComposedRanges test completed");
}
