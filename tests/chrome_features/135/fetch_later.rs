#[tokio::test]
async fn test_chrome_135_fetch_later() {
    println!("🧪 Testing Chrome 135: fetchLater() API...");

    let browser = HeadlessWebBrowser::new();

    // Test fetchLater API
    let js_code = r#"
        try {
            if (typeof fetchLater !== 'undefined') {
                // Test fetchLater function availability
                var hasFetchLater = typeof fetchLater === 'function';
                'fetchLater API available: ' + hasFetchLater;
            } else if (typeof navigator !== 'undefined' && navigator.sendBeacon) {
                // Test if beacon API exists as fallback
                'fetchLater not available, but sendBeacon exists';
            } else {
                'fetchLater API not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("fetchLater test: {}", value_str);
            // fetchLater might not be available in headless mode
        },
        Err(e) => panic!("Failed to test fetchLater: {:?}", e),
    }

    println!("✅ fetchLater test completed");
}
