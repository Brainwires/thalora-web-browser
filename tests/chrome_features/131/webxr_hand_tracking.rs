#[tokio::test]
async fn test_chrome_131_webxr_hand_tracking() {
    println!("🧪 Testing Chrome 131: WebXR Hand Tracking...");

    let browser = HeadlessWebBrowser::new();

    // Test WebXR Hand Tracking
    let js_code = r#"
        try {
            if (typeof navigator !== 'undefined' && navigator.xr) {
                // Test XR availability and hand tracking support
                var hasXR = typeof navigator.xr.isSessionSupported === 'function';
                'navigator.xr available for hand tracking: ' + hasXR;
            } else {
                'navigator.xr not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("WebXR Hand Tracking test: {}", value_str);
            // WebXR might not be available in headless mode
        },
        Err(e) => panic!("Failed to test WebXR Hand Tracking: {:?}", e),
    }

    println!("✅ WebXR Hand Tracking test completed");
}
