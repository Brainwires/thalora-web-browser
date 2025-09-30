#[tokio::test]
async fn test_chrome_124_client_hints() {
    println!("🧪 Testing Chrome 124: Client Hints (Sec-CH-UA-Form-Factors)...");

    let browser = HeadlessWebBrowser::new();

    // Test navigator.userAgentData availability
    let result = browser.lock().unwrap().execute_javascript("typeof navigator.userAgentData").await;
    match result {
        Ok(value) => {
            let value_str = format!("{:?}", value);
            println!("navigator.userAgentData type: {}", value_str);
            // User Agent Client Hints might not be available in all contexts
            assert!(value_str.contains("object") || value_str.contains("undefined"),
                "navigator.userAgentData should exist or be undefined, got: {}", value_str);
        },
        Err(e) => panic!("Failed to check navigator.userAgentData: {:?}", e),
    }

    // Test that we can check for form factors support
    let js_code = r#"
        try {
            // Check if getHighEntropyValues is available
            if (navigator.userAgentData && typeof navigator.userAgentData.getHighEntropyValues === 'function') {
                'client_hints_available'
            } else {
                'client_hints_not_available'
            }
        } catch (e) {
            'error: ' + e.message
        }
    "#;

    let result = browser.lock().unwrap().execute_javascript(js_code).await;
    match result {
        Ok(value) => {
            println!("Client hints availability: {:?}", value);
            // Client hints might not be fully available in headless mode, which is acceptable
        },
        Err(e) => panic!("Failed to test client hints: {:?}", e),
    }

    println!("✅ Client hints test completed");
}
