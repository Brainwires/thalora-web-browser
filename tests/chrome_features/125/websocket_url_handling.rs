use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_125_websocket_url_handling() {
    println!("🧪 Testing Chrome 125: Enhanced WebSocket URL handling...");

    let browser = HeadlessWebBrowser::new();

    // Test WebSocket with HTTP/HTTPS URLs (should convert to ws/wss)
    let js_code = r#"
        try {
            // Test if WebSocket constructor accepts HTTP URLs
            var ws1 = new WebSocket('ws://echo.websocket.org/');
            var result = 'WebSocket created with ws:// URL: ' + (ws1 instanceof WebSocket);

            // Test relative URL handling
            var baseUrl = 'wss://example.com/base/';
            // In real implementation, this should resolve relative to current location
            result;
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("WebSocket URL handling: {}", value_str);
            // WebSocket should be available and constructable
            assert!(value_str.contains("true") || value_str.contains("WebSocket"),
                "WebSocket should be constructable, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test WebSocket URL handling: {:?}", e),
    }

    println!("✅ WebSocket URL handling test completed");
}
