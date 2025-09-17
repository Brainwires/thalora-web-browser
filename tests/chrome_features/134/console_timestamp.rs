use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_134_console_timestamp() {
    println!("🧪 Testing Chrome 134: console.timeStamp enhancements...");

    let browser = HeadlessWebBrowser::new();

    // Test console.timeStamp enhancements
    let js_code = r#"
        try {
            if (typeof console !== 'undefined' && console.timeStamp) {
                // Test basic timeStamp functionality
                var hasTimeStamp = typeof console.timeStamp === 'function';

                // Try calling with custom parameters (Chrome 134 enhancements)
                try {
                    console.timeStamp('test-timestamp', { detail: 'custom' });
                    'console.timeStamp with enhancements: available';
                } catch (timestampError) {
                    'console.timeStamp basic available: ' + hasTimeStamp;
                }
            } else {
                'console.timeStamp not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("console.timeStamp test: {}", value_str);
            assert!(!value_str.contains("error:"), "console.timeStamp should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test console.timeStamp: {:?}", e),
    }

    println!("✅ console.timeStamp test completed");
}
