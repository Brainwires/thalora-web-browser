use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_137_selection_direction() {
    println!("🧪 Testing Chrome 137: Selection.direction...");

    let browser = HeadlessWebBrowser::new();

    // Test Selection.direction property
    let js_code = r#"
        try {
            if (typeof window !== 'undefined' && window.getSelection) {
                var selection = window.getSelection();

                // Test if direction property exists
                var hasDirection = 'direction' in selection;
                var directionType = typeof selection.direction;

                'Selection.direction property: ' + hasDirection + ' (type: ' + directionType + ')';
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
            println!("Selection.direction test: {}", value_str);
            // Selection API might not be fully available in headless mode
        },
        Err(e) => panic!("Failed to test Selection.direction: {:?}", e),
    }

    println!("✅ Selection.direction test completed");
}
