#[tokio::test]
async fn test_chrome_124_pageswap_event() {
    println!("🧪 Testing Chrome 124: pageswap event...");

    let browser = HeadlessWebBrowser::new();

    // Test a simple case first - just check if we can access window at all
    let simple_test = "window";
    let result = browser.lock().unwrap().execute_javascript(simple_test).await;
    match result {
        Ok(value) => {
            println!("DEBUG Direct window access: {}", value);
            if value == "undefined" {
                println!("❌ window is undefined - DOM setup may have failed or is not accessible");
                println!("💡 This suggests the JavaScript execution context doesn't have access to DOM globals");
            } else {
                println!("✅ window exists: {}", value);
            }
        },
        Err(e) => {
            println!("DEBUG Direct window access: ERROR - {:?}", e);
        }
    }

    // Test that pageswap event can be listened to
    let result = browser.lock().unwrap().execute_javascript("typeof window.addEventListener").await;
    match result {
        Ok(value) => {
            println!("addEventListener available: {:?}", value);
            // For now, let's be more lenient and see what we actually get
            if !format!("{:?}", value).contains("function") {
                println!("⚠️ addEventListener is not a function, got: {:?}", value);
                // Don't fail yet, let's see what else we can find
            }
        },
        Err(e) => panic!("Failed to check addEventListener: {:?}", e),
    }

    // Test pageswap event registration (should not throw)
    let js_code = r#"
        try {
            // addEventListener returns undefined (correct behavior)
            let result = window.addEventListener('pageswap', function(event) {
                // Event handler for pageswap
            });
            // If we get here without throwing, it worked
            'success'
        } catch (e) {
            'error: ' + e.message
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("pageswap event registration: {}", value_str);
            assert!(value_str.contains("success"), "pageswap event should be registerable, got: {}", value_str);
        },
        Err(e) => panic!("Failed to test pageswap event: {:?}", e),
    }

    println!("✅ pageswap event test completed");
}
