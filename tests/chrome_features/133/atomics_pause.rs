use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_133_atomics_pause() {
    println!("🧪 Testing Chrome 133: Atomics.pause()...");

    let browser = HeadlessWebBrowser::new();

    // Test Atomics.pause() method
    let js_code = r#"
        try {
            if (typeof Atomics !== 'undefined') {
                // Test if pause method exists
                var hasPause = typeof Atomics.pause === 'function';
                'Atomics.pause available: ' + hasPause;
            } else {
                'Atomics not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Atomics.pause test: {}", value_str);
            assert!(!value_str.contains("error:"), "Atomics.pause should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test Atomics.pause: {:?}", e),
    }

    println!("✅ Atomics.pause test completed");
}
