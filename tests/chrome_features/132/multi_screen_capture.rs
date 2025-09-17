#[tokio::test]
async fn test_chrome_132_multi_screen_capture() {
    println!("🧪 Testing Chrome 132: Multi-Screen Capture API...");

    let browser = HeadlessWebBrowser::new();

    // Test Multi-Screen Capture API
    let js_code = r#"
        try {
            if (typeof navigator !== 'undefined' && navigator.mediaDevices) {
                // Test if getAllScreensMedia is available
                var hasGetAllScreensMedia = typeof navigator.mediaDevices.getAllScreensMedia === 'function';
                'getAllScreensMedia available: ' + hasGetAllScreensMedia;
            } else {
                'navigator.mediaDevices not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Multi-Screen Capture test: {}", value_str);
            // Multi-screen capture might require enterprise policy
        },
        Err(e) => panic!("Failed to test Multi-Screen Capture: {:?}", e),
    }

    println!("✅ Multi-Screen Capture test completed");
}
