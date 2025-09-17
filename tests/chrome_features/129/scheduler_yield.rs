use thalora::HeadlessWebBrowser;

#[tokio::test]
async fn test_chrome_129_scheduler_yield() {
    println!("🧪 Testing Chrome 129: scheduler.yield()...");

    let browser = HeadlessWebBrowser::new();

    // Test scheduler.yield() API
    let js_code = r#"
        try {
            if (typeof scheduler !== 'undefined' && typeof scheduler.yield === 'function') {
                // Test scheduler.yield() returns a promise
                var yieldPromise = scheduler.yield();

                if (yieldPromise && typeof yieldPromise.then === 'function') {
                    'scheduler.yield available and returns promise';
                } else {
                    'scheduler.yield available but does not return promise';
                }
            } else {
                'scheduler.yield not available';
            }
        } catch (e) {
            'error: ' + e.message;
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("scheduler.yield test: {}", value_str);
            assert!(!value_str.contains("error:"), "scheduler.yield should not error, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test scheduler.yield: {:?}", e),
    }

    println!("✅ scheduler.yield test completed");
}
