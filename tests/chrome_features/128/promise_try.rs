#[tokio::test]
async fn test_chrome_128_promise_try() {
    println!("🧪 Testing Chrome 128: Promise.try...");

    let browser = HeadlessWebBrowser::new();

    // Test Promise.try() static method
    let js_code = r#"
        try {
            if (typeof Promise.try === 'function') {
                // Test Promise.try with synchronous function
                var result = Promise.try(() => {
                    return 'sync success';
                });

                // Test Promise.try with throwing function
                var errorResult = Promise.try(() => {
                    throw new Error('test error');
                });

                'Promise.try available: ' + (result instanceof Promise) + ', error handling: ' + (errorResult instanceof Promise);
            } else {
                'Promise.try not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("Promise.try test: {}", value_str);
            assert!(!value_str.contains("error:"), "Promise.try should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test Promise.try: {:?}", e),
    }

    println!("✅ Promise.try test completed");
}
